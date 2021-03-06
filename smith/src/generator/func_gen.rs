/// Generates function names, parameters, return types
use std::{cell::RefCell, rc::Rc};

use super::{
    consts, context::Context, name_gen::NameGenerator, scope_entry::ScopeEntry,
    scope_entry::StructScopeEntry, stmt_gen::StmtGenerator, struct_gen, struct_gen::StructTable,
};
use crate::program::{
    function::{Function, Param},
    stmt::block_stmt::BlockStmt,
    types::{BorrowTypeID, TypeID},
    var::Var,
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

    fn gen_params<R: Rng>(&self, context: Rc<RefCell<Context>>, rng: &mut R) -> Vec<Param> {
        let mut has_global_struct = false;

        let mut param_name_gen = NameGenerator::new(String::from("param_"));
        let mut param_list: Vec<Param> = Vec::new();

        for _ in 0..self.max_params {
            if rng.gen_range(0.0..1.0) < param_list.len() as f32 / consts::MAX_FUNC_PARAMS as f32 {
                break;
            }

            let param_name = param_name_gen.next().unwrap();
            let mut param = self.gen_param(param_name.clone(), rng);

            while has_global_struct
                && param.get_type() == TypeID::StructType(struct_gen::GLOBAL_STRUCT_NAME.to_owned())
            {
                param = self.gen_param(param_name.clone(), rng);
            }

            if param.get_type() == TypeID::StructType(struct_gen::GLOBAL_STRUCT_NAME.to_owned()) {
                has_global_struct = true;
            }

            let scope_entry: ScopeEntry;

            match param.get_type() {
                TypeID::StructType(struct_name) => {
                    let struct_template =
                        self.struct_table.get_struct_template(&struct_name).unwrap();
                    let struct_scope_entry =
                        StructScopeEntry::from_param(&param, struct_template, self.struct_table);
                    scope_entry = ScopeEntry::Struct(struct_scope_entry);
                }
                TypeID::NullType => panic!("Trying to generate param of null type"),
                _ => {
                    let var = Var::from_param(&param);
                    scope_entry = ScopeEntry::Var(var);
                }
            }

            param_list.push(param.clone());

            match param.get_borrow_type() {
                BorrowTypeID::None => {
                    context
                        .borrow()
                        .scope
                        .borrow_mut()
                        .insert(&param.get_name(), scope_entry);
                }
                BorrowTypeID::Ref => {
                    context.borrow().scope.borrow_mut().insert_borrow(
                        &param.get_name(),
                        scope_entry,
                        &param.get_name(),
                    );
                }
                BorrowTypeID::MutRef => {
                    context.borrow().scope.borrow_mut().insert_borrow(
                        &param.get_name(),
                        scope_entry,
                        &param.get_name(),
                    );
                }
            }
        }

        param_list
    }

    // Returns the generated function and whether or not it is the main function
    pub fn gen_func<R: Rng>(
        &mut self,
        context: Rc<RefCell<Context>>,
        rng: &mut R,
        is_main: bool,
    ) -> Function {
        // Function scope, add params
        context.borrow_mut().enter_scope();

        let params: Vec<Param>;
        let func_name: String;
        let return_type: TypeID;

        if is_main {
            params = Vec::new();
            func_name = String::from("main");
            return_type = TypeID::NullType;
        } else {
            params = self.gen_params(Rc::clone(&context), rng);
            func_name = self.name_gen.next().unwrap();
            return_type = self.struct_table.rand_type_with_null(rng);
        }

        let mut stmt_generator = StmtGenerator::new(self.struct_table);

        let block_stmt: BlockStmt;

        if is_main {
            match self.struct_table.get_global_struct() {
                Some(struct_template) => {
                    block_stmt =
                        stmt_generator.block_stmt_main(Rc::clone(&context), struct_template, rng);
                }
                None => {
                    block_stmt = stmt_generator.block_stmt_with_return(
                        Rc::clone(&context),
                        rng,
                        return_type.clone(),
                    )
                }
            }
        } else {
            block_stmt =
                stmt_generator.block_stmt_with_return(Rc::clone(&context), rng, return_type.clone())
        }

        context.borrow_mut().leave_scope();

        Function::new(func_name, params, return_type, block_stmt)
    }
}
