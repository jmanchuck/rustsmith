use rand::prelude::SliceRandom;
use rand::Rng;

use crate::program::function::FunctionTemplate;
use crate::program::struct_template::StructTemplate;
use crate::program::types::TypeID;
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

    pub fn is_type(&self, type_id: TypeID) -> bool {
        self.get_type() == type_id
    }

    pub fn is_mut_var(&self) -> bool {
        match self {
            Self::Var(var) => var.is_mut(),
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
    struct_template: StructTemplate,
    fields_map: BTreeMap<String, VarScopeEntry>,
    is_mut: bool,
}

impl StructScopeEntry {
    pub fn new(
        struct_var_name: String,
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
            struct_template,
            fields_map,
            is_mut,
        }
    }

    pub fn is_mut(&self) -> bool {
        self.is_mut
    }

    pub fn get_type(&self) -> TypeID {
        self.type_id.clone()
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
    entries: BTreeMap<String, Rc<ScopeEntry>>,
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
        self.entries.insert(name, entry);
    }

    pub fn lookup(&self, name: String) -> Option<Rc<ScopeEntry>> {
        if self.entries.contains_key(&name) {
            return Some(Rc::clone(self.entries.get(&name).unwrap()));
        }
        match &self.parent {
            Some(parent_scope) => parent_scope.borrow().lookup(name),
            None => None,
        }
    }

    pub fn remove_entry(&mut self, name: String) {
        if self.entries.contains_key(&name) {
            self.entries.remove(&name);
        } else {
            match &self.parent {
                Some(parent_scope) => parent_scope.borrow_mut().remove_entry(name),
                _ => (),
            }
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn len_all(&self) -> usize {
        match &self.parent {
            Some(parent) => self.len() + parent.borrow().len(),
            None => self.len(),
        }
    }

    pub fn filter_by_type(&self, type_id: TypeID) -> Vec<(String, Rc<ScopeEntry>)> {
        let mut result: Vec<(String, Rc<ScopeEntry>)> = Vec::new();

        for (name, entry) in self.get_all_entries().iter() {
            if entry.is_type(type_id.clone()) {
                result.push((name.clone(), Rc::clone(entry)));
            }
        }

        result
    }

    pub fn filter_with_closure<T>(&self, filter: T) -> Vec<(String, Rc<ScopeEntry>)>
    where
        T: Fn(&ScopeEntry) -> bool,
    {
        let mut result: Vec<(String, Rc<ScopeEntry>)> = Vec::new();
        let entries = self.get_all_entries();

        for (entry_name, entry) in entries {
            if filter(entry.as_ref()) {
                result.push((entry_name, Rc::clone(&entry)));
            }
        }

        result
    }

    // Quicker than getting all entries since we check the scope stack from top to bottom
    pub fn contains_type(&self, type_id: TypeID) -> bool {
        for (_, entry) in self.entries.iter() {
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

    pub fn rand_mut<R: Rng>(&self, rng: &mut R) -> Rc<ScopeEntry> {
        let mut_var_list: Vec<Rc<ScopeEntry>> = self
            .get_all_entries()
            .values()
            .filter(|x| x.is_mut_var())
            .map(|x| Rc::clone(&x))
            .collect();

        Rc::clone(mut_var_list.choose(rng).unwrap())
    }

    pub fn mut_count(&self) -> usize {
        // Filters on condition that entry is a variable and is mutable then counts it
        let local_count = self
            .entries
            .iter()
            .filter(|entry| -> bool {
                if let ScopeEntry::Var(e) = entry.1.as_ref() {
                    e.is_mut()
                } else {
                    false
                }
            })
            .count();

        // Count parent if parent is available
        let parent_count = if let Some(parent_scope) = &self.parent {
            parent_scope.borrow().mut_count()
        } else {
            0
        };

        local_count + parent_count
    }

    pub fn get_entries(&self) -> BTreeMap<String, Rc<ScopeEntry>> {
        let mut entries_view: BTreeMap<String, Rc<ScopeEntry>> = BTreeMap::new();
        for (entry_name, scope_entry) in self.entries.iter() {
            entries_view.insert(entry_name.clone(), Rc::clone(scope_entry));
            if let ScopeEntry::Struct(struct_scope_entry) = scope_entry.as_ref() {
                for (entry_name, scope_entry) in struct_scope_entry.get_field_entries() {
                    entries_view.insert(entry_name.clone(), Rc::clone(&scope_entry));
                }
            }
        }
        entries_view
    }

    pub fn get_all_entries(&self) -> BTreeMap<String, Rc<ScopeEntry>> {
        let mut result: BTreeMap<String, Rc<ScopeEntry>> = BTreeMap::new();
        self.get_parent_entries(&mut result);

        result
    }

    fn get_parent_entries(&self, result: &mut BTreeMap<String, Rc<ScopeEntry>>) {
        for (entry_name, scope_entry) in self.get_entries() {
            if !result.contains_key(&entry_name) {
                result.insert(entry_name.clone(), Rc::clone(&scope_entry));
            }
        }
        match &self.parent {
            Some(parent_scope) => parent_scope.borrow().get_parent_entries(result),
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
            struct_template,
            vec![(String::from(".field_1"), IntTypeID::U8.as_type())],
            false,
        );

        scope.add(var_name, Rc::new(struct_scope_entry.as_scope_entry()));

        println!("{:#?}", scope);

        assert_eq!(2, scope.get_all_entries().len());
    }
}
