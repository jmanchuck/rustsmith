/// Data structure presenting a variable in scope
/// Contains variable, struct and function entry types
use std::{collections::BTreeMap, fmt, rc::Rc};

use crate::program::{
    function::{FunctionTemplate, Param},
    struct_template::StructTemplate,
    types::{BorrowTypeID, TypeID},
    var::Var,
};

use super::struct_gen::StructTable;

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
        matches!(self, Self::Var(_))
    }

    pub fn is_func(&self) -> bool {
        matches!(self, Self::Func(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
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
    fields_map: BTreeMap<String, Rc<ScopeEntry>>,
    is_mut: bool,
}

impl StructScopeEntry {
    pub fn new(
        borrow_type: BorrowTypeID,
        struct_template: StructTemplate,
        struct_table: &StructTable,
        is_mut: bool,
    ) -> Self {
        let mut fields_map: BTreeMap<String, Rc<ScopeEntry>> = BTreeMap::new();
        for (field_name, field_type) in struct_template.fields_iter() {
            let scope_entry = match field_type {
                TypeID::StructType(struct_name) => StructScopeEntry::new(
                    BorrowTypeID::None,
                    struct_table.get_struct_template(struct_name).unwrap(),
                    struct_table,
                    is_mut || borrow_type == BorrowTypeID::MutRef,
                )
                .as_scope_entry(),
                _ => Var::new(field_type.clone(), field_name.clone(), is_mut).as_scope_entry(),
            };

            fields_map.insert(field_name.clone(), Rc::new(scope_entry));
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
        struct_table: &StructTable,
    ) -> Self {
        let is_mut = param.get_borrow_type() == BorrowTypeID::MutRef;
        let field_borrow_type = if param.get_borrow_type() != BorrowTypeID::Ref {
            BorrowTypeID::None
        } else {
            BorrowTypeID::Ref
        };

        let mut fields_map: BTreeMap<String, Rc<ScopeEntry>> = BTreeMap::new();
        for (field_name, field_type) in struct_template.fields_iter() {
            let scope_entry = match field_type {
                TypeID::StructType(struct_name) => StructScopeEntry::new(
                    field_borrow_type,
                    struct_table.get_struct_template(struct_name).unwrap(),
                    struct_table,
                    is_mut,
                )
                .as_scope_entry(),
                _ => Var::new(field_type.clone(), field_name.clone(), is_mut).as_scope_entry(),
            };

            fields_map.insert(field_name.clone(), Rc::new(scope_entry));
        }

        StructScopeEntry {
            type_id: struct_template.get_type(),
            borrow_type: param.get_borrow_type(),
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

    pub fn get_borrow_type(&self) -> BorrowTypeID {
        self.borrow_type
    }

    pub fn get_field_entries(&self) -> BTreeMap<String, Rc<ScopeEntry>> {
        self.get_field_entries_r()
    }

    fn get_field_entries_r(&self) -> BTreeMap<String, Rc<ScopeEntry>> {
        let mut result: BTreeMap<String, Rc<ScopeEntry>> = BTreeMap::new();

        for (field_name, scope_entry) in &self.fields_map {
            result.insert(field_name.clone(), scope_entry.clone());

            if let ScopeEntry::Struct(struct_scope_entry) = scope_entry.as_ref() {
                for (sub_field_name, scope_entry) in struct_scope_entry.get_field_entries_r() {
                    result.insert(format!("{}.{}", field_name, sub_field_name), scope_entry);
                }
            }
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
    use crate::program::types::IntTypeID;

    use super::*;
    #[test]
    /*  Flattening entries:
        2 entries: struct A -> {field1: B, field2: i32}
        2 entries: struct B -> {field1: C, field2: i32}
        1 entry: struct C -> {field1: i32}
    */
    fn struct_scope_entry_flattens_correctly() {
        let i32_type = IntTypeID::I32.as_type();

        let struct_c_type = TypeID::StructType("C".to_string());
        let struct_c_fields = vec![("field_1".to_string(), i32_type.clone())];
        let struct_c_template = StructTemplate::new_from_fields("C".to_string(), struct_c_fields);

        let struct_b_type = TypeID::StructType("B".to_string());
        let struct_b_fields = vec![
            ("field_1".to_string(), struct_c_type.clone()),
            ("field_2".to_string(), i32_type.clone()),
        ];
        let struct_b_template = StructTemplate::new_from_fields("B".to_string(), struct_b_fields);
        let struct_a_fields = vec![
            ("field_1".to_string(), struct_b_type.clone()),
            ("field_2".to_string(), i32_type.clone()),
        ];
        let struct_a_template = StructTemplate::new_from_fields("A".to_string(), struct_a_fields);

        let mut struct_table = StructTable::new();
        struct_table.insert_struct(struct_a_template.clone());
        struct_table.insert_struct(struct_b_template.clone());
        struct_table.insert_struct(struct_c_template.clone());

        let a_scope_entry =
            StructScopeEntry::new(BorrowTypeID::None, struct_a_template, &struct_table, false);

        assert_eq!(a_scope_entry.get_field_entries().len(), 5);
    }
}
