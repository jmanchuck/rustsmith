use super::{
    function::Param,
    types::{BorrowTypeID, TypeID},
};

#[derive(Clone, Debug)]
pub struct Var {
    type_id: TypeID,
    borrow_type: BorrowTypeID,
    name: String,
    is_mut: bool,
}

impl Var {
    pub fn new(type_id: TypeID, name: String, is_mut: bool) -> Self {
        Var {
            type_id,
            borrow_type: BorrowTypeID::None,
            name,
            is_mut,
        }
    }

    pub fn new_with_borrow(
        type_id: TypeID,
        borrow_type: BorrowTypeID,
        name: String,
        is_mut: bool,
    ) -> Self {
        Var {
            type_id,
            borrow_type,
            name,
            is_mut,
        }
    }

    pub fn new_ref(type_id: TypeID, name: String, is_mut: bool) -> Self {
        Var {
            type_id,
            borrow_type: BorrowTypeID::Ref,
            name,
            is_mut,
        }
    }

    pub fn new_mut_ref(type_id: TypeID, name: String, is_mut: bool) -> Self {
        Var {
            type_id,
            borrow_type: BorrowTypeID::MutRef,
            name,
            is_mut,
        }
    }

    pub fn from_param(param: &Param) -> Self {
        Var {
            type_id: param.get_type(),
            borrow_type: param.get_borrow_type(),
            name: param.get_name(),
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

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl ToString for Var {
    fn to_string(&self) -> String {
        self.get_name()
    }
}
