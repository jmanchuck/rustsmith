use crate::program::{function::FunctionTemplate, types::TypeID};

use super::expr::Expr;

pub struct FunctionCallExpr {
    function_template: FunctionTemplate,
    arguments: Vec<Expr>,
}

impl FunctionCallExpr {
    pub fn new(function_template: FunctionTemplate, arguments: Vec<Expr>) -> Self {
        FunctionCallExpr {
            function_template,
            arguments,
        }
    }

    pub fn get_type(&self) -> TypeID {
        self.function_template.get_type()
    }
}

impl ToString for FunctionCallExpr {
    fn to_string(&self) -> String {
        let mut result: Vec<String> = Vec::new();
        let func_name = self.function_template.get_name();

        // let params: Vec<&Param> = self.function_template.params_iter().collect();

        for i in 0..self.arguments.len() {
            let arg_string = format!(
                "{}",
                // params[i].get_borrow_type().to_string(),
                self.arguments[i].to_string()
            );
            result.push(arg_string);
        }

        format!("{}({})", func_name, result.join(", "))
    }
}

#[cfg(test)]
mod test {
    use crate::program::{expr::int_expr::IntExpr, function::Param, types::IntTypeID};

    use super::*;
    #[test]
    fn correct_string_repr() {
        let return_type = IntTypeID::U128.as_type();
        let func_name = String::from("test_function");

        let param_a = Param::new(String::from("param_a"), IntTypeID::I32.as_type());
        let function_template = FunctionTemplate::new(func_name, vec![param_a], return_type);

        let arg = IntExpr::new_i32(20).as_expr();
        let call_expr = FunctionCallExpr::new(function_template, vec![arg]);

        assert_eq!(call_expr.to_string(), "test_function(20)");
    }
}
