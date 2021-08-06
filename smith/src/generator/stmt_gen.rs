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
    context::Context,
    expr_gen::ExprGenerator,
    name_gen::NameGenerator,
    scope_entry::{ScopeEntry, StructScopeEntry},
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
        context: Rc<RefCell<Context>>,
        depth: u32,
        rng: &mut R,
    ) -> BlockStmt {
        context.borrow_mut().enter_scope();

        let mut stmt_list: Vec<Stmt> = Vec::new();

        for _ in 0..MAX_STMTS_IN_BLOCK {
            let stmt = self.stmt(Rc::clone(&context), depth, rng);
            stmt_list.push(stmt);
        }

        context.borrow_mut().leave_scope();

        BlockStmt::new_from_vec(stmt_list)
    }

    pub fn block_stmt_main<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        struct_template: StructTemplate,
        depth: u32,
        rng: &mut R,
    ) -> BlockStmt {
        let mut stmt_list: Vec<Stmt> = Vec::new();

        stmt_list.push(self.global_struct_stmt(struct_template, Rc::clone(&context), rng));

        for _ in 1..MAX_STMTS_IN_BLOCK - 1 {
            let stmt = self.stmt(Rc::clone(&context), depth, rng);
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
        context: Rc<RefCell<Context>>,
        depth: u32,
        rng: &mut R,
        return_type: TypeID,
    ) -> BlockStmt {
        let mut block_stmt = self.block_stmt(Rc::clone(&context), depth, rng);

        if return_type == TypeID::NullType {
            return block_stmt;
        }

        // Create the return statement
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            return_type.clone(),
            BorrowTypeID::None,
        );
        let return_expr = expr_generator.expr(rng);
        context.borrow_mut().reset_expr_depths();

        let return_stmt = ReturnStmt::new(return_type, return_expr);

        block_stmt.push(return_stmt.as_stmt());

        block_stmt
    }

    pub fn stmt<R: Rng>(&mut self, context: Rc<RefCell<Context>>, depth: u32, rng: &mut R) -> Stmt {
        let stmt_select: StmtVariants = rng.gen();

        match stmt_select {
            StmtVariants::AssignStatement if context.borrow().scope.borrow().mut_count() > 0 => {
                self.assign_stmt(context, rng).as_stmt()
            }
            StmtVariants::ConditionalStatement if depth > 0 => {
                self.conditional_stmt(context, depth, rng).as_stmt()
            }
            StmtVariants::LoopStatement if depth > 0 => {
                self.for_loop_stmt(context, depth, rng).as_stmt()
            }
            StmtVariants::LetStatement | _ => self.let_stmt(context, rng).as_stmt(),
        }
    }

    pub fn let_stmt<R: Rng>(&mut self, context: Rc<RefCell<Context>>, rng: &mut R) -> LetStmt {
        let rand_type_id = self.struct_table.rand_type(rng);

        context.borrow_mut().expr_depth = 0;
        // TODO: Allow let statements for mutable and immutable references
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            rand_type_id.clone(),
            BorrowTypeID::None,
        );

        // TODO: make this a better random choice to choose whether mutable variable or not
        let is_mut = rng.gen_bool(0.5);

        // LHS of the let statement
        let var = Var::new(
            rand_type_id.clone(),
            self.var_name_gen.next().unwrap(),
            is_mut,
        );

        context.borrow_mut().enter_scope();
        let expr = expr_generator.expr(rng);
        context.borrow_mut().reset_expr_depths();

        context.borrow_mut().leave_scope();

        let scope_entry: ScopeEntry;

        // Insert struct scope entry, which keeps its own flattened fields in a vec
        if let TypeID::StructType(struct_name) = rand_type_id {
            let struct_scope_entry = StructScopeEntry::new(
                BorrowTypeID::None,
                self.struct_table.get_struct_template(&struct_name).unwrap(),
                self.struct_table,
                is_mut,
            );
            scope_entry = ScopeEntry::Struct(struct_scope_entry);
        } else {
            scope_entry = ScopeEntry::Var(var.clone());
        }

        context
            .borrow()
            .scope
            .borrow_mut()
            .insert(&var.get_name(), scope_entry);
        LetStmt::new(var, expr)
    }

    pub fn for_loop_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        depth: u32,
        rng: &mut R,
    ) -> ExprStmt {
        let rand_int_type: IntTypeID = rand::random();
        let rand_type = rand_int_type.as_type();

        context.borrow_mut().expr_depth = 0;

        let generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            rand_type.clone(),
            BorrowTypeID::None,
        );

        let lower_range = generator.arith_expr(rng);
        context.borrow_mut().reset_expr_depths();

        let upper_range = generator.arith_expr(rng);
        context.borrow_mut().reset_expr_depths();

        let iter_expr = IterRange::new(rand_int_type, lower_range, upper_range).as_iter_expr();

        let var = Var::new(rand_type.clone(), self.var_name_gen.next().unwrap(), false);

        let block_stmt = self.block_stmt(Rc::clone(&context), depth - 1, rng);
        let for_loop_expr = ForLoopExpr::new(rand_type, var, iter_expr, block_stmt);
        ExprStmt::new(for_loop_expr.as_expr())
    }

    pub fn assign_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        rng: &mut R,
    ) -> AssignStmt {
        context
            .borrow_mut()
            .stmt_type
            .push(StmtVariants::AssignStatement);
        let borrower = String::from("temp_mut_borrow");
        let mutables =
            context
                .borrow()
                .scope
                .borrow()
                .filter_with_closure(|scope_entry, borrow_status| {
                    scope_entry.is_mut() || borrow_status == BorrowStatus::MutBorrowed
                });

        let var_choice = match mutables.choose(rng) {
            Some(choice) => choice,
            None => panic!("No mutable variables to assign to"),
        };

        let (var_name, (scope_entry, _)) = var_choice;

        // What happens in the expr, stays in the expr
        context.borrow_mut().enter_scope();

        // Borrow the LHS - which only exists temporarily within this RHS expr scope
        context
            .borrow()
            .scope
            .borrow_mut()
            .mut_borrow_entry(&borrower, &var_name);

        let type_id = scope_entry.get_type();

        // TODO: When we allow references as variables, assignment must have the same ref type
        // For the case of mutable references, we can assign regardless of ref type
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            type_id.clone(),
            BorrowTypeID::None,
        );

        let expr = expr_generator.expr(rng);
        context.borrow_mut().reset_expr_depths();

        context.borrow_mut().stmt_type.pop();

        let left_var = Var::new(scope_entry.get_type(), var_name.clone(), true);

        // We need to dereference if it is a mutable reference, but not if it is a field of a mutref struct
        let deref =
            scope_entry.is_borrow_type(BorrowTypeID::MutRef) && !left_var.get_name().contains('.');

        context.borrow_mut().leave_scope();

        AssignStmt::new(left_var, expr, deref)
    }

    pub fn conditional_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        depth: u32,
        rng: &mut R,
    ) -> ConditionalStmt {
        let mut conditional_blocks: Vec<(BoolExpr, BlockStmt)> = Vec::new();
        context.borrow_mut().expr_depth = 0;

        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            TypeID::BoolType,
            BorrowTypeID::None,
        );

        loop {
            if rng.gen_range(0.0..1.0)
                < conditional_blocks.len() as f32 / MAX_CONDITIONAL_BRANCHES as f32
            {
                break;
            }
            context.borrow_mut().enter_scope();
            let bool_expr = expr_generator.bool_expr(rng);
            context.borrow_mut().reset_expr_depths();

            context.borrow_mut().leave_scope();

            let block_stmt = self.block_stmt(Rc::clone(&context), depth - 1, rng);

            conditional_blocks.push((bool_expr, block_stmt));
        }

        let else_body = if rng.gen::<bool>() {
            let block_stmt = self.block_stmt(Rc::clone(&context), depth - 1, rng);
            Some(block_stmt)
        } else {
            None
        };

        ConditionalStmt::new_from_vec(conditional_blocks, else_body)
    }

    pub fn global_struct_stmt<R: Rng>(
        &self,
        struct_template: StructTemplate,
        context: Rc<RefCell<Context>>,
        rng: &mut R,
    ) -> Stmt {
        context.borrow_mut().expr_depth = 0;

        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            struct_template.get_type(),
            BorrowTypeID::None,
        );

        let expr = expr_generator.global_struct_expr(rng).as_expr();
        context.borrow_mut().reset_expr_depths();

        let scope_entry =
            StructScopeEntry::new(BorrowTypeID::None, struct_template, self.struct_table, true)
                .as_scope_entry();

        let global_struct_type = scope_entry.get_type();

        context
            .borrow()
            .scope
            .borrow_mut()
            .insert(&struct_gen::GLOBAL_STRUCT_VAR_NAME.to_string(), scope_entry);

        let left_var = Var::new(
            global_struct_type,
            struct_gen::GLOBAL_STRUCT_VAR_NAME.to_string(),
            true,
        );

        LetStmt::new(left_var, expr).as_stmt()
    }
}
