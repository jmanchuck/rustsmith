/// Generates statements
use std::{cell::RefCell, rc::Rc};

use crate::rng::{Rng, SliceChoose};

use crate::{
    generator::filters::*,
    program::{
        expr::{
            arithmetic_expr::{ArithmeticExpr, BinaryOp, IntExpr},
            bool_expr::BoolExpr,
            expr::{Expr, RawExpr},
            iter_expr::IterRange,
        },
        stmt::{
            array_assign_stmt::ArrayAssignStmt, assign_stmt::AssignStmt, block_stmt::BlockStmt,
            conditional_stmt::ConditionalStmt, expr_stmt::ExprStmt, for_loop_stmt::ForLoopStmt,
            let_stmt::LetStmt, op_assign_stmt::OpAssignStmt, return_stmt::ReturnStmt, stmt::Stmt,
        },
        struct_template::StructTemplate,
        types::{ArrayTypeID, BorrowTypeID, IntTypeID, TypeID},
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
                self.inject_loop_stopper(&mut stmt_list, for_loop_stmt);
            }
            stmt_list.push(stmt);
        }

        Self::append_checksum_folds(&mut stmt_list);

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
            vec![self.global_struct_stmt(struct_template.clone(), Rc::clone(&context), rng)];

        for _ in 0..consts::MAX_STMTS_IN_BLOCK {
            let mut stmt = self.stmt(Rc::clone(&context), rng);
            if let Stmt::LoopStatement(for_loop_stmt) = &mut stmt {
                self.inject_loop_stopper(&mut stmt_list, for_loop_stmt);
            }
            stmt_list.push(stmt);
        }

        Self::append_checksum_folds(&mut stmt_list);

        // Fold every field of the global struct into the checksum, then print
        // it — the single observable output of the program.
        for (field_name, _) in self.struct_table.flatten_struct_template(&struct_template) {
            let fold = RawExpr::new(format!(
                "cs({}{} as u128)",
                struct_gen::GLOBAL_STRUCT_VAR_NAME,
                field_name
            ))
            .as_expr();
            stmt_list.push(ExprStmt::new(fold).as_stmt());
        }

        let print_stmt = ExprStmt::new(
            RawExpr::new("println!(\"{}\", CHECKSUM.load(Ordering::Relaxed))".to_string())
                .as_expr(),
        )
        .as_stmt();

        stmt_list.push(print_stmt);

        BlockStmt::new_from_vec(stmt_list)
    }

    // Folds every owned int- or bool-typed local in the block into the global
    // checksum at scope exit, so computation feeding any local is observable
    // and cannot be eliminated as dead code (Csmith's trick).
    //
    // Reference-typed lets are skipped: reading them at scope exit would
    // extend their borrow past where non-lexical lifetimes ended it, and the
    // generator emits code that is only legal under that early end (e.g.
    // assigning to the borrowed variable right after the reference's last
    // use). The referenced locals are folded directly instead. Struct locals
    // are skipped because they may have been moved.
    //
    // Array locals are folded element-wise via `.iter()` rather than by
    // index, so the folds cannot mask a bounds-check-elision bug in the
    // guarded index reads/writes being tested.
    fn append_checksum_folds(stmt_list: &mut Vec<Stmt>) {
        let mut folds: Vec<Stmt> = Vec::new();
        for stmt in stmt_list.iter().rev() {
            if let Stmt::LetStatement(let_stmt) = stmt {
                let var = let_stmt.var();
                if var.get_borrow_type() != BorrowTypeID::None {
                    continue;
                }
                let fold = match var.get_type() {
                    TypeID::IntType(_) | TypeID::BoolType => {
                        RawExpr::new(format!("cs({} as u128)", var.get_name()))
                    }
                    TypeID::ArrayType(_) => RawExpr::new(format!(
                        "for e in {}.iter() {{ cs(*e as u128); }}",
                        var.get_name()
                    )),
                    _ => continue,
                };
                folds.push(ExprStmt::new(fold.as_expr()).as_stmt());
            }
        }
        stmt_list.extend(folds);
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
                    Some(self.assign_stmt(context, rng))
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
        let rand_type_id = self.struct_table.rand_type_for_let(rng);
        // Arrays are only ever let-bound by value (phase 1: no array borrows).
        let rand_borrow_type_id: BorrowTypeID = if let TypeID::ArrayType(_) = rand_type_id {
            BorrowTypeID::None
        } else {
            rng.gen()
        };

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
    // before the loop, and appends a break check + increment inside the loop.
    // The break fires either when this loop's own counter exceeds the per-loop
    // cap or when the program-wide fuel supply (see the emitted prelude) is
    // exhausted — the latter bounds total work even when loops multiply
    // through nested function calls.
    fn inject_loop_stopper(&mut self, stmt_list: &mut Vec<Stmt>, loop_stmt: &mut ForLoopStmt) {
        let counter_name = self.var_name_gen.next().unwrap();
        let counter_var = Var::new(IntTypeID::U32.as_type(), counter_name.clone(), true);
        let counter_val = IntExpr::new_u32(0).as_expr();
        let counter_let_stmt = LetStmt::new(counter_var.clone(), counter_val).as_stmt();

        stmt_list.push(counter_let_stmt);

        let break_stmt = ExprStmt::new(
            RawExpr::new(format!(
                "if {} > {}u32 || fuel_exhausted() {{ break; }}",
                counter_name,
                consts::MAX_FOR_LOOP_ITERS
            ))
            .as_expr(),
        )
        .as_stmt();

        let increment_stmt = OpAssignStmt::new(
            counter_var,
            IntExpr::new_u32(1).as_arith_expr(),
            BinaryOp::ADD,
        );

        loop_stmt.push_stmt(break_stmt);
        loop_stmt.push_stmt(increment_stmt.as_stmt());
    }

    pub fn for_loop_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        rng: &mut R,
    ) -> ForLoopStmt {
        let rand_int_type: IntTypeID = rng.gen();
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

    pub fn assign_stmt<R: Rng>(&mut self, context: Rc<RefCell<Context>>, rng: &mut R) -> Stmt {
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

        // Mutable arrays mostly receive guarded element writes; occasionally
        // the whole array is reassigned through the ordinary path below.
        if let TypeID::ArrayType(array_type) = scope_entry.get_type() {
            if rng.gen_range(0u32..4) > 0 {
                let stmt = self.array_element_assign_stmt(
                    Rc::clone(&context),
                    var_name.clone(),
                    array_type,
                    rng,
                );
                context.borrow_mut().leave_scope();
                return stmt;
            }
        }

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

        AssignStmt::new(left_var, expr, deref).as_stmt()
    }

    // Guarded element write to a mutable in-scope array: either the plain
    // form `arr[((idx) as usize) % LEN] = expr;` or one of the op-assign
    // wrapping forms (see ArrayAssignStmt). Called inside the assign
    // statement's temporary scope.
    fn array_element_assign_stmt<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        array_name: String,
        array_type: ArrayTypeID,
        rng: &mut R,
    ) -> Stmt {
        let idx_type: IntTypeID = rng.gen();
        let idx_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            idx_type.as_type(),
            BorrowTypeID::None,
        );
        context.borrow_mut().reset_expr_depth();
        let index: ArithmeticExpr = idx_generator.expr(rng).into();

        let elem_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&context),
            array_type.elem.as_type(),
            BorrowTypeID::None,
        );
        context.borrow_mut().reset_expr_depth();
        let rhs: ArithmeticExpr = elem_generator.expr(rng).into();

        let op = if rng.gen::<bool>() {
            Some(rng.gen::<BinaryOp>())
        } else {
            None
        };

        ArrayAssignStmt::new(array_name, array_type, index, rhs, op).as_stmt()
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
            if rng.gen_range(0.0f32..1.0)
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

        // A &mut variable needs an explicit deref for `x = x.wrapping_add(..)`
        // (auto-deref only applied to the old method-call form), but not a
        // field of a mut-ref struct.
        let deref =
            scope_entry.is_borrow_type(BorrowTypeID::MutRef) && !var_name.contains('.');

        OpAssignStmt::new_with_deref(lhs_var, expr, op, deref)
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::program::expr::arithmetic_expr::IntExpr;
    use crate::program::types::ArrayTypeID;

    #[test]
    fn checksum_folds_arrays_element_wise_via_iter() {
        let int_var = Var::new(IntTypeID::U16.as_type(), "var_0".to_string(), false);
        let int_let = LetStmt::new(int_var, IntExpr::new_u16(3).as_expr()).as_stmt();

        let array_type = ArrayTypeID::new(IntTypeID::I8, 2);
        let array_var = Var::new(array_type.as_type(), "var_1".to_string(), true);
        let array_expr = RawExpr::new("[1i8, 2i8]".to_string()).as_expr();
        let array_let = LetStmt::new(array_var, array_expr).as_stmt();

        let mut stmt_list = vec![int_let, array_let];
        StmtGenerator::append_checksum_folds(&mut stmt_list);

        let rendered: Vec<String> = stmt_list.iter().map(|s| s.to_string()).collect();
        assert_eq!(rendered.len(), 4);
        assert_eq!(rendered[2], "for e in var_1.iter() { cs(*e as u128); };");
        assert_eq!(rendered[3], "cs(var_0 as u128);");
    }
}
