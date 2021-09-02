/// Main entry point for generating a program
use std::cell::RefCell;
use std::rc::Rc;

use crate::program::program::Program;
use rand::Rng;

use super::consts;
use super::context::Context;
use super::func_gen::FuncGenerator;
use super::scope_entry::FuncScopeEntry;
use super::struct_gen::StructTable;

pub fn gen_main<R: Rng>(rng: &mut R) -> String {
    let mut func_count: u8 = 0;

    let mut program = Program::new();

    let context = Rc::new(RefCell::new(Context::new()));

    let mut struct_table = StructTable::new();

    let static_struct_template = struct_table.gen_global_struct(rng);

    program.push_struct_template(static_struct_template);
    loop {
        if rng.gen_range(0.0..1.0) < struct_table.len() as f32 / consts::MAX_STRUCTS as f32 {
            break;
        }
        let struct_template = struct_table.gen_struct(rng);
        program.push_struct_template(struct_template);
    }

    let mut func_gen = FuncGenerator::new(&struct_table, consts::MAX_FUNC_PARAMS);

    loop {
        // generate main on some probability proportional to number of generated funcs vs max (linear)
        let is_main = rng.gen_range(0.0..1.0) < func_count as f32 / consts::MAX_FUNCS as f32;

        let function = func_gen.gen_func(Rc::clone(&context), rng, is_main);

        context.borrow().scope.borrow_mut().insert(
            &function.get_name(),
            FuncScopeEntry::new(function.get_return_type(), function.get_template())
                .as_scope_entry(),
        );

        program.push_function(function);

        func_count += 1;

        if is_main {
            break;
        }
    }

    program.to_string()
}
