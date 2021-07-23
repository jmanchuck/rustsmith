use rand::prelude::SliceRandom;
use rand::Rng;

use crate::program::function::{FunctionTemplate, Param};
use crate::program::struct_template::StructTemplate;
use crate::program::types::{BorrowStatus, BorrowTypeID, TypeID};
use crate::program::var::Var;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum ScopeEntry {
    Var(VarScopeEntry),
    Func(FuncScopeEntry),
    Struct(StructScopeEntry),
}

impl ScopeEntry {
    pub fn get_type(&self) -> TypeID {
        match self {
            Self::Var(entry) => entry.get_type(),
            Self::Func(entry) => entry.get_type(),
            Self::Struct(entry) => entry.get_type(),
        }
    }

    pub fn get_borrow_type(&self) -> BorrowTypeID {
        match self {
            Self::Var(entry) => entry.get_borrow_type(),
            Self::Func(_) => BorrowTypeID::None,
            Self::Struct(entry) => entry.get_borrow_type(),
        }
    }

    pub fn is_type(&self, type_id: TypeID) -> bool {
        self.get_type() == type_id
    }

    pub fn is_borrow_type(&self, borrow_type_id: BorrowTypeID) -> bool {
        self.get_borrow_type() == borrow_type_id
    }

    pub fn is_mut(&self) -> bool {
        match self {
            Self::Var(var) => var.is_mut(),
            Self::Struct(s) => s.is_mut(),
            _ => false,
        }
    }

    pub fn is_var(&self) -> bool {
        match self {
            Self::Var(_) => true,
            _ => false,
        }
    }

    pub fn is_func(&self) -> bool {
        match self {
            Self::Func(_) => true,
            _ => false,
        }
    }

    pub fn is_struct(&self) -> bool {
        match self {
            Self::Struct(_) => true,
            _ => false,
        }
    }
}

pub type VarScopeEntry = Var;
impl VarScopeEntry {
    pub fn as_scope_entry(self) -> ScopeEntry {
        ScopeEntry::Var(self)
    }
}

#[derive(Debug)]
pub struct FuncScopeEntry {
    type_id: TypeID,
    function_template: FunctionTemplate,
}

impl FuncScopeEntry {
    pub fn new(type_id: TypeID, function_template: FunctionTemplate) -> Self {
        FuncScopeEntry {
            type_id,
            function_template,
        }
    }

    pub fn get_type(&self) -> TypeID {
        self.type_id.clone()
    }

    pub fn get_template(&self) -> FunctionTemplate {
        self.function_template.clone()
    }

    pub fn as_scope_entry(self) -> ScopeEntry {
        ScopeEntry::Func(self)
    }
}

pub struct StructScopeEntry {
    type_id: TypeID,
    borrow_type: BorrowTypeID,
    struct_template: StructTemplate,
    fields_map: BTreeMap<String, VarScopeEntry>,
    is_mut: bool,
}

impl StructScopeEntry {
    pub fn new(
        struct_var_name: String,
        borrow_type: BorrowTypeID,
        struct_template: StructTemplate,
        flattened_fields: Vec<(String, TypeID)>,
        is_mut: bool,
    ) -> Self {
        let mut fields_map: BTreeMap<String, VarScopeEntry> = BTreeMap::new();
        for (field_name, field_type) in flattened_fields {
            let mapped_name = format!("{}{}", struct_var_name.clone(), field_name.clone());
            let var_scope_entry = VarScopeEntry::new(field_type, mapped_name.clone(), is_mut);
            fields_map.insert(mapped_name, var_scope_entry);
        }

        StructScopeEntry {
            type_id: struct_template.get_type(),
            borrow_type,
            struct_template,
            fields_map,
            is_mut,
        }
    }

    pub fn from_param(
        param: &Param,
        struct_template: StructTemplate,
        flattened_fields: Vec<(String, TypeID)>,
    ) -> Self {
        // Mutable reference as param means that its fields can be assigned to
        let is_mut_ref = param.get_borrow_type() == BorrowTypeID::MutRef;

        let mut fields_map: BTreeMap<String, VarScopeEntry> = BTreeMap::new();
        for (field_name, field_type) in flattened_fields {
            let mapped_name = format!("{}{}", param.get_name(), field_name.clone());
            let var_scope_entry = VarScopeEntry::new(field_type, mapped_name.clone(), is_mut_ref);
            fields_map.insert(mapped_name, var_scope_entry);
        }

        // TODO: Allow mutable params
        StructScopeEntry {
            type_id: param.get_type(),
            borrow_type: param.get_borrow_type(),
            struct_template,
            fields_map,
            is_mut: false,
        }
    }

    pub fn is_mut(&self) -> bool {
        self.is_mut
    }

    pub fn get_type(&self) -> TypeID {
        self.type_id.clone()
    }

    pub fn get_borrow_type(&self) -> BorrowTypeID {
        self.borrow_type.clone()
    }

    pub fn get_field_entries(&self) -> BTreeMap<String, Rc<ScopeEntry>> {
        let mut result: BTreeMap<String, Rc<ScopeEntry>> = BTreeMap::new();
        for (field_name, var_scope_entry) in &self.fields_map {
            result.insert(
                field_name.clone(),
                Rc::new(ScopeEntry::Var(var_scope_entry.clone())),
            );
        }

        result
    }

    pub fn remove_field(&mut self, field_name: String) {
        self.fields_map.remove(&field_name);
    }

    pub fn as_scope_entry(self) -> ScopeEntry {
        ScopeEntry::Struct(self)
    }
}

impl fmt::Debug for StructScopeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StructScope")
            .field("Template", &self.struct_template)
            .field("Mutable", &self.is_mut)
            .finish()
    }
}

pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    entries: BTreeMap<String, (Rc<ScopeEntry>, BorrowStatus)>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            parent: None,
            entries: BTreeMap::new(),
        }
    }

    pub fn new_from_parent(parent: Rc<RefCell<Scope>>) -> Self {
        Scope {
            parent: Some(parent),
            entries: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, name: String, entry: Rc<ScopeEntry>) {
        self.add_with_borrow_type(name, entry, BorrowStatus::None);
    }

    pub fn add_with_borrow_type(
        &mut self,
        name: String,
        entry: Rc<ScopeEntry>,
        borrow_status: BorrowStatus,
    ) {
        self.entries.insert(name, (entry, borrow_status));
    }

    pub fn lookup(&self, name: String) -> Option<(Rc<ScopeEntry>, BorrowStatus)> {
        if self.entries.contains_key(&name) {
            return Some(self.entries.get(&name).unwrap().clone());
        }
        match &self.parent {
            Some(parent_scope) => parent_scope.borrow().lookup(name),
            None => None,
        }
    }

    pub fn remove_entry(&mut self, mut name: String) {
        // Remove the entire struct entry
        if name.contains('.') {
            name = name.split_at(name.find('.').unwrap()).0.to_string();
        }

        if self.entries.contains_key(&name) {
            self.entries.remove(&name);
        } else {
            match &self.parent {
                Some(parent_scope) => parent_scope.borrow_mut().remove_entry(name),
                None => (),
            }
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn set_borrow_status(&mut self, mut name: String, borrow_status: BorrowStatus) {
        // Sets the entire struct entry borrow status
        if name.contains('.') {
            name = name.split_at(name.find('.').unwrap()).0.to_string();
        }
        if self.entries.contains_key(&name) {
            self.entries.get_mut(&name).unwrap().1 = borrow_status;
        } else {
            match &self.parent {
                Some(parent_scope) => parent_scope
                    .borrow_mut()
                    .set_borrow_status(name, borrow_status),
                None => (),
            }
        }
    }

    pub fn filter_by_type(&self, type_id: TypeID) -> Vec<(String, Rc<ScopeEntry>)> {
        let filter = |scope_entry: &ScopeEntry, _| -> bool { scope_entry.is_type(type_id.clone()) };

        self.filter_with_closure(filter)
            .into_iter()
            .map(|(x, y, _)| (x, y))
            .collect()
    }

    pub fn filter_with_closure<T>(&self, filter: T) -> Vec<(String, Rc<ScopeEntry>, BorrowStatus)>
    where
        T: Fn(&ScopeEntry, BorrowStatus) -> bool,
    {
        let mut result: Vec<(String, Rc<ScopeEntry>, BorrowStatus)> = Vec::new();
        for (entry_name, (entry, borrow_status)) in self.get_all_entries() {
            if filter(entry.as_ref(), borrow_status) {
                result.push((entry_name, Rc::clone(&entry), borrow_status));
            }
        }

        result
    }

    pub fn contains_filter<T>(&self, filter: T) -> bool
    where
        T: Fn(&ScopeEntry, BorrowStatus) -> bool,
    {
        for (_, (entry, borrow_status)) in self.get_all_entries() {
            if filter(entry.as_ref(), borrow_status) {
                return true;
            }
        }
        return false;
    }

    // Quicker than getting all entries since we check the scope stack from top to bottom
    pub fn contains_type(&self, type_id: TypeID) -> bool {
        for (_, (entry, _)) in self.entries.iter() {
            if entry.is_type(type_id.clone()) {
                return true;
            }
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().contains_type(type_id);
        }
        return false;
    }

    pub fn contains_var_type(&self, type_id: TypeID) -> bool {
        for (_, entry) in self.filter_by_type(type_id).iter() {
            if let ScopeEntry::Var(_) = entry.as_ref() {
                return true;
            }
        }
        return false;
    }

    pub fn contains_function_type(&self, type_id: TypeID) -> bool {
        for (_, entry) in self.filter_by_type(type_id).iter() {
            if let ScopeEntry::Func(_) = entry.as_ref() {
                return true;
            }
        }
        return false;
    }

    pub fn rand_mut<R: Rng>(&self, rng: &mut R) -> (String, Rc<ScopeEntry>, BorrowStatus) {
        let filter = |scope_entry: &ScopeEntry, borrow_status: BorrowStatus| -> bool {
            scope_entry.is_mut() && (borrow_status != BorrowStatus::Borrowed)
        };
        let mutables = self.filter_with_closure(filter);

        mutables.choose(rng).unwrap().clone()
    }

    pub fn mut_count(&self) -> usize {
        self.filter_with_closure(
            |scope_entry: &ScopeEntry, borrow_status: BorrowStatus| -> bool {
                scope_entry.is_mut() && borrow_status != BorrowStatus::Borrowed
            },
        )
        .len()
    }

    pub fn get_entries(&self) -> BTreeMap<String, (Rc<ScopeEntry>, BorrowStatus)> {
        self.entries.clone()
    }

    // Should all be unique names since we start from current scope and work up, only adding new names (i.e. nearest name in scope is seen)
    pub fn get_all_entries(&self) -> BTreeMap<String, (Rc<ScopeEntry>, BorrowStatus)> {
        let mut result: BTreeMap<String, (Rc<ScopeEntry>, BorrowStatus)> = BTreeMap::new();
        self.get_entries_r(&mut result);

        result
    }

    // Recursively gets entries
    fn get_entries_r(&self, result: &mut BTreeMap<String, (Rc<ScopeEntry>, BorrowStatus)>) {
        for (entry_name, (scope_entry, borrow_status)) in self.get_entries() {
            // If variable name is not in scope
            if !result.contains_key(&entry_name) {
                result.insert(entry_name.clone(), (Rc::clone(&scope_entry), borrow_status));

                // Flattening done here to avoid name overlap
                // i.e. let a = struct -> let a = i32 -> a.field is no longer valid
                if let ScopeEntry::Struct(struct_scope_entry) = scope_entry.as_ref() {
                    for (entry_name, scope_entry) in struct_scope_entry.get_field_entries() {
                        result.insert(entry_name.clone(), (Rc::clone(&scope_entry), borrow_status));
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

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scope")
            .field("Entries", &self.get_all_entries())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use crate::program::types::IntTypeID;

    use super::*;
    #[test]
    fn correct_number_of_entries() {
        let scope = Rc::new(RefCell::new(Scope::new()));
        let num_parent_scope = 5;

        for i in 0..num_parent_scope {
            scope.borrow_mut().add(
                i.to_string(),
                Rc::new(
                    VarScopeEntry::new(TypeID::NullType, String::new(), false).as_scope_entry(),
                ),
            );
        }

        let mut child_scope = Scope::new_from_parent(Rc::clone(&scope));
        let num_child_scope = 10;

        for i in 0..num_child_scope {
            child_scope.add(
                (i + num_parent_scope).to_string(), // avoid variable name overlap
                Rc::new(
                    VarScopeEntry::new(TypeID::NullType, String::new(), false).as_scope_entry(),
                ),
            );
        }

        assert_eq!(
            child_scope.get_all_entries().len(),
            num_parent_scope + num_child_scope
        );

        assert_eq!(scope.borrow_mut().get_all_entries().len(), num_parent_scope);
    }
    #[test]
    fn uses_top_scope_variables_for_duplicate_variable_names() {
        let scope = Rc::new(RefCell::new(Scope::new()));
        let num_parent_scope = 5;

        for i in 0..num_parent_scope {
            scope.borrow_mut().add(
                i.to_string(),
                Rc::new(
                    VarScopeEntry::new(TypeID::NullType, String::new(), false).as_scope_entry(),
                ),
            );
        }

        let mut child_scope = Scope::new_from_parent(Rc::clone(&scope));
        let num_child_scope = 3;

        for i in 0..num_child_scope {
            child_scope.add(
                i.to_string(), // new scope should take up 3 names
                Rc::new(
                    VarScopeEntry::new(TypeID::NullType, String::new(), false).as_scope_entry(),
                ),
            );
        }

        assert_eq!(child_scope.get_all_entries().len(), num_parent_scope);
        assert_eq!(scope.borrow_mut().get_all_entries().len(), num_parent_scope);
    }

    #[test]
    fn checks_if_contains_type_correctly() {
        let scope = Rc::new(RefCell::new(Scope::new()));

        let type_id = IntTypeID::U8.as_type();
        let var_name = String::from("a");

        scope.borrow_mut().add(
            var_name.clone(),
            Rc::new(VarScopeEntry::new(type_id.clone(), var_name, false).as_scope_entry()),
        );

        assert!(scope.borrow().contains_type(type_id.clone()));

        let child_scope = Scope::new_from_parent(Rc::clone(&scope));

        assert!(child_scope.contains_type(type_id));
    }

    #[test]
    fn filters_by_type_correctly() {
        let mut scope = Scope::new();
        let type_id = TypeID::StructType(String::from("StructA"));

        let bogus_type = IntTypeID::I64.as_type();
        let bogus_var = VarScopeEntry::new(bogus_type, String::from("other_type"), false);
        scope.add(
            String::from("other_type"),
            Rc::new(bogus_var.as_scope_entry()),
        );

        // Add 4 entries
        for i in 'a'..='d' {
            let var = VarScopeEntry::new(type_id.clone(), i.to_string(), false);
            scope.add(i.to_string(), Rc::new(var.as_scope_entry()));
        }
        let filtered: Vec<(String, Rc<ScopeEntry>)> = scope.filter_by_type(type_id.clone());
        assert_eq!(filtered.len(), 4);

        let mut child_scope = Scope::new_from_parent(Rc::new(RefCell::new(scope)));

        // Add another 4 entries
        for i in 'e'..='h' {
            let var = VarScopeEntry::new(type_id.clone(), i.to_string(), false);
            child_scope.add(i.to_string(), Rc::new(var.as_scope_entry()));
        }

        let filtered = child_scope.filter_by_type(type_id.clone());

        assert_eq!(filtered.len(), 8);
    }

    #[test]
    fn flattens_struct_field_scope_entries() {
        let mut scope = Scope::new();

        let var_name = String::from("test");
        let struct_name = String::from("Test");
        let mut struct_template = StructTemplate::new(struct_name);

        struct_template.insert_field(String::from("field_1"), IntTypeID::U8.as_type());

        // 3rd argument flattened_fields expects an already flattened string with periods, allowing access
        // directly when obtaining the scope_entry from the flattened struct
        let struct_scope_entry = StructScopeEntry::new(
            var_name.clone(),
            BorrowTypeID::None,
            struct_template,
            vec![(String::from(".field_1"), IntTypeID::U8.as_type())],
            false,
        );

        scope.add(var_name, Rc::new(struct_scope_entry.as_scope_entry()));

        println!("{:#?}", scope);

        assert_eq!(2, scope.get_all_entries().len());
    }

    #[test]
    fn filters_with_closure_correctly() {
        let scope = Rc::new(RefCell::new(Scope::new()));

        let type_id = IntTypeID::U8.as_type();
        let var_name = String::from("a");

        scope.borrow_mut().add(
            var_name.clone(),
            Rc::new(VarScopeEntry::new(type_id.clone(), var_name, false).as_scope_entry()),
        );

        let correct_closure = |scope_entry: &ScopeEntry, _| -> bool {
            scope_entry.is_var() && scope_entry.get_type() == type_id.clone()
        };

        assert_eq!(scope.borrow().filter_with_closure(correct_closure).len(), 1);

        let incorrect_closure = |scope_entry: &ScopeEntry, _| -> bool {
            scope_entry.is_func() && scope_entry.get_type() == type_id.clone()
        };

        assert_eq!(
            scope.borrow().filter_with_closure(incorrect_closure).len(),
            0
        );
    }

    #[test]
    fn updates_borrow_status_correctly() {
        let mut scope = Scope::new();
        let type_id = IntTypeID::U8.as_type();
        let var_name = String::from("a");

        // Borrow status is None
        scope.add(
            var_name.clone(),
            Rc::new(VarScopeEntry::new(type_id.clone(), var_name.clone(), false).as_scope_entry()),
        );

        // Set borrow status to mutably borrowed
        scope.set_borrow_status(var_name, BorrowStatus::MutBorrowed);

        assert!(scope.contains_filter(|x, y| x.is_var() && y == BorrowStatus::MutBorrowed));

        assert!(!scope.contains_filter(|x, y| x.is_var() && y == BorrowStatus::None))
    }

    #[test]
    fn changes_struct_borrow_status_when_borrowing_field() {
        let mut scope = Scope::new();

        let var_name = String::from("test");
        let struct_name = String::from("Test");
        let mut struct_template = StructTemplate::new(struct_name);

        struct_template.insert_field(String::from("field_1"), IntTypeID::U8.as_type());

        let struct_scope_entry = StructScopeEntry::new(
            var_name.clone(),
            BorrowTypeID::None,
            struct_template,
            vec![(String::from(".field_1"), IntTypeID::U8.as_type())],
            false,
        );

        // Borrow status is None for the struct
        scope.add(
            var_name.clone(),
            Rc::new(struct_scope_entry.as_scope_entry()),
        );

        // This should return a vec containing the struct's field
        let result = scope.filter_with_closure(|x, y| x.is_var() && y == BorrowStatus::None);

        assert_eq!(result.len(), 1);

        // Before we borrow the struct field
        assert_eq!(
            scope.get_entries().get(&var_name).unwrap().1,
            BorrowStatus::None
        );

        // Borrow the struct field
        let (struct_field_name, _, _) = &result[0];
        scope.set_borrow_status(struct_field_name.clone(), BorrowStatus::Borrowed);

        // After we borrow the struct field
        assert_eq!(
            scope.get_entries().get(&var_name).unwrap().1,
            BorrowStatus::Borrowed
        );
    }
}
