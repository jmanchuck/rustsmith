use crate::program::{types::TypeID, var::Var};

use super::expr::Expr;

pub struct ArrayExpr {
    type_id: TypeID,
    count: u32,
    elements: Vec<Expr>,
}

impl ArrayExpr {
    pub fn new(type_id: TypeID, count: u32, elements: Vec<Expr>) -> Self {
        ArrayExpr {
            type_id,
            count,
            elements,
        }
    }

    pub fn get_type(&self) -> TypeID {
        self.type_id.clone()
    }

    pub fn len(&self) -> u32 {
        self.count
    }
}

impl ToString for ArrayExpr {
    fn to_string(&self) -> String {
        let initializer = self
            .elements
            .iter()
            .map(|expr| expr.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        format!("[{}]", initializer)
    }
}

pub struct ArrayIndexExpr {
    index: u32,
    var: Var,
}

impl ArrayIndexExpr {
    pub fn new(index: u32, var: Var) -> Self {
        ArrayIndexExpr { index, var }
    }
}

impl ToString for ArrayIndexExpr {
    fn to_string(&self) -> String {
        format!("{}[{}]", self.var.to_string(), self.index)
    }
}
