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
    function::FunctionTemplate,
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

    fn try_struct_expr<R: Rng>(
        &self,
        struct_name: &String,
        expr_choice: StructExprVariants,
        rng: &mut R,
    ) -> Option<StructExpr> {
        match expr_choice {
            StructExprVariants::Func => {
                let struct_func_filter = Filters::new()
                    .with_filters(vec![is_type_filter(self.type_id.clone()), is_func_filter()]);

                if struct_func_filter
                    .filter(&self.context.borrow().scope)
                    .len()
                    > 0
                {
                    match self.func_call_expr(rng) {
                        Some(expr) => Some(StructExpr::Func(expr)),
                        None => None,
                    }
                } else {
                    None
                }
            }
            StructExprVariants::Var => {
                let struct_var_filter = self.make_struct_filter();
                if struct_var_filter.filter(&self.context.borrow().scope).len() > 0 {
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
                    Some(StructExpr::Var(var))
                } else {
                    None
                }
            }
            StructExprVariants::Literal => {
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
                Some(StructExpr::Literal(
                    self.struct_literal(struct_template, rng),
                ))
            }
        }
    }

    fn make_struct_filter(&self) -> Filters {
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
        struct_var_filter
    }

    fn struct_expr<R: Rng>(&self, struct_name: String, rng: &mut R) -> StructExpr {
        let mut expr_choice: StructExprVariants = rng.gen();
        let loop_limit = 100;
        for _ in 0..loop_limit {
            if let Some(expr) = self.try_struct_expr(&struct_name, expr_choice, rng) {
                return expr;
            } else {
                expr_choice = rng.gen();
            }
        }

        panic!("Could not generate stmt");
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

    fn try_arith_expr<R: Rng>(
        &self,
        expr_choice: ArithmeticExprVariants,
        rng: &mut R,
    ) -> Option<ArithmeticExpr> {
        match expr_choice {
            ArithmeticExprVariants::Int => Some(self.int_expr(rng).as_arith_expr()),
            ArithmeticExprVariants::Binary => Some(self.binary_int_expr(rng).as_arith_expr()),
            ArithmeticExprVariants::Var => {
                let arith_var_filter = Filters::new().with_filters(vec![
                    is_var_filter(),
                    is_type_filter(self.type_id.clone()),
                    is_borrow_type_filter(self.borrow_type_id),
                    is_not_mut_borrowed_filter(),
                ]);
                match self.var_from_filter(arith_var_filter, rng) {
                    Some(expr) => Some(expr.into()),
                    None => None,
                }
            }
            ArithmeticExprVariants::Func => match self.func_call_expr(rng) {
                Some(expr) => Some(expr.into()),
                None => None,
            },
        }
    }

    fn arith_expr<R: Rng>(&self, rng: &mut R) -> ArithmeticExpr {
        self.context.borrow_mut().arith_expr_depth += 1;

        let mut expr_choice: ArithmeticExprVariants = rng.gen();

        if self.context.borrow_mut().arith_expr_depth > consts::MAX_ARITH_EXPR_DEPTH {
            expr_choice = ArithmeticExprVariants::Int;
        }

        let loop_limit = 100;
        for _ in 0..loop_limit {
            if let Some(expr) = self.try_arith_expr(expr_choice, rng) {
                return expr;
            } else {
                expr_choice = rng.gen();
            }
        }
        panic!("Could not generate arithmetic expr");
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

    fn try_bool_expr<R: Rng>(
        &self,
        expr_choice: BoolExprVariants,
        rng: &mut R,
    ) -> Option<BoolExpr> {
        match expr_choice {
            BoolExprVariants::Binary => Some(self.binary_bool_expr(rng).as_bool_expr()),
            BoolExprVariants::Comparison => Some(self.comparison_expr(rng).as_bool_expr()),

            BoolExprVariants::Negation => Some(self.negation_expr(rng).as_bool_expr()),
            BoolExprVariants::Func => match self.func_call_expr(rng) {
                Some(func_call_expr) => Some(func_call_expr.into()),
                None => None,
            },
            BoolExprVariants::Var => {
                let bool_var_filter = Filters::new().with_filters(vec![
                    is_var_filter(),
                    is_type_filter(self.type_id.clone()),
                    is_borrow_type_filter(self.borrow_type_id),
                    is_not_mut_borrowed_filter(),
                ]);

                match self.var_from_filter(bool_var_filter, rng) {
                    Some(expr) => Some(expr.into()),
                    None => None,
                }
            }
            BoolExprVariants::Bool => Some(self.bool_literal(rng).as_bool_expr()),
        }
    }

    fn bool_expr<R: Rng>(&self, rng: &mut R) -> BoolExpr {
        self.context.borrow_mut().bool_expr_depth += 1;

        let mut expr_choice: BoolExprVariants = rng.gen();

        if self.context.borrow_mut().bool_expr_depth > consts::MAX_BOOL_EXPR_DEPTH {
            expr_choice = BoolExprVariants::Bool;
        }

        let loop_limit = 100;
        for _ in 0..loop_limit {
            if let Some(expr) = self.try_bool_expr(expr_choice, rng) {
                return expr;
            } else {
                expr_choice = rng.gen();
            }
        }

        panic!("Could not generate boolean expr");
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

    fn var_from_filter<R: Rng>(&self, filters: Filters, rng: &mut R) -> Option<Var> {
        let var_list = filters.filter(&self.context.borrow().scope);
        let var_choice = var_list.choose(rng);

        if let None = var_choice {
            return None;
        }
        let var_choice = var_choice.unwrap();

        Some(Var::new(self.type_id.clone(), var_choice.0.clone(), false))
    }

    pub fn func_call_expr_from_template<R: Rng>(
        &self,
        function_template: FunctionTemplate,
        rng: &mut R,
    ) -> FunctionCallExpr {
        let mut arguments: Vec<Expr> = Vec::new();

        for param in function_template.params_iter() {
            let generator =
                ExprGenerator::new_sub_expr(self, param.get_type(), param.get_borrow_type());

            match param.get_borrow_type() {
                BorrowTypeID::None => arguments.push(generator.expr(rng)),
                BorrowTypeID::Ref => arguments.push(generator.borrow_expr(rng).as_expr()),
                BorrowTypeID::MutRef => {
                    arguments.push(generator.func_mut_borrow_expr(rng).as_expr())
                }
            }
        }

        FunctionCallExpr::new(function_template, arguments)
    }

    // Assumes that the function with the correct type already exists
    fn func_call_expr<R: Rng>(&self, rng: &mut R) -> Option<FunctionCallExpr> {
        let filters = Filters::new()
            .with_filters(vec![is_func_filter(), is_type_filter(self.type_id.clone())]);

        let func_list: Vec<(String, (Rc<ScopeEntry>, BorrowStatus))> =
            filters.filter(&self.context.borrow().scope);

        let choice = func_list.choose(rng);

        match choice {
            None => return None,
            Some(_) => (),
        }

        let (_entry_name, (entry_choice, _)) = choice.unwrap();

        if let ScopeEntry::Func(func_scope_entry) = entry_choice.as_ref() {
            Some(self.func_call_expr_from_template(func_scope_entry.get_template(), rng))
        } else {
            None
        }
    }

    // TODO: Force mutable borrow on global struct? Prevent instantiation
    pub fn borrow_expr<R: Rng>(&self, rng: &mut R) -> BorrowExpr {
        match self.borrow_type_id {
            BorrowTypeID::Ref => self.immut_borrow_expr(rng),
            BorrowTypeID::MutRef => self.mut_borrow_expr(rng),
            _ => panic!("Expr generator calling borrow expr when borrow type is none"),
        }
    }

    // A mutable borrow in a function constitutes a 'use' of that borrow
    // All previous variable borrows must go out of scope
    fn func_mut_borrow_expr<R: Rng>(&self, rng: &mut R) -> BorrowExpr {
        let filters = Filters::new().with_filters(vec![
            is_type_filter(self.type_id.clone()),
            is_var_struct_filter(),
            is_mut_or_mut_ref_filter(),
        ]);

        let entries: Vec<(String, (Rc<ScopeEntry>, BorrowStatus))> = filters
            .filter(&self.context.borrow().scope)
            .into_iter()
            .filter(|(entry_name, (_, _))| {
                match self
                    .context
                    .borrow()
                    .scope
                    .borrow()
                    .lookup_borrow_context(entry_name)
                {
                    Some(borrow_context) => !borrow_context.is_func_mut_borrowed(),
                    None => false,
                }
            })
            .collect();
        let choice = entries.choose(rng);

        match choice {
            Some((entry_name, (scope_entry, _))) => {
                let var = Var::new(self.type_id.clone(), entry_name.clone(), false);

                self.context
                    .borrow()
                    .scope
                    .borrow_mut()
                    .func_mut_borrow(&entry_name);

                // We explicitly borrow if the borrow type isn't a mut ref (i.e. it's a literal so we have to &mut)
                BorrowExpr::new(
                    BorrowTypeID::MutRef,
                    var.as_expr(),
                    scope_entry.get_borrow_type() != BorrowTypeID::MutRef,
                )
            }
            None => BorrowExpr::new(self.borrow_type_id, self.literal_expr(rng), true),
        }
    }

    // Mutable borrow expr DOES NOT remove previous borrows
    // This borrow expr is strictly for instantiation
    fn mut_borrow_expr<R: Rng>(&self, rng: &mut R) -> BorrowExpr {
        let filters = Filters::new().with_filters(vec![
            is_type_filter(self.type_id.clone()),
            is_var_struct_filter(),
            is_mut_or_mut_ref_filter(),
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
                BorrowExpr::new(
                    BorrowTypeID::MutRef,
                    var.as_expr(),
                    scope_entry.get_borrow_type() != BorrowTypeID::MutRef,
                )
            }
            None => BorrowExpr::new(self.borrow_type_id, self.literal_expr(rng), true),
        }
    }

    fn immut_borrow_expr<R: Rng>(&self, rng: &mut R) -> BorrowExpr {
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

                BorrowExpr::new(
                    BorrowTypeID::Ref,
                    var.as_expr(),
                    scope_entry.get_borrow_type() != BorrowTypeID::Ref,
                )
            }
            None => BorrowExpr::new(self.borrow_type_id, self.literal_expr(rng), true),
        }
    }

    pub fn int32<R: Rng>(rng: &mut R) -> IntExpr {
        IntExpr::new_i32(rng.gen::<i32>())
    }
}
