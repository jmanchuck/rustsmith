use crate::program::types::{BorrowStatus, BorrowTypeID, TypeID};

use super::scope_entry::ScopeEntry;

pub fn is_func_filter() -> Box<dyn Fn(&ScopeEntry, BorrowStatus) -> bool> {
    Box::new(|scope_entry, _| scope_entry.is_func())
}

pub fn is_var_filter() -> Box<dyn Fn(&ScopeEntry, BorrowStatus) -> bool> {
    Box::new(|scope_entry, _| scope_entry.is_var())
}

pub fn is_struct_filter() -> Box<dyn Fn(&ScopeEntry, BorrowStatus) -> bool> {
    Box::new(|scope_entry, _| scope_entry.is_struct())
}

pub fn is_type_filter(type_id: TypeID) -> Box<dyn Fn(&ScopeEntry, BorrowStatus) -> bool> {
    Box::new(move |scope_entry, _| scope_entry.is_type(type_id.clone()))
}

pub fn is_borrow_type_filter(
    borrow_type_id: BorrowTypeID,
) -> Box<dyn Fn(&ScopeEntry, BorrowStatus) -> bool> {
    Box::new(move |scope_entry, _| scope_entry.is_borrow_type(borrow_type_id.clone()))
}

pub fn is_borrowed_filter() -> Box<dyn Fn(&ScopeEntry, BorrowStatus) -> bool> {
    Box::new(|_, borrow_status| borrow_status == BorrowStatus::Borrowed)
}

pub fn is_mut_borrowed_filter() -> Box<dyn Fn(&ScopeEntry, BorrowStatus) -> bool> {
    Box::new(|_, borrow_status| borrow_status == BorrowStatus::MutBorrowed)
}

pub fn is_not_borrowed_filter() -> Box<dyn Fn(&ScopeEntry, BorrowStatus) -> bool> {
    Box::new(|_, borrow_status| borrow_status == BorrowStatus::None)
}

#[cfg(test)]
mod test {
    use crate::generator::scope_entry::VarScopeEntry;

    use super::*;
    #[test]
    fn can_combine_filters() {
        let type_a = TypeID::BoolType;
        let type_b = TypeID::NullType;
        let var_a = VarScopeEntry::new(type_a.clone(), "".to_string(), false).as_scope_entry();
        let var_b = VarScopeEntry::new(type_b.clone(), "".to_string(), false).as_scope_entry();

        let vars = vec![var_a, var_b];

        let filter_a = is_type_filter(type_a);
        let filter_b = is_type_filter(type_b);

        let mut count = 0;

        for var in vars {
            if filter_a(&var, BorrowStatus::None) || filter_b(&var, BorrowStatus::None) {
                count += 1;
            }
        }

        assert_eq!(count, 2);
    }
}
