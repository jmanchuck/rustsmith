use std::{cell::RefCell, rc::Rc};

use super::{
    main_gen,
    name_gen::NameGenerator,
    scope::{Scope, ScopeEntry},
    stmt_gen::{self, StmtGenerator},
    struct_gen::StructTable,
};
use crate::{
    generator::struct_gen,
    program::{
        expr::expr::RawExpr,
        function::{Function, Param},
        stmt::expr_stmt::ExprStmt,
        types::{BorrowTypeID, TypeID},
        var::Var,
    },
};
use rand::Rng;

pub struct FuncGenerator<'a> {
    struct_table: &'a StructTable, // We only need a immutable reference
    max_params: u32,
    name_gen: NameGenerator,
}

impl<'a> FuncGenerator<'a> {
    pub fn new(struct_table: &'a StructTable, max_params: u32) -> Self {
        FuncGenerator {
            struct_table,
            max_params,
            name_gen: NameGenerator::new(String::from("function_")),
        }
    }

    fn gen_param<R: Rng>(&self, name: String, rng: &mut R) -> Param {
        let rand_type_id = self.struct_table.rand_type_with_global(rng);
        let rand_borrow_type_id: BorrowTypeID = rng.gen();

        if let TypeID::StructType(s) = &rand_type_id {
            if s.eq(struct_gen::GLOBAL_STRUCT_NAME) {
                if rng.gen::<bool>() {
                    return Param::new_ref(name, rand_type_id);
                } else {
                    return Param::new_mut_ref(name, rand_type_id);
                }
            }
        }

        Param::new_with_borrow(name, rand_type_id, rand_borrow_type_id)
    }

    // TODO: Allow it to take global struct once and only once
    fn gen_params<R: Rng>(&self, scope: Rc<RefCell<Scope>>, rng: &mut R) -> Vec<Param> {
        let mut has_global_struct = false;

        let mut param_name_gen = NameGenerator::new(String::from("param_"));
        let mut param_list: Vec<Param> = Vec::new();

        for _ in 0..self.max_params {
            if rng.gen_range(0.0..1.0) < param_list.len() as f32 / main_gen::MAX_FUNC_PARAMS as f32
            {
                break;
            }

            let param_name = param_name_gen.next().unwrap();
            let mut param = self.gen_param(param_name.clone(), rng);

            while has_global_struct
                && param.get_type() == TypeID::StructType(struct_gen::GLOBAL_STRUCT_NAME.to_owned())
            {
                param = self.gen_param(param_name.clone(), rng);
                println!("Regenerating param since attempting to take another global struct");
            }

            if param.get_type() == TypeID::StructType(struct_gen::GLOBAL_STRUCT_NAME.to_owned()) {
                has_global_struct = true;
            }

            let var = Var::from_param(&param);
            param_list.push(param.clone());

            scope
                .borrow_mut()
                .add(param.get_name(), Rc::new(ScopeEntry::Var(var)));
        }

        param_list
    }

    // Returns the generated function and whether or not it is the main function
    pub fn gen_func<R: Rng>(
        &mut self,
        scope: Rc<RefCell<Scope>>,
        rng: &mut R,
        is_main: bool,
    ) -> Function {
        // Function scope
        let function_scope = Rc::new(RefCell::new(Scope::new_from_parent(Rc::clone(&scope))));

        let params: Vec<Param>;
        let func_name: String;
        let return_type: TypeID;

        if is_main {
            params = Vec::new();
            func_name = String::from("main");
            return_type = TypeID::NullType;
        } else {
            params = self.gen_params(Rc::clone(&function_scope), rng);
            func_name = self.name_gen.next().unwrap();
            return_type = self.struct_table.rand_type_with_null(rng);
        }

        let mut stmt_generator = StmtGenerator::new(self.struct_table);

        // Generate block stmt with a return at the end
        let mut block_stmt = stmt_generator.block_stmt_with_return(
            Rc::clone(&function_scope),
            stmt_gen::MAX_STMT_DEPTH,
            rng,
            return_type.clone(),
        );

        if is_main {
            match self.struct_table.get_global_struct() {
                Some(struct_template) => {
                    let global_let_stmt =
                        stmt_generator.static_struct_stmt(struct_template.clone(), scope, rng);
                    block_stmt.push_front(global_let_stmt);

                    let print_serialized = RawExpr::new(format!(
                        "println!(\"{{}}\", (serde_json::to_string(&{}).unwrap()))",
                        struct_gen::GLOBAL_STRUCT_VAR_NAME
                    ))
                    .as_expr();

                    let print_stmt = ExprStmt::new(print_serialized).as_stmt();

                    block_stmt.push(print_stmt);
                }
                None => (),
            }
        }

        Function::new(func_name, params, return_type, block_stmt)
    }
}
