use std::{cell::RefCell, rc::Rc};

use crate::program::{
    expr::{
        binary_expr::{BinaryExpr, BinaryOp},
        expr::{ArithmeticExpr, ArithmeticExprVariants, Expr},
        func_call_expr::FunctionCallExpr,
        int_expr::{IntExpr, IntValue},
        struct_expr::{StructExpr, StructExprVariants, StructLiteral},
    },
    struct_template::StructTemplate,
    types::TypeID,
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
    depth: u32,
}

impl<'a> ExprGenerator<'a> {
    pub fn new(
        struct_table: &'a StructTable,
        scope: Rc<RefCell<Scope>>,
        type_id: TypeID,
        depth: u32,
    ) -> Self {
        ExprGenerator {
            struct_table,
            scope,
            type_id,
            depth,
        }
    }

    pub fn new_from(other: &'a ExprGenerator, type_id: TypeID) -> Self {
        ExprGenerator {
            struct_table: &other.struct_table,
            scope: Rc::clone(&other.scope),
            type_id,
            depth: other.depth,
        }
    }

    pub fn expr<R: Rng>(&self, rng: &mut R) -> Expr {
        match &self.type_id {
            TypeID::IntType(_int_type_id) => self.arith_expr(self.depth, rng).as_expr(),
            TypeID::StructType(struct_name) => self.struct_expr(struct_name.clone(), rng).as_expr(),
            TypeID::NullType => panic!("Tried to construct an expression of null type"),
        }
    }

    // pub fn expr_with_ref_qualifier<R: Rng>(
    //     &self,
    //     ref_qualifier: RefTypeID,
    //     rng: &mut R,
    // ) -> Result<Expr, String> {
    //     let var_filter_by_mut = |scope_entry: ScopeEntry| match scope_entry {
    //         ScopeEntry::Var(var) => var.get_borrow_type() == RefTypeID::MutRef,
    //         ScopeEntry::Struct(_) => todo!(),
    //         _ => false,
    //     };

    //     match ref_qualifier {
    //         RefTypeID::None => Ok(self.expr(rng)),
    //         RefTypeID::Ref => Ok(self.expr(rng)),
    //         RefTypeID::MutRef => if self.scope.borrow().contains_function_type(self.type_id) {},
    //     }
    // }

    pub fn global_struct_expr<R: Rng>(&self, rng: &mut R) -> StructExpr {
        let struct_template = self.struct_table.get_global_struct().unwrap();

        StructExpr::Literal(self.struct_literal(struct_template, rng))
    }

    pub fn struct_expr<R: Rng>(&self, struct_name: String, rng: &mut R) -> StructExpr {
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
            StructExprVariants::Var
                if self.scope.borrow().contains_var_type(self.type_id.clone()) =>
            {
                let var = self.var(rng);
                // For struct expression, using the expression is equivalent to a move

                // TODO is the move here removing the correct entry? hmmm
                self.scope.borrow_mut().remove_entry(var.get_name());

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
            let mut generator = ExprGenerator::new_from(self, field_type_id.clone());
            generator.depth = if self.depth > 0 {
                generator.depth - 1
            } else {
                0
            };
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

        match expr_choice {
            ArithmeticExprVariants::Binary if depth > 0 => {
                self.binary_int_expr(depth, rng).as_arith_expr()
            }
            ArithmeticExprVariants::Var
                if self.scope.borrow().contains_var_type(self.type_id.clone()) =>
            {
                ArithmeticExpr::Var(self.var(rng))
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

    // Assumes that the function with the correct type already exists
    fn func_call_expr<R: Rng>(&self, rng: &mut R) -> Result<FunctionCallExpr, String> {
        let var_list: Vec<(String, Rc<ScopeEntry>)> =
            self.scope.borrow().filter_by_type(self.type_id.clone());

        let func_list: Vec<(String, Rc<ScopeEntry>)> =
            var_list.into_iter().filter(|x| x.1.is_func()).collect();

        let entry_choice = func_list.choose(rng).unwrap();

        if let ScopeEntry::Func(func_scope_entry) = entry_choice.1.as_ref() {
            let function_template = func_scope_entry.get_template();
            let mut arguments: Vec<Expr> = Vec::new();
            for param in function_template.params_iter() {
                let generator = ExprGenerator::new_from(self, param.get_type());

                // TODO use with borrow type
                // let expr = generator.expr_with_ref_qualifier(rng);
                let expr = generator.expr(rng);
                arguments.push(expr);
            }

            Ok(FunctionCallExpr::new(function_template, arguments))
        } else {
            Err(format!(
                "Could not find function with return type {}",
                self.type_id.to_string()
            ))
        }
    }

    fn var<R: Rng>(&self, rng: &mut R) -> Var {
        let var_list: Vec<(String, Rc<ScopeEntry>)> =
            self.scope.borrow().filter_by_type(self.type_id.clone());

        let var_list: Vec<(String, Rc<ScopeEntry>)> =
            var_list.into_iter().filter(|x| x.1.is_var()).collect();

        let var_choice = var_list.choose(rng).unwrap();

        Var::new(self.type_id.clone(), var_choice.0.clone(), false)
    }

    pub fn int32<R: Rng>(rng: &mut R) -> IntExpr {
        IntExpr::new_i32(rng.gen::<i32>())
    }
}
