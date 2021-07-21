use std::{cell::RefCell, rc::Rc};

use crate::program::{
    expr::{
        arithmetic_expr::{
            ArithmeticExpr, ArithmeticExprVariants, BinaryExpr, BinaryOp, IntExpr, IntValue,
        },
        bool_expr::{
            BinBoolExpr, BoolExpr, BoolExprVariants, BoolOp, BoolValue, ComparisonExpr,
            ComparisonOp, NegationExpr,
        },
        borrow_expr::BorrowExpr,
        expr::Expr,
        func_call_expr::FunctionCallExpr,
        struct_expr::{StructExpr, StructExprVariants, StructLiteral},
    },
    struct_template::StructTemplate,
    types::{BorrowStatus, BorrowTypeID, IntTypeID, TypeID},
    var::Var,
};
use rand::{seq::SliceRandom, Rng};

use super::{
    scope::{Scope, ScopeEntry},
    struct_gen::StructTable,
};

pub const MAX_EXPR_DEPTH: u32 = 4;

pub struct ExprGenerator<'a> {
    struct_table: &'a StructTable,
    scope: Rc<RefCell<Scope>>,
    type_id: TypeID,
    borrow_type_id: BorrowTypeID,
    depth: u32,
}

impl<'a> ExprGenerator<'a> {
    pub fn new(
        struct_table: &'a StructTable,
        scope: Rc<RefCell<Scope>>,
        type_id: TypeID,
        borrow_type_id: BorrowTypeID,
        depth: u32,
    ) -> Self {
        ExprGenerator {
            struct_table,
            scope,
            type_id,
            borrow_type_id,
            depth,
        }
    }

    pub fn new_sub_expr(
        other: &'a ExprGenerator,
        type_id: TypeID,
        borrow_type_id: BorrowTypeID,
    ) -> Self {
        ExprGenerator {
            struct_table: &other.struct_table,
            scope: Rc::clone(&other.scope),
            type_id,
            borrow_type_id,
            depth: if other.depth == 0 { 0 } else { other.depth - 1 },
        }
    }

    pub fn expr<R: Rng>(&self, rng: &mut R) -> Expr {
        match &self.type_id {
            TypeID::IntType(_) => self.arith_expr(self.depth, rng).as_expr(),
            TypeID::StructType(struct_name) => self.struct_expr(struct_name.clone(), rng).as_expr(),
            TypeID::BoolType => self.bool_expr(self.depth, rng).as_expr(),
            TypeID::NullType => panic!("Tried to construct an expression of null type"),
        }
    }

    // TODO: Ideally we shouldn't have this, and use a context to decide where to go
    pub fn literal_expr<R: Rng>(&self, rng: &mut R) -> Expr {
        match &self.type_id {
            TypeID::IntType(_) => self.int_expr(rng).as_expr(),
            TypeID::StructType(struct_name) => self
                .struct_literal(
                    self.struct_table
                        .get_struct_template(struct_name.clone())
                        .unwrap(),
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

    pub fn struct_expr<R: Rng>(&self, struct_name: String, rng: &mut R) -> StructExpr {
        let struct_var_filter = |scope_entry: &ScopeEntry, borrow_status: BorrowStatus| -> bool {
            scope_entry.is_struct()
                            && self.type_id == scope_entry.get_type()
                            && self.borrow_type_id == scope_entry.get_borrow_type()
                            // TODO: Change this to allow passing refs
                            && borrow_status == BorrowStatus::None
        };

        let struct_template = self
            .struct_table
            .get_struct_template(struct_name.clone())
            .unwrap_or_else(|| {
                panic!(
                    "Table: {:?}, searching: {}",
                    self.struct_table,
                    struct_name.clone()
                );
            });

        let expr_choice: StructExprVariants = rng.gen();

        match expr_choice {
            StructExprVariants::Var if self.scope.borrow().contains_filter(struct_var_filter) => {
                let var = self.var(struct_var_filter, rng);

                // Move only happens if it's not borrow
                // For struct expression, using the expression is equivalent to a move
                if self.borrow_type_id == BorrowTypeID::None {
                    self.scope.borrow_mut().remove_entry(var.get_name());
                }

                // Return variable
                StructExpr::Var(var)
            }
            StructExprVariants::Literal | _ => {
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
            // Only needed when there are borrows in struct fields
            let generator =
                ExprGenerator::new_sub_expr(self, field_type_id.clone(), BorrowTypeID::None);

            let field_expr = generator.expr(rng);
            field_values.push(field_expr);
        }

        if field_values.len() != struct_template.num_fields() {
            panic!(
                "Did not generate the right amount of fields, expected: {}, got: {}",
                struct_template.num_fields(),
                field_values.len()
            );
        }

        StructLiteral::new(struct_template, field_values)
    }

    pub fn arith_expr<R: Rng>(&self, depth: u32, rng: &mut R) -> ArithmeticExpr {
        let expr_choice: ArithmeticExprVariants = rng.gen();
        let arith_var_filter = |scope_entry: &ScopeEntry, borrow_status: BorrowStatus| -> bool {
            scope_entry.is_var()
                && scope_entry.get_type() == self.type_id
                && scope_entry.get_borrow_type() == self.borrow_type_id
                && borrow_status != BorrowStatus::MutBorrowed
        };

        match expr_choice {
            ArithmeticExprVariants::Binary if depth > 0 => {
                self.binary_int_expr(depth, rng).as_arith_expr()
            }
            ArithmeticExprVariants::Var
                if self.scope.borrow().contains_filter(arith_var_filter) =>
            {
                ArithmeticExpr::Var(self.var(arith_var_filter, rng))
            }

            // We constrain nested function call depth to be the same as binary expr depth
            ArithmeticExprVariants::Func
                if self
                    .scope
                    .borrow()
                    .contains_function_type(self.type_id.clone())
                    && depth > 0 =>
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

    fn binary_int_expr<R: Rng>(&self, depth: u32, rng: &mut R) -> BinaryExpr {
        let op: BinaryOp = rng.gen();

        let left = self.arith_expr(depth - 1, rng);
        let right = self.arith_expr(depth - 1, rng);

        BinaryExpr::new(left, right, op)
    }

    fn int_expr<R: Rng>(&self, rng: &mut R) -> IntExpr {
        if let TypeID::IntType(int_type_id) = self.type_id {
            IntExpr::new(IntValue::rand_from_type(int_type_id.clone(), rng))
        } else {
            panic!("Rand int expr called but generator not instantiated with integer type")
        }
    }

    pub fn bool_expr<R: Rng>(&self, depth: u32, rng: &mut R) -> BoolExpr {
        let expr_choice: BoolExprVariants = rng.gen();
        let bool_var_filter = |scope_entry: &ScopeEntry, borrow_status: BorrowStatus| -> bool {
            scope_entry.is_var()
                && scope_entry.get_type() == self.type_id
                && scope_entry.get_borrow_type() == self.borrow_type_id
                && borrow_status != BorrowStatus::MutBorrowed
        };
        match expr_choice {
            BoolExprVariants::Binary if depth > 0 => {
                self.binary_bool_expr(depth, rng).as_bool_expr()
            }
            BoolExprVariants::Comparison if depth > 0 => {
                self.comparison_expr(depth, rng).as_bool_expr()
            }
            BoolExprVariants::Negation if depth > 0 => {
                self.negation_expr(depth, rng).as_bool_expr()
            }
            BoolExprVariants::Func
                if self
                    .scope
                    .borrow()
                    .contains_function_type(self.type_id.clone())
                    && depth > 0 =>
            {
                let result = self.func_call_expr(rng);
                match result {
                    Ok(func_call_expr) => BoolExpr::Func(func_call_expr),
                    Err(s) => panic!("{}", s),
                }
            }
            BoolExprVariants::Var if self.scope.borrow().contains_filter(bool_var_filter) => {
                BoolExpr::Var(self.var(bool_var_filter, rng))
            }
            BoolExprVariants::Bool | _ => self.bool_literal(rng).as_bool_expr(),
        }
    }

    fn bool_literal<R: Rng>(&self, rng: &mut R) -> BoolValue {
        BoolValue::new(rng.gen::<bool>())
    }

    fn binary_bool_expr<R: Rng>(&self, depth: u32, rng: &mut R) -> BinBoolExpr {
        let op: BoolOp = rng.gen();

        let left = self.bool_expr(depth - 1, rng);
        let right = self.bool_expr(depth - 1, rng);

        BinBoolExpr::new(left, right, op)
    }

    fn comparison_expr<R: Rng>(&self, depth: u32, rng: &mut R) -> ComparisonExpr {
        let op: ComparisonOp = rng.gen();

        let int_type: IntTypeID = rng.gen();

        // TODO: Borrow type
        let generator = ExprGenerator::new_sub_expr(self, int_type.as_type(), BorrowTypeID::None);

        let left = generator.arith_expr(depth - 1, rng);
        let right = generator.arith_expr(depth - 1, rng);

        ComparisonExpr::new(left, right, op)
    }

    fn negation_expr<R: Rng>(&self, depth: u32, rng: &mut R) -> NegationExpr {
        let bool_expr = self.bool_expr(depth - 1, rng);

        NegationExpr::new(bool_expr)
    }

    fn var<T, R: Rng>(&self, filter: T, rng: &mut R) -> Var
    where
        T: Fn(&ScopeEntry, BorrowStatus) -> bool,
    {
        let var_list = self.scope.borrow().filter_with_closure(filter);

        let var_choice = var_list.choose(rng).unwrap();

        Var::new(self.type_id.clone(), var_choice.0.clone(), false)
    }

    // Assumes that the function with the correct type already exists
    fn func_call_expr<R: Rng>(&self, rng: &mut R) -> Result<FunctionCallExpr, String> {
        let func_list: Vec<(String, Rc<ScopeEntry>, BorrowStatus)> =
            self.scope.borrow().filter_with_closure(|scope_entry, _| {
                scope_entry.is_func() && scope_entry.is_type(self.type_id.clone())
            });

        let (_entry_name, entry_choice, _) = func_list.choose(rng).unwrap();

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
                                &generator,
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

    fn mut_borrow_expr<R: Rng>(&self, rng: &mut R) -> Result<BorrowExpr, ()> {
        let filter = |scope_entry: &ScopeEntry, borrow_status: BorrowStatus| -> bool {
            scope_entry.get_type() == self.type_id
                && scope_entry.is_mut()
                && !scope_entry.is_func()
                && (borrow_status == BorrowStatus::None)
        };

        let entries = self.scope.borrow().filter_with_closure(filter);

        let choice = entries.choose(rng);

        match choice {
            Some((entry_name, scope_entry, _)) => {
                let var = Var::new(self.type_id.clone(), entry_name.clone(), false);

                self.scope
                    .borrow_mut()
                    .set_borrow_status(entry_name.clone(), BorrowStatus::MutBorrowed);

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
        let filter = |scope_entry: &ScopeEntry, borrow_status: BorrowStatus| -> bool {
            scope_entry.get_type() == self.type_id
                && !scope_entry.is_func()
                && (borrow_status != BorrowStatus::MutBorrowed)
        };

        let entries = self.scope.borrow().filter_with_closure(filter);
        let choice = entries.choose(rng);

        match choice {
            Some((entry_name, scope_entry, _)) => {
                let var = Var::new(self.type_id.clone(), entry_name.clone(), false);

                self.scope
                    .borrow_mut()
                    .set_borrow_status(entry_name.clone(), BorrowStatus::Borrowed);

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
