mod borrow_scope;
pub mod context;
mod expr_gen;
pub mod filters;
mod func_gen;
pub mod main_gen;
mod name_gen;
pub mod scope;
pub mod scope_entry;
mod static_gen;
mod stmt_gen;
pub mod struct_gen;
pub mod weights;

mod consts;

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::program::types::TypeID;

    use super::scope::*;
    use super::scope_entry::*;
    #[test]
    fn test_panic_for_borrow_mut_scope_parent() {
        let parent_scope = Rc::new(RefCell::new(Scope::new()));

        let var = VarScopeEntry::new(TypeID::NullType, "a".to_string(), false).as_scope_entry();

        parent_scope.borrow_mut().insert(&"a".to_string(), var);

        let child_scope = Rc::new(RefCell::new(Scope::new_from_parent(parent_scope.clone())));

        child_scope.borrow_mut().remove_entry(&"a".to_string());
    }
}
