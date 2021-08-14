use std::{cell::RefCell, rc::Rc};

use crate::program::{
    expr::{
        arithmetic_expr::{ArithmeticExpr, BinaryExpr, BinaryOp, IntExpr, IntValue},
        bool_expr::{
            BinBoolExpr, BoolExpr, BoolOp, BoolValue, ComparisonExpr, ComparisonOp, NegationExpr,
        },
        borrow_expr::BorrowExpr,
        expr::Expr,
        func_call_expr::FunctionCallExpr,
        struct_expr::{StructExpr, StructLiteral},
    },
    struct_template::StructTemplate,
    types::{BorrowStatus, BorrowTypeID, IntTypeID, TypeID},
    var::Var,
};
use rand::{seq::SliceRandom, Rng};

use super::{
    consts, context::Context, filters::*, scope_entry::ScopeEntry, struct_gen::StructTable,
    weights::expr::variants::*,
};

pub struct ExprGenerator<'table> {
    struct_table: &'table StructTable,
    context: Rc<RefCell<Context>>,
    type_id: TypeID,
    borrow_type_id: BorrowTypeID,
}

impl<'table> ExprGenerator<'table> {
    pub fn new(
        struct_table: &'table StructTable,
        context: Rc<RefCell<Context>>,
        type_id: TypeID,
        borrow_type_id: BorrowTypeID,
    ) -> Self {
        ExprGenerator {
            struct_table,
            context,
            type_id,
            borrow_type_id,
        }
    }

    pub fn new_sub_expr(
        other: &'table ExprGenerator,
        type_id: TypeID,
        borrow_type_id: BorrowTypeID,
    ) -> Self {
        other.context.borrow_mut().expr_depth += 1;
        ExprGenerator {
            struct_table: &other.struct_table,
            context: Rc::clone(&other.context),
            type_id,
            borrow_type_id,
        }
    }

    pub fn expr<R: Rng>(&self, rng: &mut R) -> Expr {
        self.context.borrow_mut().expr_depth += 1;
        if self.context.borrow().expr_depth < consts::MAX_EXPR_DEPTH {
            match &self.type_id {
                TypeID::IntType(_) => self.arith_expr(rng).as_expr(),
                TypeID::StructType(struct_name) => {
                    self.struct_expr(struct_name.clone(), rng).as_expr()
                }
                TypeID::BoolType => self.bool_expr(rng).as_expr(),
                TypeID::NullType => panic!("Tried to construct an expression of null type"),
            }
        } else {
            self.literal_expr(rng)
        }
    }

    // TODO: Ideally we shouldn't have this, and use a context to decide where to go
    pub fn literal_expr<R: Rng>(&self, rng: &mut R) -> Expr {
        match &self.type_id {
            TypeID::IntType(_) => self.int_expr(rng).as_expr(),
            TypeID::StructType(struct_name) => self
                .struct_literal(
                    self.struct_table.get_struct_template(&struct_name).unwrap(),
                    rng,
                )
                .as_struct_expr()
                .as_expr(),
            TypeID::BoolType => self.bool_literal(rng).as_bool_expr().as_expr(),
            TypeID::NullType => panic!("Tried to construct an expression of null type"),
        }
    }

    pub fn global_struct_expr<R: Rng>(&self, rng: &mut R) -> StructExpr {
        let struct_template = self.struct_table.get_global_struct().unwrap();

        StructExpr::Literal(self.struct_literal(struct_template, rng))
    }

    fn struct_expr<R: Rng>(&self, struct_name: String, rng: &mut R) -> StructExpr {
        let mut struct_var_filter = Filters::new().with_filters(vec![
            is_struct_filter(),
            is_type_filter(self.type_id.clone()),
            is_borrow_type_filter(self.borrow_type_id),
        ]);

        // Preventing moves if we're inside a loop
        if self.context.borrow().loop_depth > 0 {
            struct_var_filter.add_filter(is_not_borrow_type_filter(BorrowTypeID::None));
        } else if self.borrow_type_id == BorrowTypeID::None {
            struct_var_filter.add_full_filter(can_move_filter(self.context.borrow().scope.clone()));
        }

        let expr_choice: StructExprVariants = rng.gen();

        match expr_choice {
            StructExprVariants::Var
                if struct_var_filter.filter(&self.context.borrow().scope).len() > 0 =>
            {
                let choice = struct_var_filter
                    .filter(&self.context.borrow().scope)
                    .choose(rng)
                    .unwrap()
                    .clone();
                let var = Var::new(self.type_id.clone(), choice.0, false);

                // Move only happens if it's not borrow
                // For struct expression, using the expression is equivalent to a move
                if self.borrow_type_id == BorrowTypeID::None {
                    self.context
                        .borrow()
                        .scope
                        .borrow_mut()
                        .remove_entry(&var.get_name());
                }

                // Return variable
                StructExpr::Var(var)
            }
            StructExprVariants::Literal | _ => {
                let struct_template = self
                    .struct_table
                    .get_struct_template(&struct_name)
                    .unwrap_or_else(|| {
                        panic!(
                            "Table: {:?}, searching: {}",
                            self.struct_table,
                            struct_name.clone()
                        );
                    });
                StructExpr::Literal(self.struct_literal(struct_template, rng))
            }
        }
    }

    fn struct_literal<R: Rng>(
        &self,
        struct_template: StructTemplate,
        rng: &mut R,
    ) -> StructLiteral {
        let mut field_values: Vec<Expr> = Vec::new();

        for (_, field_type_id) in struct_template.fields_iter() {
            // TODO: allow different borrows other than move for struct fields
            // Only needed when there are references in struct fields
            let generator =
                ExprGenerator::new_sub_expr(self, field_type_id.clone(), BorrowTypeID::None);

            let field_expr = generator.expr(rng);
            field_values.push(field_expr);
        }

        StructLiteral::new(struct_template, field_values)
    }

    fn arith_expr<R: Rng>(&self, rng: &mut R) -> ArithmeticExpr {
        self.context.borrow_mut().arith_expr_depth += 1;

        let expr_choice: ArithmeticExprVariants = rng.gen();
        let arith_var_filter = Filters::new().with_filters(vec![
            is_var_filter(),
            is_type_filter(self.type_id.clone()),
            is_borrow_type_filter(self.borrow_type_id),
            is_not_mut_borrowed_filter(),
        ]);

        let arith_func_filter = Filters::new().with_filters(vec![
            is_func_filter(),
            is_type_filter(self.type_id.clone()),
            is_borrow_type_filter(self.borrow_type_id),
        ]);

        match expr_choice {
            ArithmeticExprVariants::Binary
                if self.context.borrow().arith_expr_depth < consts::MAX_ARITH_EXPR_DEPTH =>
            {
                self.binary_int_expr(rng).as_arith_expr()
            }
            ArithmeticExprVariants::Var
                if arith_var_filter.filter(&self.context.borrow().scope).len() > 0 =>
            {
                ArithmeticExpr::Var(self.var_from_filter(arith_var_filter, rng))
            }

            // We constrain nested function call depth to be the same as binary expr depth
            ArithmeticExprVariants::Func
                if arith_func_filter.filter(&self.context.borrow().scope).len() > 0
                    && self.context.borrow().expr_depth < consts::MAX_EXPR_DEPTH =>
            {
                let result = self.func_call_expr(rng);

                match result {
                    Ok(func_call_expr) => ArithmeticExpr::Func(func_call_expr),
                    Err(s) => panic!("{}", s),
                }
            }
            ArithmeticExprVariants::Int | _ => self.int_expr(rng).as_arith_expr(),
        }
    }

    fn binary_int_expr<R: Rng>(&self, rng: &mut R) -> BinaryExpr {
        let op: BinaryOp = rng.gen();

        let left = self.arith_expr(rng);
        let right = self.arith_expr(rng);

        BinaryExpr::new(left, right, op)
    }

    fn int_expr<R: Rng>(&self, rng: &mut R) -> IntExpr {
        if let TypeID::IntType(int_type_id) = self.type_id {
            IntExpr::new(IntValue::rand_from_type(int_type_id.clone(), rng))
        } else {
            panic!("Rand int expr called but generator not instantiated with integer type")
        }
    }

    fn bool_expr<R: Rng>(&self, rng: &mut R) -> BoolExpr {
        self.context.borrow_mut().bool_expr_depth += 1;

        let expr_choice: BoolExprVariants = rng.gen();
        let bool_var_filter = Filters::new().with_filters(vec![
            is_var_filter(),
            is_type_filter(self.type_id.clone()),
            is_borrow_type_filter(self.borrow_type_id),
            is_not_mut_borrowed_filter(),
        ]);

        let bool_func_filter = |scope_entry: Rc<ScopeEntry>, _| -> bool {
            scope_entry.is_func()
                && scope_entry.get_type() == self.type_id
                && scope_entry.get_borrow_type() == self.borrow_type_id
        };

        match expr_choice {
            BoolExprVariants::Binary
                if self.context.borrow().bool_expr_depth < consts::MAX_BOOL_EXPR_DEPTH =>
            {
                self.binary_bool_expr(rng).as_bool_expr()
            }
            BoolExprVariants::Comparison
                if self.context.borrow().bool_expr_depth < consts::MAX_BOOL_EXPR_DEPTH =>
            {
                self.comparison_expr(rng).as_bool_expr()
            }
            BoolExprVariants::Negation
                if self.context.borrow().bool_expr_depth < consts::MAX_BOOL_EXPR_DEPTH =>
            {
                self.negation_expr(rng).as_bool_expr()
            }
            BoolExprVariants::Func
                if self
                    .context
                    .borrow()
                    .scope
                    .borrow()
                    .contains_filter(bool_func_filter)
                    && self.context.borrow().bool_expr_depth < consts::MAX_BOOL_EXPR_DEPTH =>
            {
                let result = self.func_call_expr(rng);
                match result {
                    Ok(func_call_expr) => BoolExpr::Func(func_call_expr),
                    Err(s) => panic!("{}", s),
                }
            }
            BoolExprVariants::Var
                if bool_var_filter.filter(&self.context.borrow().scope).len() > 0 =>
            {
                BoolExpr::Var(self.var_from_filter(bool_var_filter, rng))
            }
            BoolExprVariants::Bool | _ => self.bool_literal(rng).as_bool_expr(),
        }
    }

    fn bool_literal<R: Rng>(&self, rng: &mut R) -> BoolValue {
        BoolValue::new(rng.gen::<bool>())
    }

    fn binary_bool_expr<R: Rng>(&self, rng: &mut R) -> BinBoolExpr {
        let op: BoolOp = rng.gen();

        let left = self.bool_expr(rng);
        let right = self.bool_expr(rng);

        BinBoolExpr::new(left, right, op)
    }

    fn comparison_expr<R: Rng>(&self, rng: &mut R) -> ComparisonExpr {
        let op: ComparisonOp = rng.gen();

        let int_type: IntTypeID = rng.gen();

        // TODO: Borrow type
        let generator = ExprGenerator::new_sub_expr(self, int_type.as_type(), BorrowTypeID::None);

        let left = generator.arith_expr(rng);
        let right = generator.arith_expr(rng);

        ComparisonExpr::new(left, right, op)
    }

    fn negation_expr<R: Rng>(&self, rng: &mut R) -> NegationExpr {
        let bool_expr = self.bool_expr(rng);

        NegationExpr::new(bool_expr)
    }

    fn var_from_filter<R: Rng>(&self, filters: Filters, rng: &mut R) -> Var {
        let var_list = filters.filter(&self.context.borrow().scope);
        let var_choice = var_list.choose(rng).unwrap();

        Var::new(self.type_id.clone(), var_choice.0.clone(), false)
    }

    // Assumes that the function with the correct type already exists
    fn func_call_expr<R: Rng>(&self, rng: &mut R) -> Result<FunctionCallExpr, String> {
        let func_list: Vec<(String, (Rc<ScopeEntry>, BorrowStatus))> = self
            .context
            .borrow()
            .scope
            .borrow()
            .filter_with_closure(|scope_entry, _| {
                scope_entry.is_func() && scope_entry.is_type(self.type_id.clone())
            });

        let (_entry_name, (entry_choice, _)) = func_list.choose(rng).unwrap();

        if let ScopeEntry::Func(func_scope_entry) = entry_choice.as_ref() {
            let function_template = func_scope_entry.get_template();
            let mut arguments: Vec<Expr> = Vec::new();

            for param in function_template.params_iter() {
                let generator =
                    ExprGenerator::new_sub_expr(self, param.get_type(), param.get_borrow_type());

                if param.get_borrow_type() == BorrowTypeID::None {
                    arguments.push(generator.expr(rng));
                } else {
                    let result = generator.borrow_expr(rng);

                    match result {
                        Ok(borrow_exp) => arguments.push(borrow_exp.as_expr()),

                        // If unable to borrow a variable, generate an expression and explicity put a ref on it
                        Err(_) => {
                            let explicit_generator = ExprGenerator::new_sub_expr(
                                self,
                                param.get_type(),
                                BorrowTypeID::None,
                            );

                            // TODO: Allow this to use functions and other more complex expressions
                            //       since literal expr is very constrained
                            // We're only doing this since we want to avoid picking up a variable that
                            // we shouldn't be allowed to take a (mut) reference of
                            let expr = explicit_generator.literal_expr(rng);
                            arguments.push(
                                BorrowExpr::new(param.get_borrow_type(), expr, true).as_expr(),
                            );
                        }
                    }
                }
            }

            Ok(FunctionCallExpr::new(function_template, arguments))
        } else {
            Err(format!(
                "Could not find function with return type {}",
                self.type_id.to_string()
            ))
        }
    }

    // TODO: Force mutable borrow on global struct? Prevent instantiation
    fn borrow_expr<R: Rng>(&self, rng: &mut R) -> Result<BorrowExpr, ()> {
        match self.borrow_type_id {
            BorrowTypeID::Ref => self.immut_borrow_expr(rng),
            BorrowTypeID::MutRef => self.mut_borrow_expr(rng),
            _ => Err(()),
        }
    }

    // TODO: Use a less conservative version
    fn mut_borrow_expr<R: Rng>(&self, rng: &mut R) -> Result<BorrowExpr, ()> {
        let filters = Filters::new().with_filters(vec![
            is_type_filter(self.type_id.clone()),
            is_not_mut_borrowed_filter(),
            is_not_borrowed_filter(),
            is_var_filter(),
            is_struct_filter(),
        ]);

        let entries = filters.filter(&self.context.borrow().scope);
        let choice = entries.choose(rng);

        match choice {
            Some((entry_name, (scope_entry, _))) => {
                let var = Var::new(self.type_id.clone(), entry_name.clone(), false);

                if entry_name.contains('.') {
                    self.context
                        .borrow()
                        .scope
                        .borrow_mut()
                        .mut_borrow_struct_field_entry(&"temp_mut_borrow".to_string(), entry_name);
                } else {
                    self.context
                        .borrow()
                        .scope
                        .borrow_mut()
                        .mut_borrow_entry(&"temp_mut_borrow".to_string(), entry_name);
                }

                // We explicitly borrow if the borrow type isn't a mut ref (i.e. it's a literal so we have to &mut)
                Ok(BorrowExpr::new(
                    BorrowTypeID::MutRef,
                    var.as_expr(),
                    scope_entry.get_borrow_type() != BorrowTypeID::MutRef,
                ))
            }
            None => Err(()),
        }
    }

    fn immut_borrow_expr<R: Rng>(&self, rng: &mut R) -> Result<BorrowExpr, ()> {
        let filters = Filters::new().with_filters(vec![
            is_type_filter(self.type_id.clone()),
            is_not_mut_borrowed_filter(),
            is_var_filter(),
            is_struct_filter(),
        ]);

        let entries = filters.filter(&self.context.borrow().scope);
        let choice = entries.choose(rng);

        match choice {
            Some((entry_name, (scope_entry, _))) => {
                let var = Var::new(self.type_id.clone(), entry_name.clone(), false);

                if entry_name.contains('.') {
                    self.context
                        .borrow()
                        .scope
                        .borrow_mut()
                        .mut_borrow_struct_field_entry(&"temp_borrow".to_string(), entry_name);
                } else {
                    self.context
                        .borrow()
                        .scope
                        .borrow_mut()
                        .mut_borrow_entry(&"temp_borrow".to_string(), entry_name);
                }

                Ok(BorrowExpr::new(
                    BorrowTypeID::Ref,
                    var.as_expr(),
                    scope_entry.get_borrow_type() != BorrowTypeID::Ref,
                ))
            }
            None => Err(()),
        }
    }

    pub fn int32<R: Rng>(rng: &mut R) -> IntExpr {
        IntExpr::new_i32(rng.gen::<i32>())
    }
}
