#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BorrowTypeID {
    None,
    Ref,
    MutRef,
}

impl BorrowTypeID {
    pub const ALL: &'static [Self] = &[Self::None, Self::Ref, Self::MutRef];

    pub fn as_borrow_status(self) -> BorrowStatus {
        match self {
            BorrowTypeID::None => BorrowStatus::None,
            BorrowTypeID::Ref => BorrowStatus::Borrowed,
            BorrowTypeID::MutRef => BorrowStatus::MutBorrowed,
        }
    }
}

impl ToString for BorrowTypeID {
    fn to_string(&self) -> String {
        match self {
            Self::None => String::from(""),
            Self::Ref => String::from("&"),
            Self::MutRef => String::from("&mut "),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BorrowStatus {
    Borrowed,
    MutBorrowed,
    None,
}

#[derive(PartialEq, Clone, Hash, Eq, Debug)]
pub enum TypeID {
    IntType(IntTypeID),
    StructType(String), // String to denote the struct name
    ArrayType(ArrayTypeID),
    BoolType,
    NullType,
}

/// Discriminants of `TypeID` (hand-written replacement for strum's
/// `EnumDiscriminants` derive).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TypeIDVariants {
    IntType,
    StructType,
    ArrayType,
    BoolType,
    NullType,
}

impl TypeIDVariants {
    pub const ALL: &'static [Self] = &[
        Self::IntType,
        Self::StructType,
        Self::ArrayType,
        Self::BoolType,
        Self::NullType,
    ];
}

impl From<&TypeID> for TypeIDVariants {
    fn from(type_id: &TypeID) -> Self {
        match type_id {
            TypeID::IntType(_) => Self::IntType,
            TypeID::StructType(_) => Self::StructType,
            TypeID::ArrayType(_) => Self::ArrayType,
            TypeID::BoolType => Self::BoolType,
            TypeID::NullType => Self::NullType,
        }
    }
}

impl From<TypeID> for TypeIDVariants {
    fn from(type_id: TypeID) -> Self {
        Self::from(&type_id)
    }
}

impl TypeID {
    pub fn to_string(&self) -> String {
        match self {
            Self::IntType(int_type_id) => int_type_id.to_string(),
            Self::StructType(string) => string.clone(),
            Self::ArrayType(array_type_id) => array_type_id.to_string(),
            &Self::BoolType => String::from("bool"),
            Self::NullType => String::from(""),
        }
    }
}

/// Fixed-size array of integers, e.g. `[i32; 4]`. Phase 1 keeps arrays
/// minimal and sound: int elements only, owned lets only (no borrows, no
/// struct fields, no function params/returns).
#[derive(PartialEq, Clone, Hash, Eq, Copy, Debug)]
pub struct ArrayTypeID {
    pub elem: IntTypeID,
    pub len: usize,
}

impl ArrayTypeID {
    pub fn new(elem: IntTypeID, len: usize) -> Self {
        ArrayTypeID { elem, len }
    }

    pub fn as_type(self) -> TypeID {
        TypeID::ArrayType(self)
    }

    pub fn to_string(&self) -> String {
        format!("[{}; {}]", self.elem.to_string(), self.len)
    }
}

#[derive(PartialEq, Clone, Hash, Eq, Copy, Debug)]
pub enum IntTypeID {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl IntTypeID {
    pub fn to_string(&self) -> String {
        match self {
            Self::I8 => "i8".to_string(),
            Self::I16 => "i16".to_string(),
            Self::I32 => "i32".to_string(),
            Self::I64 => "i64".to_string(),
            Self::I128 => "i128".to_string(),
            Self::U8 => "u8".to_string(),
            Self::U16 => "u16".to_string(),
            Self::U32 => "u32".to_string(),
            Self::U64 => "u64".to_string(),
            Self::U128 => "u128".to_string(),
        }
    }

    pub fn as_type(self) -> TypeID {
        TypeID::IntType(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn array_type_has_correct_string_representation() {
        let array_type = ArrayTypeID::new(IntTypeID::I32, 4);
        assert_eq!(array_type.to_string(), "[i32; 4]");
        assert_eq!(array_type.as_type().to_string(), "[i32; 4]");
    }
}
