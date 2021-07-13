use super::{function::Function, stmt::static_stmt::StaticStmt, struct_template::StructTemplate};

pub struct Program {
    statics: StaticList,
    functions: FunctionList,
    structs: StructList,
}

impl Program {
    pub fn new() -> Self {
        Program {
            statics: StaticList::new(),
            functions: FunctionList::new(),
            structs: StructList::new(),
        }
    }

    pub fn push_static_stmt(&mut self, stmt: StaticStmt) {
        self.statics.push(stmt);
    }

    pub fn push_function(&mut self, function: Function) {
        self.functions.push(function);
    }

    pub fn push_struct_template(&mut self, struct_template: StructTemplate) {
        self.structs.push(struct_template);
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}\n{}\n{}",
            self.statics.to_string(),
            self.structs.to_string(),
            self.functions.to_string(),
        )
    }
}

struct StaticList {
    list: Vec<StaticStmt>,
}

impl StaticList {
    pub fn new() -> Self {
        StaticList { list: Vec::new() }
    }

    pub fn push(&mut self, stmt: StaticStmt) {
        self.list.push(stmt);
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for stmt in self.list.iter() {
            string.push_str(stmt.to_string().as_str());
            string.push('\n');
        }
        string
    }
}

struct FunctionList {
    list: Vec<Function>,
}

impl FunctionList {
    pub fn new() -> Self {
        FunctionList { list: Vec::new() }
    }

    pub fn push(&mut self, function: Function) {
        self.list.push(function);
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for function in self.list.iter() {
            string.push_str(function.to_string().as_str());
            string.push('\n');
        }
        string
    }
}

struct StructList {
    list: Vec<StructTemplate>,
}

impl StructList {
    pub fn new() -> Self {
        StructList { list: Vec::new() }
    }

    pub fn push(&mut self, struct_template: StructTemplate) {
        self.list.push(struct_template);
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for template in self.list.iter() {
            string.push_str(template.to_string().as_str());
            string.push('\n');
        }
        string
    }
}
