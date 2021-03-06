/// Generates statements
use std::{cell::RefCell, rc::Rc};

use rand::{prelude::SliceRandom, Rng};

use crate::{
    generator::filters::*,
    program::{
        expr::{
            arithmetic_expr::{ArithmeticExpr, BinaryOp, IntExpr},
            bool_expr::{BoolExpr, ComparisonExpr, ComparisonOp},
            expr::{Expr, RawExpr},
            iter_expr::IterRange,
        },
        stmt::{
            assign_stmt::AssignStmt, block_stmt::BlockStmt, conditional_stmt::ConditionalStmt,
            expr_stmt::ExprStmt, for_loop_stmt::ForLoopStmt, let_stmt::LetStmt,
            op_assign_stmt::OpAssignStmt, return_stmt::ReturnStmt, stmt::Stmt,
        },
        struct_template::StructTemplate,
        types::{BorrowTypeID, IntTypeID, TypeID},
        var::Var,
    },
};

use super::{
    consts,
    context::Context,
    expr_gen::ExprGenerator,
    name_gen::NameGenerator,
    scope_entry::{ScopeEntry, StructScopeEntry},
    struct_gen::{self, StructTable},
    weights::stmt::variants::StmtVariants,
};

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

    pub fn block_stmt<R: Rng>(&mut self, context: Rc<RefCell<Context>>, rng: &mut R) -> BlockStmt {
        context.borrow_mut().enter_scope();

        let mut stmt_list: Vec<Stmt> = Vec::new();

        for _ in 0..consts::MAX_STMTS_IN_BLOCK {
            let mut stmt = self.stmt(Rc::clone(&context), rng);
            if let Stmt::LoopStatement(for_loop_stmt) = &mut stmt {
                if rng.gen::<f32>() < consts::PROB_MAX_FOR_LOOP_ITERS {
                    self.inject_loop_stopper(&mut stmt_list, for_loop_stmt);
                }
            }
            stmt_list.push(stmt);
        }

        context.borrow_mut().leave_scope();

        BlockStmt::new_from_vec(stmt_list)
    }

    pub fn block_stmt_main<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        struct_template: StructTemplate,
        rng: &mut R,
    ) -> BlockStmt {
        let mut stmt_list: Vec<Stmt> =
            vec![self.global_struct_stmt(struct_template, Rc::clone(&context), rng)];

        for _ in 0..consts::MAX_STMTS_IN_BLOCK {
            let mut stmt = self.stmt(Rc::clone(&context), rng);
            if let Stmt::LoopStatement(for_loop_stmt) = &mut stmt {
                if rng.gen::<f32>() < consts::PROB_MAX_FOR_LOOP_ITERS {
                    self.inject_loop_stopper(&mut stmt_list, for_loop_stmt);
                }
            }
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
        rng: &mut R,
        return_type: TypeID,
    ) -> BlockStmt {
        let mut block_stmt = self.block_stmt(Rc::clone(&context), rng);

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

        context.borrow_mut().reset_expr_depth();

        let return_expr = expr_generator.expr(rng);

        let return_stmt = ReturnStmt::new(return_type, return_expr);

        block_stmt.push(return_stmt.as_stmt());

        block_stmt
    }

    fn try_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        stmt_type: StmtVariants,
        rng: &mut R,
    ) -> Option<Stmt> {
        match stmt_type {
            StmtVariants::LetStatement => Some(self.let_stmt(context, rng).as_stmt()),
            StmtVariants::ConditionalStatement => {
                if context.borrow().if_depth < consts::MAX_CONDITIONAL_DEPTH {
                    Some(self.conditional_stmt(context, rng).as_stmt())
                } else {
                    None
                }
            }
            StmtVariants::AssignStatement => {
                if context.borrow().scope.borrow().mut_count() > 0 {
                    Some(self.assign_stmt(context, rng).as_stmt())
                } else {
                    None
                }
            }
            StmtVariants::OpAssignStatement => {
                let filters = Filters::new()
                    .with_filters(vec![is_mut_or_mut_ref_filter(), is_int_type_filter()]);
                if !filters.filter(&context.borrow().scope).is_empty() {
                    Some(self.op_assign_stmt(context, rng).as_stmt())
                } else {
                    None
                }
            }
            StmtVariants::LoopStatement => {
                if context.borrow().loop_depth < consts::MAX_LOOP_DEPTH {
                    Some(self.for_loop_stmt(context, rng).as_stmt())
                } else {
                    None
                }
            }
            StmtVariants::FuncCallStatement => {
                if context
                    .borrow()
                    .scope
                    .borrow()
                    .contains_filter(is_func_filter())
                {
                    Some(self.func_call_stmt(context, rng).as_stmt())
                } else {
                    None
                }
            }
        }
    }

    pub fn stmt<R: Rng>(&mut self, context: Rc<RefCell<Context>>, rng: &mut R) -> Stmt {
        let mut stmt_select: StmtVariants = rng.gen();

        let loop_limit = 100;
        for _ in 0..loop_limit {
            if let Some(stmt) = self.try_stmt(Rc::clone(&context), stmt_select, rng) {
                return stmt;
            } else {
                stmt_select = rng.gen();
            }
        }

        panic!("Could not generate stmt");
    }

    pub fn let_stmt<R: Rng>(&mut self, context: Rc<RefCell<Context>>, rng: &mut R) -> LetStmt {
        let rand_type_id = self.struct_table.rand_type(rng);
        let rand_borrow_type_id: BorrowTypeID = rng.gen();

        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            rand_type_id.clone(),
            rand_borrow_type_id,
        );

        // TODO: make this a better random choice to choose whether mutable variable or not
        let is_mut = rng.gen_bool(0.5) && rand_borrow_type_id == BorrowTypeID::None;

        // LHS of the let statement
        let var = Var::new_with_borrow(
            rand_type_id.clone(),
            rand_borrow_type_id,
            self.var_name_gen.next().unwrap(),
            is_mut,
        );

        context.borrow_mut().enter_scope();
        context.borrow_mut().reset_expr_depth();
        let expr = match rand_borrow_type_id {
            BorrowTypeID::None => expr_generator.expr(rng),
            _ => expr_generator.borrow_expr(rng).as_expr(),
        };

        context.borrow_mut().leave_scope();

        let scope_entry: ScopeEntry;

        // Insert struct scope entry, which keeps its own flattened fields in a vec
        if let TypeID::StructType(struct_name) = rand_type_id {
            let struct_scope_entry = StructScopeEntry::new(
                rand_borrow_type_id,
                self.struct_table.get_struct_template(&struct_name).unwrap(),
                self.struct_table,
                is_mut,
            );
            scope_entry = ScopeEntry::Struct(struct_scope_entry);
        } else {
            scope_entry = ScopeEntry::Var(var.clone());
        }

        if rand_borrow_type_id == BorrowTypeID::None {
            context
                .borrow()
                .scope
                .borrow_mut()
                .insert(&var.get_name(), scope_entry);
        } else if let Expr::Variable(_) = expr {
            if rand_borrow_type_id == BorrowTypeID::Ref {
                context.borrow().scope.borrow_mut().insert_borrow(
                    &var.get_name(),
                    scope_entry,
                    &expr.to_string(),
                );
            } else if rand_borrow_type_id == BorrowTypeID::MutRef {
                context.borrow().scope.borrow_mut().insert_mut_borrow(
                    &var.get_name(),
                    scope_entry,
                    &expr.to_string(),
                );
            }
        }

        LetStmt::new(var, expr)
    }

    // Takes the stmt list being generated, inserts an initialiser variable
    // Insert a conditional statement into loop to check if initialiser variable > max
    // Insert an increment statement into loop at the end
    fn inject_loop_stopper(&mut self, stmt_list: &mut Vec<Stmt>, loop_stmt: &mut ForLoopStmt) {
        let counter_name = self.var_name_gen.next().unwrap();
        let counter_var = Var::new(IntTypeID::U32.as_type(), counter_name, true);
        let counter_val = IntExpr::new_u32(0).as_expr();
        let counter_let_stmt = LetStmt::new(counter_var.clone(), counter_val).as_stmt();

        stmt_list.push(counter_let_stmt);

        let comparison_val = IntExpr::new_u32(consts::MAX_FOR_LOOP_ITERS);
        let comparison_expr = ComparisonExpr::new(
            counter_var.clone().into(),
            comparison_val.as_arith_expr(),
            ComparisonOp::Greater,
        )
        .as_bool_expr();
        let break_block = BlockStmt::new_from_vec(vec![ExprStmt::new(
            RawExpr::new("break".to_string()).as_expr(),
        )
        .as_stmt()]);
        let mut loop_break_stmt = ConditionalStmt::new();
        loop_break_stmt.insert_conditional(comparison_expr, break_block);

        let increment_stmt = OpAssignStmt::new(
            counter_var,
            IntExpr::new_u32(1).as_arith_expr(),
            BinaryOp::ADD,
        );

        loop_stmt.push_stmt(loop_break_stmt.as_stmt());
        loop_stmt.push_stmt(increment_stmt.as_stmt());
    }

    pub fn for_loop_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        rng: &mut R,
    ) -> ForLoopStmt {
        let rand_int_type: IntTypeID = rand::random();
        let rand_type = rand_int_type.as_type();

        context.borrow_mut().loop_depth += 1;

        let generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            rand_type.clone(),
            BorrowTypeID::None,
        );

        context.borrow_mut().reset_expr_depth();
        let lower_range: ArithmeticExpr = generator.expr(rng).into();

        context.borrow_mut().reset_expr_depth();
        let upper_range: ArithmeticExpr = generator.expr(rng).into();

        let iter_expr = IterRange::new(rand_int_type, lower_range, upper_range).as_iter_expr();

        let var = Var::new(rand_type.clone(), self.var_name_gen.next().unwrap(), false);

        let block_stmt = self.block_stmt(Rc::clone(&context), rng);

        let for_loop_stmt = ForLoopStmt::new(rand_type, var, iter_expr, block_stmt);
        context.borrow_mut().loop_depth -= 1;

        for_loop_stmt
    }

    pub fn assign_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        rng: &mut R,
    ) -> AssignStmt {
        let mut_filter = Filters::new().with_filters(vec![is_mut_or_mut_ref_filter()]);
        let mutables = mut_filter.filter(&context.borrow().scope);

        let var_choice = match mutables.choose(rng) {
            Some(choice) => choice,
            None => panic!("No mutable variables to assign to"),
        };

        let (var_name, (scope_entry, _)) = var_choice;

        // What happens in the expr, stays in the expr
        context.borrow_mut().enter_scope();

        context
            .borrow()
            .scope
            .borrow_mut()
            .func_mut_borrow(var_name);

        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            scope_entry.get_type(),
            BorrowTypeID::None,
        );

        context.borrow_mut().reset_expr_depth();

        let expr = expr_generator.expr(rng);

        context.borrow_mut().leave_scope();
        if let Expr::Variable(_) = expr {
            if scope_entry.is_borrow_type(BorrowTypeID::MutRef) {
                context.borrow().scope.borrow_mut().use_mut_borrow(var_name);
            }
        }

        let left_var = Var::new(scope_entry.get_type(), var_name.clone(), true);

        // We need to dereference if it is a mutable reference, but not if it is a field of a mutref struct
        let deref =
            scope_entry.is_borrow_type(BorrowTypeID::MutRef) && !left_var.get_name().contains('.');

        AssignStmt::new(left_var, expr, deref)
    }

    pub fn conditional_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        rng: &mut R,
    ) -> ConditionalStmt {
        let mut conditional_blocks: Vec<(BoolExpr, BlockStmt)> = Vec::new();
        context.borrow_mut().if_depth += 1;

        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            TypeID::BoolType,
            BorrowTypeID::None,
        );

        loop {
            if rng.gen_range(0.0..1.0)
                < conditional_blocks.len() as f32 / consts::MAX_CONDITIONAL_BRANCHES as f32
            {
                break;
            }
            context.borrow_mut().enter_scope();
            context.borrow_mut().reset_expr_depth();
            let bool_expr: BoolExpr = expr_generator.expr(rng).into();

            context.borrow_mut().leave_scope();

            let block_stmt = self.block_stmt(Rc::clone(&context), rng);

            conditional_blocks.push((bool_expr, block_stmt));
        }

        let else_body = if rng.gen::<bool>() {
            let block_stmt = self.block_stmt(Rc::clone(&context), rng);
            Some(block_stmt)
        } else {
            None
        };
        context.borrow_mut().if_depth -= 1;

        ConditionalStmt::new_from_vec(conditional_blocks, else_body)
    }

    pub fn op_assign_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        rng: &mut R,
    ) -> OpAssignStmt {
        let filters =
            Filters::new().with_filters(vec![is_mut_or_mut_ref_filter(), is_int_type_filter()]);

        let var_list = filters.filter(&context.borrow().scope);
        let (var_name, (scope_entry, _)) = var_list.choose(rng).unwrap();
        let type_id = scope_entry.get_type();

        // What happens in the expr, stays in the expr
        context.borrow_mut().enter_scope();

        context
            .borrow()
            .scope
            .borrow_mut()
            .func_mut_borrow(var_name);

        let generator = ExprGenerator::new(
            &self.struct_table,
            context.clone(),
            type_id.clone(),
            BorrowTypeID::None,
        );

        context.borrow_mut().reset_expr_depth();
        let expr = generator.expr(rng).into();

        context.borrow_mut().leave_scope();

        let op = rng.gen();
        let lhs_var = Var::new(type_id, var_name.clone(), false);

        OpAssignStmt::new(lhs_var, expr, op)
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
        context.borrow_mut().reset_expr_depth();

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

    pub fn func_call_stmt<R: Rng>(&self, context: Rc<RefCell<Context>>, rng: &mut R) -> ExprStmt {
        let filters = Filters::new().with_filters(vec![is_func_filter()]);
        let func_list = filters.filter(&context.borrow().scope);
        let choice = func_list.choose(rng).unwrap();

        let (_, (entry, _)) = choice;

        let func_entry;

        if let ScopeEntry::Func(f_entry) = entry.as_ref() {
            func_entry = f_entry;
        } else {
            panic!("Filter did not return func entry");
        }

        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            TypeID::NullType,
            BorrowTypeID::None,
        );

        let func_call_expr =
            expr_generator.func_call_expr_from_template(func_entry.get_template(), rng);

        ExprStmt::new(func_call_expr.as_expr())
    }
}
