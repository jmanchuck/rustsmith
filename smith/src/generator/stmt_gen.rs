use std::{cell::RefCell, rc::Rc};

use rand::{prelude::SliceRandom, Rng};

use crate::program::{
    expr::{bool_expr::BoolExpr, expr::RawExpr, for_loop_expr::ForLoopExpr, iter_expr::IterRange},
    stmt::{
        assign_stmt::AssignStmt,
        block_stmt::BlockStmt,
        conditional_stmt::ConditionalStmt,
        expr_stmt::ExprStmt,
        let_stmt::LetStmt,
        return_stmt::ReturnStmt,
        stmt::{Stmt, StmtVariants},
    },
    struct_template::StructTemplate,
    types::{BorrowStatus, BorrowTypeID, IntTypeID, TypeID},
    var::Var,
};

use super::{
    expr_gen::{self, ExprGenerator},
    name_gen::NameGenerator,
    scope::{Scope, ScopeEntry, StructScopeEntry},
    struct_gen::{self, StructTable},
};

const MAX_STMTS_IN_BLOCK: u8 = 6;
const MAX_CONDITIONAL_BRANCHES: u8 = 2;
pub const MAX_STMT_DEPTH: u32 = 2; // Only refers to conditional statements

pub struct StmtGenerator<'a> {
    struct_table: &'a StructTable,
    var_name_gen: NameGenerator,
}

impl<'a> StmtGenerator<'a> {
    pub fn new(struct_table: &'a StructTable) -> Self {
        StmtGenerator {
            struct_table,
            var_name_gen: NameGenerator::new(String::from("var_")),
        }
    }

    pub fn block_stmt<R: Rng>(
        &mut self,
        scope: Rc<RefCell<Scope>>,
        depth: u32,
        rng: &mut R,
    ) -> BlockStmt {
        let mut stmt_list: Vec<Stmt> = Vec::new();

        for _ in 0..MAX_STMTS_IN_BLOCK {
            let stmt = self.stmt(Rc::clone(&scope), depth, rng);
            stmt_list.push(stmt);
        }

        BlockStmt::new_from_vec(stmt_list)
    }

    pub fn block_stmt_main<R: Rng>(
        &mut self,
        scope: Rc<RefCell<Scope>>,
        struct_template: StructTemplate,
        depth: u32,
        rng: &mut R,
    ) -> BlockStmt {
        let mut stmt_list: Vec<Stmt> = Vec::new();

        stmt_list.push(self.global_struct_stmt(struct_template, Rc::clone(&scope), rng));

        for _ in 1..MAX_STMTS_IN_BLOCK - 1 {
            let stmt = self.stmt(Rc::clone(&scope), depth, rng);
            stmt_list.push(stmt);
        }

        let print_serialized = RawExpr::new(format!(
            "println!(\"{{}}\", (serde_json::to_string(&{}).unwrap()))",
            struct_gen::GLOBAL_STRUCT_VAR_NAME
        ))
        .as_expr();

        let print_stmt = ExprStmt::new(print_serialized).as_stmt();

        stmt_list.push(print_stmt);

        BlockStmt::new_from_vec(stmt_list)
    }

    pub fn block_stmt_with_return<R: Rng>(
        &mut self,
        scope: Rc<RefCell<Scope>>,
        depth: u32,
        rng: &mut R,
        return_type: TypeID,
    ) -> BlockStmt {
        let mut block_stmt = self.block_stmt(Rc::clone(&scope), depth, rng);

        if return_type == TypeID::NullType {
            return block_stmt;
        }

        // Create the return statement
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            return_type.clone(),
            BorrowTypeID::None,
            expr_gen::MAX_EXPR_DEPTH,
        );
        let return_expr = expr_generator.expr(rng);
        let return_stmt = ReturnStmt::new(return_type, return_expr);

        block_stmt.push(return_stmt.as_stmt());

        block_stmt
    }

    pub fn stmt<R: Rng>(&mut self, scope: Rc<RefCell<Scope>>, depth: u32, rng: &mut R) -> Stmt {
        let stmt_select: StmtVariants = rng.gen();

        match stmt_select {
            StmtVariants::AssignStatement if scope.borrow().mut_count() > 0 => {
                self.assign_stmt(scope, rng).as_stmt()
            }
            StmtVariants::ConditionalStatement if depth > 0 => {
                self.conditional_stmt(scope, depth, rng).as_stmt()
            }
            StmtVariants::LoopStatement if depth > 0 => {
                self.for_loop_stmt(scope, depth, rng).as_stmt()
            }
            StmtVariants::LetStatement | _ => self.let_stmt(scope, rng).as_stmt(),
        }
    }

    pub fn let_stmt<R: Rng>(&mut self, scope: Rc<RefCell<Scope>>, rng: &mut R) -> LetStmt {
        let rand_type_id = self.struct_table.rand_type(rng);

        // TODO: Allow let statements for mutable and immutable references
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            rand_type_id.clone(),
            BorrowTypeID::None,
            expr_gen::MAX_EXPR_DEPTH,
        );

        // TODO: make this a better random choice to choose whether mutable variable or not
        let is_mut = rng.gen_bool(0.5);

        let var = Var::new(
            rand_type_id.clone(),
            self.var_name_gen.next().unwrap(),
            is_mut,
        );

        let expr = expr_generator.expr(rng);

        let scope_entry: ScopeEntry;

        // Insert struct scope entry, which keeps its own flattened fields in a vec
        if let TypeID::StructType(struct_name) = rand_type_id {
            let struct_scope_entry = StructScopeEntry::new(
                var.get_name(),
                BorrowTypeID::None,
                self.struct_table.get_struct_template(&struct_name).unwrap(),
                self.struct_table.flatten_struct(&struct_name),
                is_mut,
            );
            scope_entry = ScopeEntry::Struct(struct_scope_entry);
        } else {
            scope_entry = ScopeEntry::Var(var.clone());
        }

        scope
            .borrow_mut()
            .insert(&var.get_name(), Rc::new(scope_entry));
        LetStmt::new(var, expr)
    }

    pub fn for_loop_stmt<R: Rng>(
        &mut self,
        scope: Rc<RefCell<Scope>>,
        depth: u32,
        rng: &mut R,
    ) -> ExprStmt {
        let rand_int_type: IntTypeID = rand::random();
        let rand_type = rand_int_type.as_type();

        let generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            rand_type.clone(),
            BorrowTypeID::None,
            expr_gen::MAX_EXPR_DEPTH,
        );

        let lower_range = generator.arith_expr(expr_gen::MAX_EXPR_DEPTH, rng);
        let upper_range = generator.arith_expr(expr_gen::MAX_EXPR_DEPTH, rng);
        let iter_expr = IterRange::new(rand_int_type, lower_range, upper_range).as_iter_expr();

        let var = Var::new(rand_type.clone(), self.var_name_gen.next().unwrap(), false);

        let child_scope = Rc::new(RefCell::new(Scope::new_from_parent(scope)));
        let block_stmt = self.block_stmt(child_scope, depth - 1, rng);
        let for_loop_expr = ForLoopExpr::new(rand_type, var, iter_expr, block_stmt);

        ExprStmt::new(for_loop_expr.as_expr())
    }

    // TODO: Use the borrow context struct to manage borrows
    pub fn assign_stmt<R: Rng>(&mut self, scope: Rc<RefCell<Scope>>, rng: &mut R) -> AssignStmt {
        // TODO: If this LHS is a field of a struct, then the entire struct should be considered borrowed
        // Had the issue of: let mut a = struct -> a.field = function(a, other_args), a cannot be function arg
        let borrower = String::from("temp_mut_borrow");
        let mutables = scope
            .borrow()
            .filter_with_closure(|scope_entry, borrow_status| {
                scope_entry.is_mut() || borrow_status == BorrowStatus::MutBorrowed
            });

        let var_choice = match mutables.choose(rng) {
            Some(choice) => choice,
            None => panic!("No mutable variables to assign to"),
        };

        let (var_name, scope_entry, _) = var_choice;

        // TODO: Prevent a mut borrow override using context variable
        scope.borrow_mut().mut_borrow_entry(&borrower, &var_name);

        let type_id = scope_entry.get_type();

        // TODO: When we allow references as variables, assignment must have the same ref type
        // For the case of mutable references, we can assign regardless of ref type
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            type_id.clone(),
            BorrowTypeID::None,
            expr_gen::MAX_EXPR_DEPTH,
        );

        let expr = expr_generator.expr(rng);

        // Return borrow status to previous state (since RHS expression is self contained)
        scope.borrow_mut().remove_entry(&borrower);

        let left_var = Var::new(scope_entry.get_type(), var_name.clone(), true);

        // If we are assigning directly onto a mutable reference, we need to dereference it
        AssignStmt::new(
            left_var,
            expr,
            scope_entry.is_borrow_type(BorrowTypeID::MutRef),
        )
    }

    pub fn conditional_stmt<R: Rng>(
        &mut self,
        scope: Rc<RefCell<Scope>>,
        depth: u32,
        rng: &mut R,
    ) -> ConditionalStmt {
        let mut conditional_blocks: Vec<(BoolExpr, BlockStmt)> = Vec::new();

        // TODO: Think about what could be the borrow type here
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            TypeID::BoolType,
            BorrowTypeID::None,
            expr_gen::MAX_EXPR_DEPTH,
        );

        loop {
            if rng.gen_range(0.0..1.0)
                < conditional_blocks.len() as f32 / MAX_CONDITIONAL_BRANCHES as f32
            {
                break;
            }

            let bool_expr = expr_generator.bool_expr(expr_gen::MAX_EXPR_DEPTH, rng);

            let block_scope = Rc::new(RefCell::new(Scope::new_from_parent(Rc::clone(&scope))));
            let block_stmt = self.block_stmt(Rc::clone(&block_scope), depth - 1, rng);

            conditional_blocks.push((bool_expr, block_stmt));
        }

        let else_body = if rng.gen::<bool>() {
            let block_scope = Rc::new(RefCell::new(Scope::new_from_parent(Rc::clone(&scope))));
            Some(self.block_stmt(block_scope, depth - 1, rng))
        } else {
            None
        };

        ConditionalStmt::new_from_vec(conditional_blocks, else_body)
    }

    pub fn global_struct_stmt<R: Rng>(
        &self,
        struct_template: StructTemplate,
        scope: Rc<RefCell<Scope>>,
        rng: &mut R,
    ) -> Stmt {
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            struct_template.get_type(),
            BorrowTypeID::None,
            expr_gen::MAX_EXPR_DEPTH,
        );

        let expr = expr_generator.global_struct_expr(rng).as_expr();

        let flattened_fields = self.struct_table.flatten_struct_template(&struct_template);

        let scope_entry = StructScopeEntry::new(
            struct_gen::GLOBAL_STRUCT_VAR_NAME.to_string(),
            BorrowTypeID::None,
            struct_template,
            flattened_fields,
            true,
        )
        .as_scope_entry();

        let global_struct_type = scope_entry.get_type();

        scope.borrow_mut().insert(
            &struct_gen::GLOBAL_STRUCT_VAR_NAME.to_string(),
            Rc::new(scope_entry),
        );

        let left_var = Var::new(
            global_struct_type,
            struct_gen::GLOBAL_STRUCT_VAR_NAME.to_string(),
            true,
        );

        LetStmt::new(left_var, expr).as_stmt()
    }
}
