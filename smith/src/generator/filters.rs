use std::{cell::RefCell, rc::Rc};

use crate::program::types::{BorrowStatus, BorrowTypeID, TypeID};

use super::{scope::Scope, scope_entry::ScopeEntry};

type ScopeBorrowClosure = Box<dyn Fn(Rc<ScopeEntry>, BorrowStatus) -> bool>;
type NameScopeBorrowClosure = Box<dyn Fn(&String, Rc<ScopeEntry>, BorrowStatus) -> bool>;

pub fn is_func_filter() -> ScopeBorrowClosure {
    Box::new(|scope_entry, _| scope_entry.is_func())
}

pub fn is_var_filter() -> ScopeBorrowClosure {
    Box::new(|scope_entry, _| scope_entry.is_var())
}

pub fn is_struct_filter() -> ScopeBorrowClosure {
    Box::new(|scope_entry, _| scope_entry.is_struct())
}

pub fn is_type_filter(type_id: TypeID) -> ScopeBorrowClosure {
    Box::new(move |scope_entry, _| scope_entry.is_type(type_id.clone()))
}

pub fn is_int_type_filter() -> ScopeBorrowClosure {
    Box::new(move |scope_entry, _| match scope_entry.get_type() {
        TypeID::IntType(_) => true,
        _ => false,
    })
}

pub fn is_borrow_type_filter(borrow_type_id: BorrowTypeID) -> ScopeBorrowClosure {
    Box::new(move |scope_entry, _| scope_entry.is_borrow_type(borrow_type_id.clone()))
}

pub fn is_not_borrow_type_filter(borrow_type_id: BorrowTypeID) -> ScopeBorrowClosure {
    Box::new(move |scope_entry, _| !scope_entry.is_borrow_type(borrow_type_id.clone()))
}

pub fn is_borrowed_filter() -> ScopeBorrowClosure {
    Box::new(|_, borrow_status| borrow_status == BorrowStatus::Borrowed)
}

pub fn is_mut_borrowed_filter() -> ScopeBorrowClosure {
    Box::new(|_, borrow_status| borrow_status == BorrowStatus::MutBorrowed)
}

pub fn is_not_mut_borrowed_filter() -> ScopeBorrowClosure {
    Box::new(|_, borrow_status| borrow_status != BorrowStatus::MutBorrowed)
}

pub fn is_not_borrowed_filter() -> ScopeBorrowClosure {
    Box::new(|_, borrow_status| borrow_status == BorrowStatus::None)
}

pub fn is_mut_or_mut_ref_filter() -> ScopeBorrowClosure {
    Box::new(|scope_entry, _| {
        scope_entry.is_mut() || scope_entry.is_borrow_type(BorrowTypeID::MutRef)
    })
}

pub fn can_move_filter(scope: Rc<RefCell<Scope>>) -> NameScopeBorrowClosure {
    Box::new(move |entry_name: &String, _, _| -> bool { scope.borrow().can_move_entry(entry_name) })
}

pub struct Filters {
    filters: Vec<ScopeBorrowClosure>,
    full_filters: Vec<NameScopeBorrowClosure>,
}

impl Filters {
    pub fn new() -> Self {
        Filters {
            filters: Vec::new(),
            full_filters: Vec::new(),
        }
    }

    pub fn with_filters(mut self, filters: Vec<ScopeBorrowClosure>) -> Self {
        self.filters = filters;
        self
    }

    pub fn with_full_filters(mut self, filters: Vec<NameScopeBorrowClosure>) -> Self {
        self.full_filters = filters;
        self
    }

    pub fn add_filter(&mut self, filter: ScopeBorrowClosure) {
        self.filters.push(filter);
    }

    pub fn add_full_filter(&mut self, filter: NameScopeBorrowClosure) {
        self.full_filters.push(filter);
    }

    pub fn filter(
        &self,
        scope: &Rc<RefCell<Scope>>,
    ) -> Vec<(String, (Rc<ScopeEntry>, BorrowStatus))> {
        let mut result: Vec<(String, (Rc<ScopeEntry>, BorrowStatus))> = Vec::new();
        for (entry_name, (scope_entry, borrow_status)) in scope.borrow().get_all_entries() {
            let mut add_entry = true;

            for small_filter in &self.filters {
                add_entry &= small_filter(scope_entry.clone(), borrow_status);
            }
            for big_filter in &self.full_filters {
                add_entry &= big_filter(&entry_name, scope_entry.clone(), borrow_status);
            }

            if add_entry {
                result.push((entry_name, (scope_entry, borrow_status)));
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use crate::{
        generator::{
            context::Context,
            scope_entry::{StructScopeEntry, VarScopeEntry},
            struct_gen::StructTable,
        },
        program::{struct_template::StructTemplate, types::IntTypeID},
    };

    use super::*;
    #[test]
    fn multiple_filter_conditions_with_filter_struct() {
        let mut context = Context::new();
        let mut filters = Filters::new();

        let total_vars = 10;
        let num_bools = 7;
        let num_borrows = 3;

        for i in 0..total_vars {
            let var_type = if i < num_bools {
                TypeID::BoolType
            } else {
                TypeID::NullType
            };
            let var = VarScopeEntry::new(var_type, i.to_string(), false).as_scope_entry();

            context.scope.borrow_mut().insert(&i.to_string(), var);
        }

        // Generate a filter that filters for booleans
        let filters_bool = is_type_filter(TypeID::BoolType);
        filters.add_filter(filters_bool);

        assert_eq!(filters.filter(&context.scope).len(), num_bools);

        context.enter_scope();
        for i in 0..num_borrows {
            let borrow_name = format!("{}_borrow", i);
            let borrow_var = VarScopeEntry::new_ref(TypeID::BoolType, borrow_name.clone(), false)
                .as_scope_entry();
            context
                .scope
                .borrow_mut()
                .insert_borrow(&borrow_name, borrow_var, &i.to_string());
        }

        let filters_is_borrow = is_borrow_type_filter(BorrowTypeID::Ref);
        filters.add_filter(filters_is_borrow);

        assert_eq!(filters.filter(&context.scope).len(), num_borrows);
    }

    #[test]
    fn can_prevent_move_with_filters() {
        let mut context = Context::new();
        let type_a = TypeID::BoolType;
        let type_b = TypeID::NullType;
        let var_a = VarScopeEntry::new(type_a.clone(), "a".to_string(), false).as_scope_entry();
        let var_b = VarScopeEntry::new(type_b.clone(), "b".to_string(), false).as_scope_entry();

        context.scope.borrow_mut().insert(&"a".to_string(), var_a);
        context.scope.borrow_mut().insert(&"b".to_string(), var_b);

        context.enter_scope();

        let var_a_borrow =
            VarScopeEntry::new_ref(type_a.clone(), "a_borrow".to_string(), false).as_scope_entry();

        context.scope.borrow_mut().insert_borrow(
            &"a_borrow".to_string(),
            var_a_borrow,
            &"a".to_string(),
        );

        let move_filter = can_move_filter(Rc::clone(&context.scope));

        assert_eq!(
            context
                .scope
                .borrow()
                .filter_with_closure_full(move_filter)
                .len(),
            1
        );

        // Sanity check
        assert_eq!(
            context
                .scope
                .borrow()
                .filter_with_closure(is_borrowed_filter())
                .len(),
            1
        );
    }

    #[test]
    /*
        This test reflects a function call where the struct field is borrowed and
        an attempt is made to move the struct. (should be illegal)

        struct B -> {field1: C}
        struct C -> {field1: i32}
    */
    fn move_filter_disables_moving_borrowed_variables() {
        let mut scope = Scope::new();

        let struct_c_type = TypeID::StructType("C".to_string());
        let struct_c_fields = vec![("field1".to_string(), IntTypeID::I32.as_type())];
        let struct_c_template = StructTemplate::new_from_fields("C".to_string(), struct_c_fields);

        let struct_b_fields = vec![("field1".to_string(), struct_c_type.clone())];
        let struct_b_template = StructTemplate::new_from_fields("B".to_string(), struct_b_fields);

        let mut struct_table = StructTable::new();
        struct_table.insert_struct(struct_b_template.clone());
        struct_table.insert_struct(struct_c_template.clone());

        let entry_b =
            StructScopeEntry::new(BorrowTypeID::None, struct_b_template, &struct_table, false)
                .as_scope_entry();

        scope.insert(&"b".to_string(), entry_b);

        scope.borrow_entry(&"dummy".to_string(), &"b.field1.field1".to_string());

        let scope = Rc::new(RefCell::new(scope));

        let move_filter = can_move_filter(scope.clone());

        assert_eq!(
            scope.borrow().filter_with_closure_full(move_filter).len(),
            0
        );
    }
}
