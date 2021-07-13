use super::expr::ArithmeticExpr;

pub enum BoolExpr {
    Bool(BoolValue),
    Binary(Box<BinBoolExpr>),
    Comparison(Box<ComparisonExpr>),
    Negation(Box<NegationExpr>),
}

impl ToString for BoolExpr {
    fn to_string(&self) -> String {
        match self {
            Self::Bool(s) => s.to_string(),
            Self::Binary(s) => (*s).to_string(),
            Self::Comparison(s) => (*s).to_string(),
            Self::Negation(s) => (*s).to_string(),
        }
    }
}

pub struct BoolValue {
    value: bool,
}

impl BoolValue {
    pub fn new(value: bool) -> Self {
        BoolValue { value }
    }
}

impl ToString for BoolValue {
    fn to_string(&self) -> String {
        if self.value {
            String::from("true")
        } else {
            String::from("false")
        }
    }
}

pub struct BinBoolExpr {
    left: BoolExpr,
    right: BoolExpr,
    op: BoolOp,
}

impl ToString for BinBoolExpr {
    fn to_string(&self) -> String {
        format!(
            "{} {} {}",
            self.left.to_string(),
            self.op.to_string(),
            self.right.to_string()
        )
    }
}

pub struct ComparisonExpr {
    left: ArithmeticExpr,
    right: ArithmeticExpr,
    op: ComparisonOp,
}

impl ToString for ComparisonExpr {
    fn to_string(&self) -> String {
        format!(
            "{} {} {}",
            self.left.to_string(),
            self.op.to_string(),
            self.right.to_string()
        )
    }
}

pub struct NegationExpr {
    expr: BoolExpr,
}

impl ToString for NegationExpr {
    fn to_string(&self) -> String {
        format!("!{}", self.expr.to_string())
    }
}

pub enum ComparisonOp {
    Greater,
    Less,
    GreaterEq,
    LessEq,
    Equal,
    NotEqual,
}

impl ToString for ComparisonOp {
    fn to_string(&self) -> String {
        match self {
            Self::Greater => String::from(">"),
            Self::Less => String::from("<"),
            Self::GreaterEq => String::from(">="),
            Self::LessEq => String::from("<="),
            Self::Equal => String::from("=="),
            Self::NotEqual => String::from("!="),
        }
    }
}

pub enum BoolOp {
    OR,
    AND,
}

impl ToString for BoolOp {
    fn to_string(&self) -> String {
        match self {
            Self::AND => String::from("&&"),
            Self::OR => String::from("||"),
        }
    }
}
