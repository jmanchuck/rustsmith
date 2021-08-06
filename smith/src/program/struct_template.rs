use std::fmt;

use super::types::TypeID;

// Representation of a struct's name, fields (and their types) and any derive macro attributes
#[derive(Clone)]
pub struct StructTemplate {
    fields: Vec<(String, TypeID)>,
    name: String,
    derive: Vec<String>,
}

impl StructTemplate {
    pub fn new(name: String) -> Self {
        StructTemplate {
            fields: Vec::new(),
            name,
            derive: Vec::new(),
        }
    }

    pub fn new_from_fields(name: String, fields: Vec<(String, TypeID)>) -> Self {
        StructTemplate {
            fields,
            name,
            derive: Vec::new(),
        }
    }

    pub fn get_type(&self) -> TypeID {
        TypeID::StructType(self.name.clone())
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn num_fields(&self) -> usize {
        self.fields.len()
    }

    pub fn insert_field(&mut self, name: String, type_id: TypeID) {
        self.fields.push((name, type_id));
    }

    pub fn insert_derive_attribute(&mut self, name: String) {
        self.derive.push(name);
    }

    pub fn fields_iter(&self) -> std::slice::Iter<(String, TypeID)> {
        self.fields.iter()
    }

    pub fn to_string(&self) -> String {
        let mut field_list = String::new();
        for (field_name, field_type) in self.fields.iter() {
            field_list.push_str(format!("{}: {},\n", field_name, field_type.to_string()).as_str());
        }
        let derive_string = if self.derive.len() > 0 {
            format!("#[derive({})]\n", self.derive.join(", "))
        } else {
            String::new()
        };

        let struct_string = format!("struct {} {{\n{}}}", self.name, field_list);

        format!("{}{}", derive_string, struct_string)
    }
}

impl fmt::Debug for StructTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StructTemplate")
            .field("name", &self.name)
            .field("fields", &self.fields)
            .finish()
    }
}
