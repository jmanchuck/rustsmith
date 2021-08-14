use std::{cell::RefCell, collections::BTreeMap, fmt, rc::Rc};

use rand::{prelude::SliceRandom, Rng};

use crate::program::types::{BorrowStatus, BorrowTypeID};

use super::{borrow_scope::BorrowContext, scope_entry::ScopeEntry};

pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    entries: BTreeMap<String, Rc<ScopeEntry>>,
    borrows: BTreeMap<String, BorrowContext>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            parent: None,
            entries: BTreeMap::new(),
            borrows: BTreeMap::new(),
        }
    }

    pub fn new_from_parent(parent: Rc<RefCell<Scope>>) -> Self {
        let borrows = parent.borrow().borrows.clone();
        Scope {
            parent: Some(parent),
            entries: BTreeMap::new(),
            borrows,
        }
    }

    pub fn get_parent(&self) -> Option<Rc<RefCell<Scope>>> {
        self.parent.clone()
    }

    pub fn insert(&mut self, entry_name: &String, scope_entry: ScopeEntry) {
        self.add_entry(entry_name.clone(), scope_entry, None);
    }

    pub fn insert_borrow(
        &mut self,
        entry_name: &String,
        scope_entry: ScopeEntry,
        borrow_source: &String,
    ) {
        self.add_entry(entry_name.clone(), scope_entry, Some(borrow_source.clone()));
        self.borrow_entry(entry_name, borrow_source);
    }

    pub fn insert_mut_borrow(
        &mut self,
        entry_name: &String,
        scope_entry: ScopeEntry,
        borrow_source: &String,
    ) {
        self.add_entry(entry_name.clone(), scope_entry, Some(borrow_source.clone()));
        self.mut_borrow_entry(entry_name, borrow_source);
    }

    fn add_entry(
        &mut self,
        entry_name: String,
        scope_entry: ScopeEntry,
        borrow_source: Option<String>,
    ) {
        if let ScopeEntry::Struct(struct_scope_entry) = &scope_entry {
            for (field_name, _) in struct_scope_entry.get_field_entries() {
                self.borrows.insert(
                    format!("{}.{}", entry_name.clone(), field_name),
                    BorrowContext::new(borrow_source.clone()),
                );
            }
        }
        self.entries
            .insert(entry_name.clone(), Rc::new(scope_entry));
        self.borrows
            .insert(entry_name.clone(), BorrowContext::new(borrow_source));
    }

    pub fn borrow_struct_field_entry(&mut self, borrower: &String, borrow_source: &String) {
        let borrow_source = String::from(borrow_source);

        // Splits a.field1.field2 to [a, field1, field2]
        let separated: Vec<&str> = borrow_source.split('.').collect();

        // Borrows a, a.field1, a.field1.field2
        for i in 0..separated.len() {
            let entry_to_borrow = separated[..=i].join(".");
            self.borrow_entry_raw(borrower, &entry_to_borrow);
        }

        // If a.field1.field2 is a struct, borrow all its child fields
        match self.lookup(&borrow_source) {
            Some((scope_entry, _)) => {
                if let ScopeEntry::Struct(struct_scope_entry) = scope_entry.as_ref() {
                    for (field_name, _) in struct_scope_entry.get_field_entries() {
                        let full_name = format!("{}.{}", borrow_source, field_name);
                        self.borrow_entry_raw(borrower, &full_name);
                    }
                }
            }
            None => (),
        }
    }

    pub fn borrow_entry(&mut self, borrower: &String, borrow_source: &String) {
        if borrow_source.contains('.') {
            self.borrow_struct_field_entry(borrower, borrow_source);
        } else {
            self.borrow_entry_raw(borrower, borrow_source);
        }
    }

    // Sorry I couldn't think of a better name pt. 1
    fn borrow_entry_raw(&mut self, borrower: &String, borrow_source: &String) {
        if !self.borrows.contains_key(borrow_source) {
            panic!(
                "Could not find borrow source in borrow scope: {}",
                borrow_source
            );
        }
        let borrow_source_context = self.borrows.get_mut(borrow_source).unwrap();
        borrow_source_context.borrow(&borrower);

        match borrow_source_context.get_mut_borrow() {
            Some(borrower) => self.remove_entry(&borrower),
            None => (),
        }
    }

    pub fn mut_borrow_entry(&mut self, borrower: &String, borrow_source: &String) {
        if borrow_source.contains('.') {
            self.mut_borrow_struct_field_entry(borrower, borrow_source);
        } else {
            self.mut_borrow_entry_raw(borrower, borrow_source);
        }
    }

    pub fn mut_borrow_struct_field_entry(&mut self, borrower: &String, borrow_source: &String) {
        let borrow_source = String::from(borrow_source);

        // Splits a.field1.field2 to [a, field1, field2]
        let separated: Vec<&str> = borrow_source.split('.').collect();

        // Borrows a, a.field1, a.field1.field2
        for i in 0..separated.len() {
            let entry_to_borrow = separated[..=i].join(".");
            self.mut_borrow_entry_raw(borrower, &entry_to_borrow);
        }

        // If a.field1.field2 is a struct, borrow all its child fields
        match self.lookup(&borrow_source) {
            Some((scope_entry, _)) => {
                if let ScopeEntry::Struct(struct_scope_entry) = scope_entry.as_ref() {
                    for (field_name, _) in struct_scope_entry.get_field_entries() {
                        let full_name = format!("{}.{}", borrow_source, field_name);
                        self.mut_borrow_entry_raw(borrower, &full_name);
                    }
                }
            }
            None => (),
        }
    }

    // Sorry I couldn't think of a better name pt. 2
    fn mut_borrow_entry_raw(&mut self, borrower: &String, borrow_source: &String) {
        if !self.borrows.contains_key(borrow_source) {
            panic!(
                "Could not find borrow source in borrow scope: {}",
                borrow_source
            );
        }
        // Delete previous immutable borrows
        let prev_borrows = self.borrows.get(borrow_source).unwrap().get_borrows();
        for borrow in prev_borrows {
            self.remove_entry(&borrow);
        }

        // Mutably borrow, deleting previous mutable borrow if exists
        let borrow_source_context = self.borrows.get_mut(borrow_source).unwrap();
        let result = borrow_source_context.mut_borrow(&borrower);

        // Delete previous mut borrow
        match result {
            Some(prev_mut_borrow) => self.remove_entry(&prev_mut_borrow),
            _ => (),
        }
    }

    pub fn borrow_count(&self, entry_name: &String) -> usize {
        match self.borrows.get(entry_name) {
            Some(borrow_context) => borrow_context.get_borrows().len(),
            _ => 0,
        }
    }

    pub fn is_mut_borrowed(&self, entry_name: &String) -> bool {
        match self.borrows.get(entry_name) {
            Some(borrow_context) => borrow_context.is_mut_borrowed(),
            _ => false,
        }
    }

    // Corresponds to a move or remove from scope
    pub fn remove_entry(&mut self, entry_name: &String) {
        if entry_name.contains('.') {
            let parent_struct_var_name = string_before_first_period(entry_name);
            self.remove_struct_scope_entry(&parent_struct_var_name);
        } else {
            self.remove_var_scope_entry(entry_name);
        }
    }

    pub fn lookup(&self, entry_name: &str) -> Option<(Rc<ScopeEntry>, BorrowStatus)> {
        match self.get_all_entries().get(entry_name) {
            Some(entry) => Some(entry.clone()),
            None => None,
        }
    }

    fn get_borrow_source(&self, entry_name: &String) -> Option<String> {
        let result = self.borrows.get(entry_name);

        match result {
            Some(borrow_context) => borrow_context.get_borrow_source(),
            _ => None,
        }
    }

    fn remove_var_scope_entry(&mut self, entry_name: &String) {
        self.remove_scope_entry(entry_name);

        let borrow_source = self.get_borrow_source(entry_name);

        match borrow_source {
            Some(borrow_source) => self.remove_borrow(entry_name, &borrow_source),
            _ => (),
        }
    }

    // Removes all fields from borrow scope and from its borrow sources
    // Removes the struct itself at the end
    fn remove_struct_scope_entry(&mut self, entry_name: &String) {
        let entry = self.lookup(entry_name);

        if let Some((scope_entry, _)) = entry {
            if let ScopeEntry::Struct(struct_scope_entry) = scope_entry.as_ref() {
                for (field_name, _) in struct_scope_entry.get_field_entries() {
                    let full_field_name = format!("{}.{}", entry_name, field_name);
                    self.remove_scope_entry(&full_field_name);

                    let borrow_source = self.get_borrow_source(&full_field_name);

                    match borrow_source {
                        Some(borrow_source) => self.remove_borrow(&full_field_name, &borrow_source),
                        _ => (),
                    }
                }
            }
        }

        self.remove_var_scope_entry(entry_name);
    }

    fn remove_scope_entry(&mut self, entry_name: &String) {
        if self.entries.contains_key(entry_name) {
            self.entries.remove(entry_name);
        } else {
            match &self.parent {
                Some(parent_scope) => parent_scope.borrow_mut().remove_scope_entry(entry_name),
                _ => (),
            }
        }
    }

    // Removes a borrow from some borrow source
    // i.e. if a is borrowed by b, remove(b, a) will delete the borrow
    fn remove_borrow(&mut self, entry_name: &String, borrow_source: &String) {
        let borrow_source_context = self.borrows.get_mut(borrow_source);

        match borrow_source_context {
            Some(borrow_context) => borrow_context.remove_borrow(entry_name),
            _ => match &self.parent {
                Some(parent_scope) => parent_scope
                    .borrow_mut()
                    .remove_borrow(entry_name, borrow_source),
                _ => (),
            },
        }
    }

    pub fn contains_filter<T>(&self, filter: T) -> bool
    where
        T: Fn(Rc<ScopeEntry>, BorrowStatus) -> bool,
    {
        self.filter_with_closure(filter).len() > 0
    }

    pub fn filter_with_closure<T>(&self, filter: T) -> Vec<(String, (Rc<ScopeEntry>, BorrowStatus))>
    where
        T: Fn(Rc<ScopeEntry>, BorrowStatus) -> bool,
    {
        let mut result: Vec<(String, (Rc<ScopeEntry>, BorrowStatus))> = Vec::new();
        for (entry_name, (scope_entry, borrow_status)) in self.get_all_entries() {
            if filter(scope_entry.clone(), borrow_status) {
                result.push((entry_name, (scope_entry, borrow_status)));
            }
        }

        result
    }

    pub fn contains_filter_full<T>(&self, filter: T) -> bool
    where
        T: Fn(&String, Rc<ScopeEntry>, BorrowStatus) -> bool,
    {
        !self.filter_with_closure_full(filter).is_empty()
    }

    pub fn filter_with_closure_full<T>(
        &self,
        filter: T,
    ) -> Vec<(String, (Rc<ScopeEntry>, BorrowStatus))>
    where
        T: Fn(&String, Rc<ScopeEntry>, BorrowStatus) -> bool,
    {
        let mut result: Vec<(String, (Rc<ScopeEntry>, BorrowStatus))> = Vec::new();
        for (entry_name, (scope_entry, borrow_status)) in self.get_all_entries() {
            if filter(&entry_name, scope_entry.clone(), borrow_status) {
                result.push((entry_name, (scope_entry, borrow_status)));
            }
        }

        result
    }

    pub fn can_move_entry(&self, entry_name: &str) -> bool {
        match self.lookup(entry_name) {
            Some((scope_entry, borrow_status)) => {
                let parent_entry_name = string_before_first_period(entry_name);
                if !parent_entry_name.eq(entry_name) {
                    self.can_move_entry(&parent_entry_name)
                } else {
                    scope_entry.is_borrow_type(BorrowTypeID::None)
                        && borrow_status == BorrowStatus::None
                }
            }
            None => false,
        }
    }

    pub fn rand_mut<R: Rng>(
        &self,
        rng: &mut R,
    ) -> Result<(String, (Rc<ScopeEntry>, BorrowStatus)), ()> {
        let filter = |scope_entry: Rc<ScopeEntry>, borrow_status: BorrowStatus| -> bool {
            (scope_entry.is_mut() && (borrow_status != BorrowStatus::Borrowed))
                || borrow_status == BorrowStatus::MutBorrowed
        };
        let mutables = self.filter_with_closure(filter);

        match mutables.choose(rng) {
            Some(choice) => Ok(choice.clone()),
            None => Err(()),
        }
    }

    pub fn mut_count(&self) -> usize {
        self.filter_with_closure(|scope_entry, _| scope_entry.is_mut())
            .len()
    }

    // Should all be unique names since we start from current scope and work up, only adding new names (i.e. nearest name in scope is seen)
    pub fn get_all_entries(&self) -> BTreeMap<String, (Rc<ScopeEntry>, BorrowStatus)> {
        let mut all_entries: BTreeMap<String, Rc<ScopeEntry>> = BTreeMap::new();
        self.get_entries_r(&mut all_entries);

        let mut result: BTreeMap<String, (Rc<ScopeEntry>, BorrowStatus)> = BTreeMap::new();
        for (entry_name, scope_entry) in all_entries {
            let borrow_status = match self.borrows.get(&entry_name) {
                Some(borrow_context) => borrow_context.get_borrow_status(),
                _ => panic!("Could not find borrow context for variable"),
            };

            result.insert(entry_name, (Rc::clone(&scope_entry), borrow_status));
        }

        result
    }

    fn get_entries(&self) -> BTreeMap<String, Rc<ScopeEntry>> {
        self.entries.clone()
    }

    // Recursively gets entries, flatten struct if we can
    fn get_entries_r(&self, result: &mut BTreeMap<String, Rc<ScopeEntry>>) {
        for (entry_name, scope_entry) in self.get_entries() {
            // If variable name is not in scope
            if !result.contains_key(&entry_name) {
                result.insert(entry_name.clone(), Rc::clone(&scope_entry));

                // Flattening done here
                if let ScopeEntry::Struct(struct_scope_entry) = scope_entry.as_ref() {
                    for (field_name, scope_entry) in struct_scope_entry.get_field_entries() {
                        result.insert(
                            format!("{}.{}", entry_name, field_name),
                            scope_entry.clone(),
                        );
                    }
                }
            }
        }
        match &self.parent {
            Some(parent_scope) => parent_scope.borrow().get_entries_r(result),
            None => (),
        }
    }
}

fn string_before_first_period(s: &str) -> String {
    let s = s.to_string();
    let period_idx = match s.find('.') {
        Some(idx) => idx,
        None => s.len(),
    };

    s[..period_idx].to_string()
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scope")
            .field("Entries", &self.get_all_entries())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        generator::{
            scope_entry::{StructScopeEntry, VarScopeEntry},
            struct_gen::StructTable,
        },
        program::{
            struct_template::StructTemplate,
            types::{BorrowTypeID, IntTypeID, TypeID},
            var::Var,
        },
    };

    use super::*;
    #[test]
    fn correct_number_of_entries() {
        let scope = Rc::new(RefCell::new(Scope::new()));
        let num_parent_scope = 5;

        for i in 0..num_parent_scope {
            scope.borrow_mut().insert(
                &i.to_string(),
                VarScopeEntry::new(TypeID::NullType, String::new(), false).as_scope_entry(),
            );
        }

        let mut child_scope = Scope::new_from_parent(Rc::clone(&scope));
        let num_child_scope = 10;

        for i in 0..num_child_scope {
            child_scope.insert(
                &(i + num_parent_scope).to_string(), // avoid variable name overlap
                VarScopeEntry::new(TypeID::NullType, String::new(), false).as_scope_entry(),
            );
        }

        assert_eq!(
            child_scope.get_all_entries().len(),
            num_parent_scope + num_child_scope
        );

        assert_eq!(scope.borrow().get_all_entries().len(), num_parent_scope);
    }

    #[test]
    /*
        struct B -> {field1: C, field2: i32}
        struct C -> {field1: i32}

        Inserting struct B should result in scope entries:
        - b, b.field1, b.field1.field1, b.field2
    */
    fn correct_number_of_struct_entries() {
        let mut scope = Scope::new();

        // Setup begins here
        let i32_type = IntTypeID::I32.as_type();

        let struct_c_type = TypeID::StructType("C".to_string());
        let struct_c_fields = vec![("field_1".to_string(), i32_type.clone())];
        let struct_c_template = StructTemplate::new_from_fields("C".to_string(), struct_c_fields);

        let struct_b_fields = vec![
            ("field_1".to_string(), struct_c_type.clone()),
            ("field_2".to_string(), i32_type.clone()),
        ];
        let struct_b_template = StructTemplate::new_from_fields("B".to_string(), struct_b_fields);

        let mut struct_table = StructTable::new();
        struct_table.insert_struct(struct_c_template);
        struct_table.insert_struct(struct_b_template.clone());

        let struct_entry =
            StructScopeEntry::new(BorrowTypeID::None, struct_b_template, &struct_table, false)
                .as_scope_entry();

        scope.insert(&"a".to_string(), struct_entry);

        assert_eq!(scope.get_all_entries().len(), 4);

        println!("{:#?}", scope.get_all_entries());
    }

    #[test]
    fn removes_borrow() {
        /* This test mimics the following:
            let a = _;
            let b = &a;
            drop(b);
        */
        let mut scope = Scope::new();

        let a = "a".to_string();
        let b = "b".to_string();

        let entry_a = Var::new(TypeID::NullType, a.clone(), false).as_scope_entry();
        let entry_b = Var::new(TypeID::NullType, b.clone(), false).as_scope_entry();

        scope.insert(&a, entry_a);
        scope.insert_borrow(&b, entry_b, &a);

        assert_eq!(scope.borrow_count(&a), 1);

        scope.remove_entry(&b);

        assert_eq!(scope.borrow_count(&a), 0);
    }

    #[test]
    fn mut_borrow_deletes_all_borrows() {
        let mut scope = Scope::new();

        let a = "a".to_string();
        let b = "b".to_string();
        let c = "c".to_string();

        let entry_a = Var::new(TypeID::NullType, a.clone(), false).as_scope_entry();
        let entry_b = Var::new(TypeID::NullType, b.clone(), false).as_scope_entry();
        let entry_c = Var::new(TypeID::NullType, c.clone(), false).as_scope_entry();

        scope.insert(&a, entry_a);
        scope.insert_borrow(&b, entry_b, &a);

        assert_eq!(scope.borrow_count(&a), 1);

        scope.insert_mut_borrow(&c, entry_c, &a);
        assert_eq!(scope.borrow_count(&a), 0);
    }

    #[test]
    fn borrow_deletes_all_mut_borrows() {
        let mut scope = Scope::new();

        let a = "a".to_string();
        let b = "b".to_string();
        let c = "c".to_string();

        let entry_a = Var::new(TypeID::NullType, a.clone(), false).as_scope_entry();
        let entry_b = Var::new(TypeID::NullType, b.clone(), false).as_scope_entry();
        let entry_c = Var::new(TypeID::NullType, c.clone(), false).as_scope_entry();

        scope.insert(&a, entry_a);

        scope.insert_mut_borrow(&c, entry_c, &a);
        assert!(scope.is_mut_borrowed(&a));

        scope.insert_borrow(&b, entry_b, &a);
        assert!(!scope.is_mut_borrowed(&a));
    }

    #[test]
    /*  This test reflects the following case:
        struct A -> {field1: B, field2: i32}
        struct B -> {field1: C, field2: i32}
        struct C -> {field1: i32}

        let a = A();
        borrow a.field1.field1 (type C)

        As a result, the following become borrowed:
        - a (type A)
        - a.field1 (type B)
        - a.field1.field1 (type C)
        - a.field1.field1.field1 (type i32)

        The remaining fields are not borrowed:
        - a.field2
        - a.field1.field2
    */
    fn borrowing_struct_field_correctly_borrows_parents_and_children() {
        let mut scope = Scope::new();

        // Setup begins here
        let i32_type = IntTypeID::I32.as_type();

        let struct_c_type = TypeID::StructType("C".to_string());
        let struct_c_fields = vec![("field1".to_string(), i32_type.clone())];
        let struct_c_template = StructTemplate::new_from_fields("C".to_string(), struct_c_fields);

        let struct_b_type = TypeID::StructType("B".to_string());
        let struct_b_fields = vec![
            ("field1".to_string(), struct_c_type.clone()),
            ("field2".to_string(), i32_type.clone()),
        ];
        let struct_b_template = StructTemplate::new_from_fields("B".to_string(), struct_b_fields);

        let struct_a_fields = vec![
            ("field1".to_string(), struct_b_type.clone()),
            ("field2".to_string(), i32_type.clone()),
        ];
        let struct_a_template = StructTemplate::new_from_fields("A".to_string(), struct_a_fields);

        let var_a = "a".to_string();
        let var_borrower = "borrower".to_string();

        let mut struct_table = StructTable::new();
        struct_table.insert_struct(struct_a_template.clone());
        struct_table.insert_struct(struct_b_template.clone());
        struct_table.insert_struct(struct_c_template.clone());

        let entry_a =
            StructScopeEntry::new(BorrowTypeID::None, struct_a_template, &struct_table, false)
                .as_scope_entry();

        // Setup ends here

        // Borrow test begins here
        scope.insert(&var_a, entry_a);

        let borrow_source = String::from("a.field1.field1");

        // Borrow
        scope.borrow_entry(&"borrower".to_string(), &borrow_source);

        // Borrowed struct fields
        assert_eq!(scope.borrow_count(&var_a), 1);

        assert_eq!(scope.borrow_count(&"a.field1".to_string()), 1);
        assert_eq!(scope.borrow_count(&"a.field1.field1".to_string()), 1);
        assert_eq!(scope.borrow_count(&"a.field1.field1.field1".to_string()), 1);

        // Not borrowed struct fields
        assert_eq!(scope.borrow_count(&"a.field2".to_string()), 0);
        assert_eq!(scope.borrow_count(&"a.field1.field2".to_string()), 0);

        // Shouldn't be able to move anything
        assert!(!scope.can_move_entry(&"a.field1".to_string()));
        assert!(!scope.can_move_entry(&"a.field1.field1".to_string()));
        assert!(!scope.can_move_entry(&"a.field1.field1.field1".to_string()));
        assert!(!scope.can_move_entry(&"a.field2".to_string()));
        assert!(!scope.can_move_entry(&"a.field1.field2".to_string()));

        // Check that removing borrow clears everything
        scope.remove_entry(&var_borrower);
    }
}
