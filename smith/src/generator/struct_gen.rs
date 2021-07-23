use crate::program::{
    struct_template::StructTemplate,
    types::{IntTypeID, TypeID, TypeIDVariants},
};
use rand::{prelude::SliceRandom, Rng};
use std::{collections::BTreeMap, fmt};

use super::name_gen::NameGenerator;

const MAX_STRUCT_FIELDS: u32 = 10;

pub const GLOBAL_STRUCT_NAME: &str = "StructGlobal";
pub const GLOBAL_STRUCT_VAR_NAME: &str = "struct_global";
const NUM_GLOBAL_STRUCT_FIELDS: u32 = 20;

// Each struct is represented by its name and contains a list of types as fields
pub struct StructTable {
    structs: BTreeMap<String, StructTemplate>,
    names: NameGenerator,
    has_global: bool,
    global_struct: Option<StructTemplate>,
}

impl StructTable {
    pub fn new() -> Self {
        StructTable {
            structs: BTreeMap::new(),
            names: NameGenerator::new(String::from("Struct")),
            has_global: false,
            global_struct: None,
        }
    }

    pub fn len(&self) -> usize {
        self.structs.len()
    }

    // We don't want this type to be INSTANTIATED, but we want it to be passed around as function arguments
    pub fn gen_global_struct(&mut self) -> StructTemplate {
        if self.has_global {
            panic!("Attempting to create another 'global' struct");
        }
        self.has_global = true;
        let name = GLOBAL_STRUCT_NAME.to_string();
        let mut struct_template = StructTemplate::new(name.clone());
        let mut field_name_gen = NameGenerator::new(String::from("field_"));

        for _ in 0..NUM_GLOBAL_STRUCT_FIELDS {
            struct_template.insert_field(field_name_gen.next().unwrap(), IntTypeID::I32.as_type());
        }

        struct_template.insert_derive_attribute(String::from("Serialize"));

        self.global_struct = Some(struct_template.clone());

        struct_template
    }

    pub fn get_global_struct(&self) -> Option<StructTemplate> {
        self.global_struct.clone()
    }

    pub fn insert_struct(&mut self, struct_template: StructTemplate) {
        self.structs
            .insert(struct_template.get_name(), struct_template);
    }

    pub fn flatten_struct(&self, struct_name: &String) -> Vec<(String, TypeID)> {
        let struct_template = self
            .get_struct_template(&struct_name)
            .unwrap_or_else(|| panic!("Could not find struct template"));

        let mut result: Vec<(String, TypeID)> = Vec::new();
        for (field_name, field_type) in struct_template.fields_iter() {
            if let TypeID::StructType(struct_name) = field_type {
                // Recursively flatten fields until we get to primitive
                let nested_fields = self.flatten_struct(struct_name);
                for (nested_field_name, nested_field_type) in nested_fields {
                    let flattened_entry = (
                        format!(".{}{}", field_name, nested_field_name.clone()),
                        nested_field_type.clone(),
                    );
                    result.push(flattened_entry);
                }
            } else {
                let flattened_entry = (format!(".{}", field_name.clone()), field_type.clone());
                result.push(flattened_entry);
            }
        }
        result
    }

    pub fn gen_struct<R: Rng>(&mut self, rng: &mut R) -> StructTemplate {
        let name = self.names.next().unwrap();
        let mut field_name_gen = NameGenerator::new(String::from("field_"));
        let mut struct_template = StructTemplate::new(name.clone());

        while rng.gen_range(0.0..1.0)
            > struct_template.num_fields() as f32 / MAX_STRUCT_FIELDS as f32
        {
            struct_template.insert_field(field_name_gen.next().unwrap(), self.rand_type(rng))
        }

        self.insert_struct(struct_template.clone());

        struct_template
    }

    // Should only be called if struct table contains at least 1 struct
    pub fn get_random_struct_name<R: Rng>(&self, rng: &mut R) -> String {
        let names: Vec<String> = self.structs.keys().cloned().collect();
        names.choose(rng).unwrap().clone()
    }

    pub fn get_random_struct_name_with_global<R: Rng>(&self, rng: &mut R) -> String {
        let mut names: Vec<String> = self.structs.keys().cloned().collect();
        match &self.global_struct {
            Some(struct_template) => names.push(struct_template.get_name()),
            None => (),
        }
        names.choose(rng).unwrap().clone()
    }

    pub fn get_random_struct_template<R: Rng>(&self, rng: &mut R) -> StructTemplate {
        let struct_templates: Vec<StructTemplate> = self.structs.values().cloned().collect();
        struct_templates.choose(rng).unwrap().clone()
    }

    pub fn get_struct_template(&self, name: &String) -> Option<StructTemplate> {
        match self.structs.get(name) {
            Some(struct_template) => Some(struct_template.clone()),
            None => match &self.global_struct {
                Some(struct_template) if *name == struct_template.get_name() => {
                    Some(struct_template.clone())
                }
                _ => None,
            },
        }
    }

    pub fn rand_type_with_global<R: Rng>(&self, rng: &mut R) -> TypeID {
        let mut loop_limit = 20;

        loop {
            let mut selected: TypeIDVariants = rng.gen();
            if loop_limit < 0 {
                if rng.gen::<bool>() {
                    selected = TypeIDVariants::IntType;
                } else {
                    selected = TypeIDVariants::BoolType;
                }
            }
            match selected {
                TypeIDVariants::StructType if self.len() > 0 => {
                    return TypeID::StructType(self.get_random_struct_name_with_global(rng));
                }

                // Inclusive of the NullType
                TypeIDVariants::IntType => {
                    let int_type_id: IntTypeID = rng.gen();
                    return int_type_id.as_type();
                }

                TypeIDVariants::BoolType => return TypeID::BoolType,

                _ => {
                    loop_limit -= 1;
                    continue;
                }
            }
        }
    }

    pub fn rand_type<R: Rng>(&self, rng: &mut R) -> TypeID {
        let mut loop_limit = 20;
        loop {
            let mut selected: TypeIDVariants = rng.gen();
            loop_limit -= 1;
            if loop_limit < 0 {
                if rng.gen::<bool>() {
                    selected = TypeIDVariants::IntType;
                } else {
                    selected = TypeIDVariants::BoolType;
                }
            }
            match selected {
                // Can only choose a struct type if the struct table contains previously generated structs
                // Guard in place since if len is > 0 then get_struct_name() will not return None

                // TODO: perhaps we can modify this when we have a better weighted selection system,
                //       where if there are no structs available we suppress weight to 0, or scale
                //       the weight to the number of generated structs
                TypeIDVariants::StructType if self.len() > 0 => {
                    return TypeID::StructType(self.get_random_struct_name(rng));
                }

                // Inclusive of the NullType
                TypeIDVariants::IntType => {
                    let int_type_id: IntTypeID = rng.gen();
                    return int_type_id.as_type();
                }

                TypeIDVariants::BoolType => return TypeID::BoolType,

                _ => {
                    loop_limit -= 1;
                    continue;
                }
            }
        }
    }

    pub fn rand_type_with_null<R: Rng>(&self, rng: &mut R) -> TypeID {
        let mut loop_limit = 20;
        loop {
            let mut selected: TypeIDVariants = rng.gen();
            loop_limit -= 1;
            if loop_limit < 0 {
                if rng.gen::<bool>() {
                    selected = TypeIDVariants::IntType;
                } else {
                    selected = TypeIDVariants::BoolType;
                }
            }
            match selected {
                // Can only choose a struct type if the struct table contains previously generated structs
                // Guard in place since if len is > 0 then get_struct_name() will not return None
                TypeIDVariants::StructType if self.len() > 0 => {
                    return TypeID::StructType(self.get_random_struct_name(rng));
                }

                // No null types in struct
                TypeIDVariants::NullType => return TypeID::NullType,
                TypeIDVariants::IntType => {
                    let int_type_id: IntTypeID = rng.gen();
                    return int_type_id.as_type();
                }
                TypeIDVariants::BoolType => return TypeID::BoolType,
                _ => continue,
            }
        }
    }
}

impl fmt::Debug for StructTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StructTable")
            .field("structs", &self.structs)
            .field("global_struct", &self.global_struct)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn creates_new_symbol_with_correct_name() {
        let mut table = StructTable::new();
        table.gen_struct(&mut rand::thread_rng());

        assert_eq!(table.len(), 1);
    }

    #[test]
    fn flattens_struct_properly() {
        /* CONSTRUCTING THE STRUCT TABLE AND STRUCT TEMPLATES */
        let mut table = StructTable::new();

        let struct_a_name = String::from("StructA");
        let mut struct_a = StructTemplate::new(struct_a_name.clone());

        let struct_b_name = String::from("StructB");
        let mut struct_b = StructTemplate::new(struct_b_name.clone());

        // insert field_1, field_2, field_3 into both structs
        for i in 1..=3 {
            let field_name = format!("field_{}", i.to_string());
            struct_a.insert_field(field_name.clone(), IntTypeID::U8.as_type());
            struct_b.insert_field(field_name, IntTypeID::U16.as_type());
        }

        // insert field 4 into struct A which contains a struct field of B
        struct_a.insert_field(String::from("field_4"), TypeID::StructType(struct_b_name));

        table.insert_struct(struct_a.clone());
        table.insert_struct(struct_b);

        /* FLATENNING THE STRUCT TEMPLATE */
        let flattened_fields = table.flatten_struct(&struct_a_name);

        assert_eq!(flattened_fields.len(), 6);

        println!("{:?}", flattened_fields);

        for i in 1..=3 {
            let field_name_a = format!(".field_{}", i.to_string());
            let field_name_b = format!(".field_4.field_{}", i.to_string());

            // Assert fields in A
            assert!(flattened_fields.contains(&(field_name_a, IntTypeID::U8.as_type())));

            // Assert nested fields from B
            assert!(flattened_fields.contains(&(field_name_b, IntTypeID::U16.as_type())));
        }
    }
}
