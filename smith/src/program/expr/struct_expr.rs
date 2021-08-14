use crate::program::{struct_template::StructTemplate, var::Var};

use super::expr::Expr;

pub enum StructExpr {
    Literal(StructLiteral),
    Var(Var),
}

impl StructExpr {
    pub fn as_expr(self) -> Expr {
        Expr::Struct(self)
    }
}

impl ToString for StructExpr {
    fn to_string(&self) -> String {
        match self {
            Self::Literal(s) => s.to_string(),
            Self::Var(s) => s.to_string(),
        }
    }
}

// Expression for instantiation of a struct
pub struct StructLiteral {
    struct_template: StructTemplate,
    field_values: Vec<Expr>,
}

impl StructLiteral {
    pub fn new(struct_template: StructTemplate, field_values: Vec<Expr>) -> Self {
        StructLiteral {
            struct_template,
            field_values,
        }
    }

    pub fn as_struct_expr(self) -> StructExpr {
        StructExpr::Literal(self)
    }
}

impl ToString for StructLiteral {
    fn to_string(&self) -> String {
        let struct_name = self.struct_template.get_name();

        let mut field_args = String::new();

        let field_names: Vec<String> = self
            .struct_template
            .fields_iter()
            .map(|x| x.0.clone())
            .collect();

        for (i, field_name) in field_names.iter().enumerate().take(self.field_values.len()) {
            field_args.push_str(
                format!("{}: {}, ", field_name, self.field_values[i].to_string()).as_str(),
            );
        }

        format!("{} {{{}}}", struct_name, field_args)
    }
}
