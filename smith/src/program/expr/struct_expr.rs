use crate::program::{struct_template::StructTemplate, var::Var};

use super::expr::{Expr, LiteralExpr};
use strum_macros::{EnumCount, EnumDiscriminants, EnumIter};

// Used to represent the construction of a struct given a template
#[derive(EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(StructExprVariants))]
#[strum_discriminants(derive(EnumCount, EnumIter))]
pub enum StructExpr {
    Literal(StructLiteral),
    Var(Var),
}

impl StructExpr {
    pub fn as_expr(self) -> Expr {
        Expr::Literal(LiteralExpr::Struct(self))
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
}

impl ToString for StructLiteral {
    fn to_string(&self) -> String {
        let struct_name = String::from(self.struct_template.get_name());

        let mut field_args = String::new();

        let field_names: Vec<String> = self
            .struct_template
            .fields_iter()
            .map(|x| x.0.clone())
            .collect();

        for i in 0..self.field_values.len() {
            field_args.push_str(
                format!("{}: {}, ", field_names[i], self.field_values[i].to_string()).as_str(),
            );
        }

        format!("{} {{{}}}", struct_name, field_args)
    }
}
