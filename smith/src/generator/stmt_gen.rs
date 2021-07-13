use std::{cell::RefCell, rc::Rc};

use rand::Rng;

use crate::program::{
    stmt::{
        assign_stmt::AssignStmt,
        block_stmt::BlockStmt,
        let_stmt::LetStmt,
        return_stmt::ReturnStmt,
        stmt::{Stmt, StmtVariants},
    },
    struct_template::StructTemplate,
    types::TypeID,
    var::Var,
};

use super::{
    expr_gen::{self, ExprGenerator},
    name_gen::NameGenerator,
    scope::{Scope, ScopeEntry, StructScopeEntry},
    struct_gen::{self, StructTable},
};

const MAX_STMTS_IN_BLOCK: u8 = 12;

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

    pub fn block_stmt<R: Rng>(&mut self, scope: Rc<RefCell<Scope>>, rng: &mut R) -> BlockStmt {
        let mut stmt_list: Vec<Stmt> = Vec::new();

        for _ in 0..MAX_STMTS_IN_BLOCK {
            let result = self.stmt(Rc::clone(&scope), rng);
            match result {
                Ok(stmt) => stmt_list.push(stmt),
                Err(msg) => println!("{}", msg),
            }
        }

        BlockStmt::new_from_vec(stmt_list)
    }

    pub fn block_stmt_with_return<R: Rng>(
        &mut self,
        scope: Rc<RefCell<Scope>>,
        rng: &mut R,
        return_type: TypeID,
    ) -> BlockStmt {
        let mut block_stmt = self.block_stmt(Rc::clone(&scope), rng);

        if return_type == TypeID::NullType {
            return block_stmt;
        }

        // Create the return statement
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            return_type.clone(),
            expr_gen::MAX_EXPR_DEPTH,
        );
        let return_expr = expr_generator.expr(rng);
        let return_stmt = ReturnStmt::new(return_type, return_expr);

        block_stmt.push(return_stmt.as_stmt());

        block_stmt
    }

    pub fn stmt<R: Rng>(&mut self, scope: Rc<RefCell<Scope>>, rng: &mut R) -> Result<Stmt, String> {
        let stmt_select: StmtVariants = rng.gen();

        match stmt_select {
            StmtVariants::AssignStatement if scope.borrow().mut_count() > 0 => {
                let assign_stmt = self.assign_stmt(scope, rng);
                match assign_stmt {
                    Ok(assign_stmt) => Ok(assign_stmt.as_stmt()),
                    Err(s) => Err(s),
                }
            }
            StmtVariants::ConditionalStatement => Ok(self.let_stmt(scope, rng).as_stmt()),
            StmtVariants::LetStatement | _ => Ok(self.let_stmt(scope, rng).as_stmt()),
        }
    }

    pub fn let_stmt<R: Rng>(&mut self, scope: Rc<RefCell<Scope>>, rng: &mut R) -> LetStmt {
        let rand_type_id = self.struct_table.rand_type(rng);

        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            rand_type_id.clone(),
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

        if let TypeID::StructType(struct_name) = rand_type_id {
            let struct_scope_entry = StructScopeEntry::new(
                var.get_name(),
                self.struct_table
                    .get_struct_template(struct_name.clone())
                    .unwrap(),
                self.struct_table.flatten_struct(struct_name),
                is_mut,
            );
            scope_entry = ScopeEntry::Struct(struct_scope_entry);
        } else {
            scope_entry = ScopeEntry::Var(var.clone());
        }

        scope.borrow_mut().add(var.get_name(), Rc::new(scope_entry));
        LetStmt::new(var, expr)
    }

    pub fn assign_stmt<R: Rng>(
        &mut self,
        scope: Rc<RefCell<Scope>>,
        rng: &mut R,
    ) -> Result<AssignStmt, String> {
        let var_choice = scope.borrow().rand_mut(rng);

        let type_id = var_choice.get_type();

        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            type_id.clone(),
            expr_gen::MAX_EXPR_DEPTH,
        );

        let expr = expr_generator.expr(rng);

        if let ScopeEntry::Var(var) = var_choice.as_ref() {
            Ok(AssignStmt::new(var.clone(), expr))
        } else {
            Err(format!(
                "Var choice is not type var, found {}",
                var_choice.get_type().to_string()
            ))
        }
    }

    pub fn static_struct_stmt<R: Rng>(
        &self,
        struct_template: StructTemplate,
        scope: Rc<RefCell<Scope>>,
        rng: &mut R,
    ) -> Stmt {
        let expr_generator = ExprGenerator::new(
            self.struct_table,
            Rc::clone(&scope),
            struct_template.get_type(),
            expr_gen::MAX_EXPR_DEPTH,
        );

        let expr = expr_generator.global_struct_expr(rng).as_expr();

        let var = Var::new(
            struct_template.get_type(),
            struct_gen::GLOBAL_STRUCT_VAR_NAME.to_string(),
            true,
        );

        LetStmt::new(var, expr).as_stmt()
    }
}
