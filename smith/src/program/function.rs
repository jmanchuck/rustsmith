use std::fmt;

use super::types::{BorrowTypeID, TypeID};
use crate::program::stmt::block_stmt::BlockStmt;

// Contains enough information to generate and invoke a function call
#[derive(Clone)]
pub struct FunctionTemplate {
    name: String,
    params: Vec<Param>,
    return_type: TypeID,
}

impl FunctionTemplate {
    pub fn new(name: String, params: Vec<Param>, return_type: TypeID) -> Self {
        FunctionTemplate {
            name,
            params,
            return_type,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_type(&self) -> TypeID {
        self.return_type.clone()
    }

    pub fn params_iter(&self) -> std::slice::Iter<Param> {
        self.params.iter()
    }

    pub fn param_list_to_string(&self) -> String {
        self.params
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}

// Main AST representation for a function
pub struct Function {
    block_stmt: BlockStmt,
    function_template: FunctionTemplate,
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<Param>,
        return_type: TypeID,
        block_stmt: BlockStmt,
    ) -> Self {
        Function {
            block_stmt,
            function_template: FunctionTemplate::new(name, params, return_type),
        }
    }

    pub fn get_template(&self) -> FunctionTemplate {
        self.function_template.clone()
    }

    pub fn get_name(&self) -> String {
        self.function_template.name.clone()
    }

    pub fn get_return_type(&self) -> TypeID {
        self.function_template.return_type.clone()
    }

    pub fn get_params(&self) -> Vec<Param> {
        self.function_template.params.to_vec()
    }

    pub fn to_string(&self) -> String {
        let mut return_string = format!("-> {} ", self.function_template.return_type.to_string());
        if self.function_template.return_type == TypeID::NullType {
            return_string = String::from("");
        }

        format!(
            "fn {}({}) {}{}",
            self.get_name(),
            self.function_template.param_list_to_string(),
            return_string,
            self.block_stmt.to_string()
        )
    }
}

impl fmt::Debug for FunctionTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionTemplate")
            .field("Name", &self.name)
            .field("Return Type", &self.return_type)
            .field("Params", &self.param_list_to_string())
            .finish()
    }
}

#[derive(Clone)]
pub struct Param {
    name: String,
    type_id: TypeID,
    borrow_type: BorrowTypeID,
}

impl Param {
    pub fn new(name: String, type_id: TypeID) -> Self {
        Param {
            name,
            type_id,
            borrow_type: BorrowTypeID::None,
        }
    }

    pub fn new_with_borrow(name: String, type_id: TypeID, borrow_type: BorrowTypeID) -> Self {
        Param {
            name,
            type_id,
            borrow_type,
        }
    }

    pub fn new_ref(name: String, type_id: TypeID) -> Self {
        Param {
            name,
            type_id,
            borrow_type: BorrowTypeID::Ref,
        }
    }

    pub fn new_mut_ref(name: String, type_id: TypeID) -> Self {
        Param {
            name,
            type_id,
            borrow_type: BorrowTypeID::MutRef,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_type(&self) -> TypeID {
        self.type_id.clone()
    }

    pub fn get_borrow_type(&self) -> BorrowTypeID {
        self.borrow_type.clone()
    }
}

impl ToString for Param {
    fn to_string(&self) -> String {
        format!(
            "{}: {}{}",
            self.name,
            self.borrow_type.to_string(),
            self.type_id.to_string()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::program::stmt::let_stmt::LetStmt;
    use crate::program::types::IntTypeID;
    use crate::program::var::Var;
    use crate::program::{
        expr::{
            expr::{ArithmeticExpr, Expr},
            int_expr::IntExpr,
        },
        stmt::stmt::Stmt,
    };

    #[test]
    fn has_correct_string_representation() {
        let func_name = String::from("main");
        let param_list = basic_param_list();
        let mut block_stmt = BlockStmt::new();
        let basic_stmt = basic_let_statement();

        block_stmt.push(basic_stmt);

        let block_stmt_string = block_stmt.to_string();

        let func = Function::new(func_name, param_list, TypeID::NullType, block_stmt);

        assert_eq!(
            func.to_string(),
            format!(
                "fn main({}) {}",
                func.function_template.param_list_to_string(),
                block_stmt_string
            )
        )
    }

    #[test]
    fn param_string() {
        let param = Param::new(String::from("a"), IntTypeID::I8.as_type());

        assert_eq!(param.to_string(), "a: i8".to_string());
    }

    fn basic_param_list() -> Vec<Param> {
        let a = Param::new("a".to_string(), TypeID::IntType(IntTypeID::I8));
        let b = Param::new("b".to_string(), TypeID::IntType(IntTypeID::U128));

        vec![a, b]
    }

    fn basic_let_statement() -> Stmt {
        let var = Var::new(IntTypeID::U8.as_type(), String::from("a"), false);

        // Const integer expression
        let expr = IntExpr::new_u8(10);
        let expr = Expr::Arithmetic(ArithmeticExpr::Int(expr));

        LetStmt::new(var, expr).as_stmt()
    }
}
