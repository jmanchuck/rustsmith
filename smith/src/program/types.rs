use strum_macros::{EnumCount, EnumDiscriminants, EnumIter};

#[derive(Debug, Clone, PartialEq, EnumDiscriminants, EnumCount, EnumIter)]
pub enum BorrowTypeID {
    None,
    Ref,
    MutRef,
}

impl BorrowTypeID {
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

#[derive(PartialEq, Clone, Hash, Eq, EnumDiscriminants, Debug)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(TypeIDVariants))]
#[strum_discriminants(derive(EnumCount, EnumIter))]
pub enum TypeID {
    IntType(IntTypeID),
    StructType(String), // String to denote the struct name
    BoolType,
    NullType,
}

impl TypeID {
    pub fn to_string(&self) -> String {
        match self {
            Self::IntType(int_type_id) => int_type_id.to_string(),
            Self::StructType(string) => string.clone(),
            &Self::BoolType => String::from("bool"),
            Self::NullType => String::from(""),
        }
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
