use std::{cell::RefCell, collections::BTreeMap, fmt, rc::Rc};

use rand::{prelude::SliceRandom, Rng};

use crate::program::{
    function::{FunctionTemplate, Param},
    struct_template::StructTemplate,
    types::{BorrowStatus, BorrowTypeID, TypeID},
    var::Var,
};

use super::borrow_scope::BorrowContext;

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
    pub fn insert(&mut self, entry_name: &String, scope_entry: Rc<ScopeEntry>) {
        self.add_entry(entry_name.clone(), scope_entry, None);
    }

    pub fn insert_borrow(
        &mut self,
        entry_name: &String,
        scope_entry: Rc<ScopeEntry>,
        borrow_source: &String,
    ) {
        self.add_entry(entry_name.clone(), scope_entry, Some(borrow_source.clone()));
        self.borrow_entry(entry_name, borrow_source);
    }

    pub fn insert_mut_borrow(
        &mut self,
        entry_name: &String,
        scope_entry: Rc<ScopeEntry>,
        borrow_source: &String,
    ) {
        self.add_entry(entry_name.clone(), scope_entry, Some(borrow_source.clone()));
        self.mut_borrow_entry(entry_name, borrow_source);
    }

    fn add_entry(
        &mut self,
        entry_name: String,
        scope_entry: Rc<ScopeEntry>,
        borrow_source: Option<String>,
    ) {
        self.entries.insert(entry_name.clone(), scope_entry);
        self.borrows
            .insert(entry_name.clone(), BorrowContext::new(borrow_source));
    }

    pub fn borrow_entry(&mut self, borrower: &String, borrow_source: &String) {
        let borrower = string_before_period(borrower);
        let borrow_source = string_before_period(borrow_source);

        if !self.borrows.contains_key(&borrow_source) {
            panic!(
                "Could not find borrow source in borrow scope: {}",
                borrow_source
            );
        }
        let borrow_source_context = self.borrows.get_mut(&borrow_source).unwrap();
        borrow_source_context.borrow(&borrower);

        match borrow_source_context.get_mut_borrow() {
            Some(borrower) => self.remove_entry(&borrower),
            None => (),
        }
    }

    pub fn mut_borrow_entry(&mut self, borrower: &String, borrow_source: &String) {
        let borrower = string_before_period(borrower);
        let borrow_source = string_before_period(borrow_source);

        if !self.borrows.contains_key(&borrow_source) {
            panic!(
                "Could not find borrow source in borrow scope: {}",
                borrow_source
            );
        }
        // Delete previous immutable borrows
        let prev_borrows = self.borrows.get(&borrow_source).unwrap().get_borrows();
        for borrow in prev_borrows {
            self.remove_entry(&borrow);
        }

        // Mutably borrow, deleting previous mutable borrow if exists
        let borrow_source_context = self.borrows.get_mut(&borrow_source).unwrap();
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

    pub fn remove_entry(&mut self, entry_name: &String) {
        self.remove_scope_entry(entry_name);

        let borrow_source = self.get_borrow_source(entry_name);

        match borrow_source {
            Some(borrow_source) => self.remove_borrow(entry_name, &borrow_source),
            _ => (),
        }
    }

    pub fn lookup(&self, entry_name: &String) -> Option<(Rc<ScopeEntry>, BorrowStatus)> {
        match self.get_all_entries().get(entry_name) {
            Some((entry, status)) => Some((entry.clone(), status.clone())),
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

    // TODO: Make this work for struct fields
    fn remove_scope_entry(&mut self, entry_name: &String) {
        if self.entries.contains_key(entry_name) {
            self.entries.remove(entry_name);
        } else {
            match &mut self.parent {
                Some(parent_scope) => parent_scope.borrow_mut().remove_scope_entry(entry_name),
                _ => (),
            }
        }
    }

    fn remove_borrow(&mut self, entry_name: &String, borrow_source: &String) {
        let borrow_source_context = self.borrows.get_mut(borrow_source);

        match borrow_source_context {
            Some(borrow_context) => borrow_context.remove_borrow(entry_name),
            _ => match &mut self.parent {
                Some(parent_scope) => parent_scope
                    .borrow_mut()
                    .remove_borrow(entry_name, borrow_source),
                _ => (),
            },
        }
    }

    pub fn contains_filter<T>(&self, filter: T) -> bool
    where
        T: Fn(&ScopeEntry, BorrowStatus) -> bool,
    {
        self.filter_with_closure(filter).len() > 0
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

    pub fn rand_mut<R: Rng>(
        &self,
        rng: &mut R,
    ) -> Result<(String, Rc<ScopeEntry>, BorrowStatus), ()> {
        let filter = |scope_entry: &ScopeEntry, borrow_status: BorrowStatus| -> bool {
            scope_entry.is_mut() && (borrow_status != BorrowStatus::Borrowed)
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

        let mut result = BTreeMap::new();
        for (entry_name, scope_entry) in all_entries {
            let borrow_status = match self.borrows.get(&entry_name) {
                Some(borrow_context) => borrow_context.get_borrow_status(),
                _ => panic!("Could not find borrow context for variable"),
            };

            result.insert(entry_name, (Rc::clone(&scope_entry), borrow_status));

            // Flattening done here - currently all children have same mutability as parent
            if let ScopeEntry::Struct(struct_scope_entry) = scope_entry.as_ref() {
                for (entry_name, scope_entry) in struct_scope_entry.get_field_entries() {
                    result.insert(entry_name.clone(), (Rc::clone(&scope_entry), borrow_status));
                }
            }
        }

        result
    }

    fn get_entries(&self) -> BTreeMap<String, Rc<ScopeEntry>> {
        self.entries.clone()
    }

    // Recursively gets entries, but does not flattten
    fn get_entries_r(&self, result: &mut BTreeMap<String, Rc<ScopeEntry>>) {
        for (entry_name, scope_entry) in self.get_entries() {
            // If variable name is not in scope
            if !result.contains_key(&entry_name) {
                result.insert(entry_name.clone(), Rc::clone(&scope_entry));
            }
        }
        match &self.parent {
            Some(parent_scope) => parent_scope.borrow_mut().get_entries_r(result),
            None => (),
        }
    }
}

fn string_before_period(s: &String) -> String {
    let mut s = s.clone();

    if s.contains('.') {
        s = s.split_at(s.find('.').unwrap()).0.to_string();
    }
    s
}

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

#[cfg(test)]
mod test {
    use crate::{
        generator::scope::VarScopeEntry,
        program::{types::TypeID, var::Var},
    };

    use super::*;
    #[test]
    fn correct_number_of_entries() {
        let scope = Rc::new(RefCell::new(Scope::new()));
        let num_parent_scope = 5;

        for i in 0..num_parent_scope {
            scope.borrow_mut().insert(
                &i.to_string(),
                Rc::new(
                    VarScopeEntry::new(TypeID::NullType, String::new(), false).as_scope_entry(),
                ),
            );
        }

        let mut child_scope = Scope::new_from_parent(Rc::clone(&scope));
        let num_child_scope = 10;

        for i in 0..num_child_scope {
            child_scope.insert(
                &(i + num_parent_scope).to_string(), // avoid variable name overlap
                Rc::new(
                    VarScopeEntry::new(TypeID::NullType, String::new(), false).as_scope_entry(),
                ),
            );
        }

        assert_eq!(
            child_scope.get_all_entries().len(),
            num_parent_scope + num_child_scope
        );

        assert_eq!(scope.borrow().get_all_entries().len(), num_parent_scope);
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

        scope.insert(&a, Rc::new(entry_a));
        scope.insert_borrow(&b, Rc::new(entry_b), &a);

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

        scope.insert(&a, Rc::new(entry_a));
        scope.insert_borrow(&b, Rc::new(entry_b), &a);

        assert_eq!(scope.borrow_count(&a), 1);

        scope.insert_mut_borrow(&c, Rc::new(entry_c), &a);
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

        scope.insert(&a, Rc::new(entry_a));

        scope.insert_mut_borrow(&c, Rc::new(entry_c), &a);
        assert!(scope.is_mut_borrowed(&a));

        scope.insert_borrow(&b, Rc::new(entry_b), &a);
        assert!(!scope.is_mut_borrowed(&a));
    }
}
