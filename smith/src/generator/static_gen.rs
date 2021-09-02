#![allow(dead_code)]
/// Generates global static variables
/// Currently unused since global static variables are not allowed
/// This should be modified to use Mutex wrappers
use std::{cell::RefCell, rc::Rc};

use rand::Rng;

use crate::program::stmt::static_stmt::StaticStmt;

use super::scope_entry::VarScopeEntry;
use super::{expr_gen::ExprGenerator, name_gen::NameGenerator, scope::Scope};

pub struct StaticGenerator {
    name_gen: NameGenerator,
}

impl StaticGenerator {
    pub fn new() -> Self {
        StaticGenerator {
            name_gen: NameGenerator::new(String::from("VAR")),
        }
    }

    pub fn gen_static<R: Rng>(&mut self, scope: Rc<RefCell<Scope>>, rng: &mut R) -> StaticStmt {
        let static_int = ExprGenerator::int32(rng);
        let var_name = self.name_gen.next().unwrap();
        scope.borrow_mut().insert(
            &var_name,
            VarScopeEntry::new(static_int.get_type(), var_name.clone(), false).as_scope_entry(),
        );

        StaticStmt::new(var_name, static_int.get_type(), static_int.as_expr())
    }
}
