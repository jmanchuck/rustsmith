use crate::program::types::{ArrayTypeID, TypeID};

use super::arithmetic_expr::ArithmeticExpr;
use super::expr::Expr;

/// RHS of an array let/assign: either a list literal `[e0, e1, ..]` or a
/// repeat literal `[e; LEN]`. Elements are int-typed arithmetic expressions,
/// already emitted in their safe (wrapping/checked, parenthesized) forms.
pub enum ArrayExpr {
    Literal(ArrayLiteralExpr),
    Repeat(ArrayRepeatExpr),
}

impl ArrayExpr {
    pub fn as_expr(self) -> Expr {
        Expr::Array(self)
    }

    pub fn get_type(&self) -> TypeID {
        match self {
            Self::Literal(s) => s.get_type(),
            Self::Repeat(s) => s.get_type(),
        }
    }
}

impl ToString for ArrayExpr {
    fn to_string(&self) -> String {
        match self {
            Self::Literal(s) => s.to_string(),
            Self::Repeat(s) => s.to_string(),
        }
    }
}

pub struct ArrayLiteralExpr {
    array_type: ArrayTypeID,
    elements: Vec<ArithmeticExpr>,
}

impl ArrayLiteralExpr {
    pub fn new(array_type: ArrayTypeID, elements: Vec<ArithmeticExpr>) -> Self {
        assert_eq!(
            elements.len(),
            array_type.len,
            "Array literal element count must match the array type length"
        );
        ArrayLiteralExpr {
            array_type,
            elements,
        }
    }

    pub fn as_array_expr(self) -> ArrayExpr {
        ArrayExpr::Literal(self)
    }

    pub fn get_type(&self) -> TypeID {
        self.array_type.as_type()
    }
}

impl ToString for ArrayLiteralExpr {
    fn to_string(&self) -> String {
        let elements = self
            .elements
            .iter()
            .map(|expr| expr.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        format!("[{}]", elements)
    }
}

pub struct ArrayRepeatExpr {
    array_type: ArrayTypeID,
    element: ArithmeticExpr,
}

impl ArrayRepeatExpr {
    pub fn new(array_type: ArrayTypeID, element: ArithmeticExpr) -> Self {
        ArrayRepeatExpr {
            array_type,
            element,
        }
    }

    pub fn as_array_expr(self) -> ArrayExpr {
        ArrayExpr::Repeat(self)
    }

    pub fn get_type(&self) -> TypeID {
        self.array_type.as_type()
    }
}

impl ToString for ArrayRepeatExpr {
    // The element is parenthesized so any expression form (method chains,
    // casts) reads unambiguously before the `; LEN` separator.
    fn to_string(&self) -> String {
        format!("[({}); {}]", self.element.to_string(), self.array_type.len)
    }
}

/// Guarded array element read: `arr[((idx) as usize) % LEN]`.
///
/// The `% LEN` guard makes any index expression in-bounds: LEN is a nonzero
/// literal (so the modulo can never divide by zero) and an `as usize` cast of
/// any integer is fully defined (truncation). This is what exercises
/// bounds-check elision in the compiler under test — the guard proves the
/// access in-bounds, so the panic path is elidable at every opt level.
pub struct ArrayIndexExpr {
    array_name: String,
    array_type: ArrayTypeID,
    index: ArithmeticExpr,
}

impl ArrayIndexExpr {
    pub fn new(array_name: String, array_type: ArrayTypeID, index: ArithmeticExpr) -> Self {
        ArrayIndexExpr {
            array_name,
            array_type,
            index,
        }
    }

    pub fn as_arith_expr(self) -> ArithmeticExpr {
        ArithmeticExpr::ArrayIndex(Box::new(self))
    }

    pub fn get_type(&self) -> TypeID {
        self.array_type.elem.as_type()
    }
}

impl ToString for ArrayIndexExpr {
    // The cast stays inside the brackets and both the index expression and
    // the cast are parenthesized (`as` has tricky precedence, same reasoning
    // as CastExpr). Any block expression inside the index (e.g. checked-div)
    // is already parenthesized by its own emitter.
    fn to_string(&self) -> String {
        format!(
            "{}[(({}) as usize) % {}]",
            self.array_name,
            self.index.to_string(),
            self.array_type.len
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::program::expr::arithmetic_expr::IntExpr;
    use crate::program::types::IntTypeID;

    #[test]
    fn array_literal_has_correct_string_representation() {
        let array_type = ArrayTypeID::new(IntTypeID::I32, 3);
        let elements = vec![
            IntExpr::new_i32(1).as_arith_expr(),
            IntExpr::new_i32(2).as_arith_expr(),
            IntExpr::new_i32(3).as_arith_expr(),
        ];
        let literal = ArrayLiteralExpr::new(array_type, elements);

        assert_eq!(literal.get_type(), array_type.as_type());
        assert_eq!(literal.to_string(), "[1i32, 2i32, 3i32]");
    }

    #[test]
    fn array_repeat_has_correct_string_representation() {
        let array_type = ArrayTypeID::new(IntTypeID::U8, 5);
        let repeat = ArrayRepeatExpr::new(array_type, IntExpr::new_u8(7).as_arith_expr());

        assert_eq!(repeat.get_type(), array_type.as_type());
        assert_eq!(repeat.to_string(), "[(7u8); 5]");
    }

    #[test]
    fn array_index_read_is_modulo_guarded() {
        let array_type = ArrayTypeID::new(IntTypeID::I64, 4);
        let index = IntExpr::new_i8(-3).as_arith_expr();
        let read = ArrayIndexExpr::new("var_0".to_string(), array_type, index);

        assert_eq!(read.get_type(), IntTypeID::I64.as_type());
        assert_eq!(read.to_string(), "var_0[((-3i8) as usize) % 4]");
    }
}
