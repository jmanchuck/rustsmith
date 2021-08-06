use std::{cell::RefCell, rc::Rc};

use super::super::program::stmt::stmt::StmtVariants;
use super::scope::Scope;
pub struct Context {
    pub scope: Rc<RefCell<Scope>>,
    pub loop_depth: usize,
    pub expr_depth: usize, // For any nested expression
    pub arith_expr_depth: usize,
    pub bool_expr_depth: usize,
    pub if_depth: usize,
    pub stmt_type: Vec<StmtVariants>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            ..Default::default()
        }
    }
    pub fn enter_scope(&mut self) {
        let new_scope = Rc::new(RefCell::new(Scope::new_from_parent(Rc::clone(&self.scope))));

        self.scope = new_scope;
    }

    pub fn leave_scope(&mut self) {
        let parent = match &self.scope.borrow().get_parent() {
            Some(parent) => Rc::clone(parent),
            _ => panic!("No parent scope found"),
        };

        self.scope = parent;
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            scope: Rc::new(RefCell::new(Scope::new())),
            loop_depth: 0,
            expr_depth: 0,
            if_depth: 0,
            arith_expr_depth: 0,
            bool_expr_depth: 0,
            stmt_type: Vec::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{generator::scope_entry::VarScopeEntry, program::types::TypeID};

    use super::*;
    #[test]
    fn leave_scope_deletes_previous_scope() {
        let mut context = Context::new();

        let var = VarScopeEntry::new(TypeID::BoolType, "a".to_string(), false).as_scope_entry();
        context.scope.borrow_mut().insert(&"a".to_string(), var);

        assert_eq!(context.scope.borrow().get_all_entries().len(), 1);

        context.enter_scope();
        let var_inner =
            VarScopeEntry::new(TypeID::BoolType, "b".to_string(), false).as_scope_entry();
        context
            .scope
            .borrow_mut()
            .insert(&"b".to_string(), var_inner);

        assert_eq!(context.scope.borrow().get_all_entries().len(), 2);

        context.leave_scope();

        assert_eq!(context.scope.borrow().get_all_entries().len(), 1);
    }
}
