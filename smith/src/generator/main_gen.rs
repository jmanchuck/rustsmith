use std::cell::RefCell;
use std::rc::Rc;

use crate::program::program::Program;
use rand::Rng;

use super::func_gen::FuncGenerator;
use super::scope::{FuncScopeEntry, Scope};
use super::struct_gen::StructTable;

pub const MAX_STATICS: u32 = 2;
pub const MAX_STRUCTS: u32 = 2;
pub const MAX_FUNCS: u32 = 10;
pub const MAX_FUNC_PARAMS: u32 = 12;

pub fn gen_main<R: Rng>(rng: &mut R) -> String {
    let mut func_count: u8 = 0;

    let mut program = Program::new();

    let global_scope = Scope::new();
    let current_scope = Rc::new(RefCell::new(global_scope));

    let mut struct_table = StructTable::new();

    let static_struct_template = struct_table.gen_global_struct();

    program.push_struct_template(static_struct_template);
    loop {
        if rng.gen_range(0.0..1.0) < struct_table.len() as f32 / MAX_STRUCTS as f32 {
            break;
        }
        let struct_template = struct_table.gen_struct(rng);
        program.push_struct_template(struct_template);
    }

    // let mut static_gen = StaticGenerator::new();

    // loop {
    //     // break on some probability proportional to number of generated statics vs max (linear)
    //     if rng.gen_range(0.0..1.0) < static_count as f32 / MAX_STATICS as f32 {
    //         break;
    //     }
    //     let static_stmt = static_gen.gen_static(Rc::clone(&current_scope), rng);
    //     program.push_static_stmt(static_stmt);

    //     static_count += 1;
    // }

    let mut func_gen = FuncGenerator::new(&struct_table, MAX_FUNC_PARAMS);

    loop {
        // generate main on some probability proportional to number of generated funcs vs max (linear)
        let is_main = rng.gen_range(0.0..1.0) < func_count as f32 / MAX_FUNCS as f32;

        let function = func_gen.gen_func(Rc::clone(&current_scope), rng, is_main);

        current_scope.borrow_mut().add(
            function.get_name(),
            Rc::new(
                FuncScopeEntry::new(function.get_return_type(), function.get_template())
                    .as_scope_entry(),
            ),
        );

        program.push_function(function);

        func_count += 1;

        if is_main {
            break;
        }
    }

    program.to_string()
}
