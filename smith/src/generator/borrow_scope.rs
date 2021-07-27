#![allow(dead_code)]
use std::collections::{BTreeMap, HashSet};

use crate::program::types::BorrowStatus;

pub struct BorrowManager {
    scopes: Vec<BorrowScope>,
}

impl BorrowManager {
    pub fn new() -> Self {
        BorrowManager {
            scopes: vec![BorrowScope::new()],
        }
    }

    // TODO
    pub fn can_borrow(&self, borrow_source: &String) -> bool {
        return false;
    }

    pub fn enter_scope(&mut self) {
        let copy = self.scopes.last().unwrap().clone();

        self.scopes.push(copy);
    }

    pub fn leave_scope(&mut self) {
        // Pop out the latest scope
        let scope = self.scopes.pop().unwrap();

        // For each variable in scope
        for (var_name, borrow_context) in scope.get_all_entries() {
            // If the variable was instantiated in the removed scope, it shouldn't be in the current scope
            // Remove its borrow if it has a borrow source
            if !self.contains_entry(var_name) && borrow_context.get_borrow_source() != None {
                self.remove_borrow(var_name, &borrow_context.get_borrow_source().unwrap());
            }
        }
    }

    // Decrements the borrow count from source
    fn remove_borrow(&mut self, var_name: &String, borrow_source: &String) {
        // Move from right to left searching for var_name, updating reference when we see
        for i in self.scopes.len()..=0 {
            let scope = self.scopes.get_mut(i).unwrap();

            if scope.contains_entry(borrow_source) {
                scope.get_mut_entry(borrow_source).remove_borrow(var_name);
                break;
            }
        }
    }

    pub fn contains_entry(&self, var_name: &String) -> bool {
        self.scopes.last().unwrap().contains_entry(var_name)
    }

    pub fn insert_entry(&mut self, var_name: String) {
        self.scopes.last_mut().unwrap().insert_entry(var_name, None);
    }

    pub fn insert_borrow_entry(&mut self, var_name: String, borrow_source: String) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert_entry(var_name.clone(), Some(borrow_source.clone()));

        self.borrow_entry(var_name, borrow_source);
    }

    pub fn insert_mut_borrow_entry(&mut self, var_name: String, borrow_source: String) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert_entry(var_name.clone(), Some(borrow_source.clone()));

        self.borrow_mut_entry(var_name, borrow_source);
    }

    fn borrow_entry(&mut self, borrower: String, borrow_source: String) {
        self.scopes
            .last_mut()
            .unwrap()
            .borrow_entry(borrower, borrow_source);
    }

    fn borrow_mut_entry(&mut self, borrower: String, borrow_source: String) {
        self.scopes
            .last_mut()
            .unwrap()
            .borrow_mut_entry(borrower, borrow_source);
    }

    pub fn get_borrowers(&self, borrow_source: &String) -> Vec<String> {
        self.scopes.last().unwrap().get_borrowers(borrow_source)
    }
}

#[derive(Clone)]
struct BorrowScope {
    vars: BTreeMap<String, BorrowContext>,
}

impl BorrowScope {
    fn new() -> Self {
        BorrowScope {
            vars: BTreeMap::new(),
        }
    }

    fn insert_entry(&mut self, var_name: String, borrow_source: Option<String>) {
        let borrow_context = BorrowContext::new(borrow_source);
        self.vars.insert(var_name, borrow_context);
    }

    fn contains_entry(&self, var_name: &String) -> bool {
        self.vars.contains_key(var_name)
    }

    fn get_mut_entry(&mut self, var_name: &String) -> &mut BorrowContext {
        self.vars.get_mut(var_name).unwrap()
    }

    fn borrow_entry(&mut self, borrower: String, borrow_source: String) {
        if !self.vars.contains_key(&borrow_source) {
            panic!(
                "Could not find borrow source in borrow scope: {}",
                borrow_source
            );
        }
        let borrow_source_context = self.vars.get_mut(&borrow_source).unwrap();
        borrow_source_context.borrow(&borrower);

        self.insert_entry(borrower, Some(borrow_source));
    }

    fn borrow_mut_entry(&mut self, borrower: String, borrow_source: String) {
        if !self.vars.contains_key(&borrow_source) {
            panic!(
                "Could not find borrow source in borrow scope: {}",
                borrow_source
            );
        }
        let borrow_source_context = self.vars.get_mut(&borrow_source).unwrap();
        borrow_source_context.borrow_mut(&borrower);

        self.insert_entry(borrower, Some(borrow_source));
    }

    fn get_borrowers(&self, borrow_source: &String) -> Vec<String> {
        self.vars.get(borrow_source).unwrap().get_borrows()
    }

    // Iterator over each variable and their respective borrow source
    fn get_all_entries(&self) -> &BTreeMap<String, BorrowContext> {
        &self.vars
    }
}

#[derive(Clone)]
struct BorrowContext {
    borrow_source: Option<String>, // Could use an Rc<RefCell<data>> to remove itself
    borrows: HashSet<String>,
    mut_borrow: Option<String>,
}

impl BorrowContext {
    pub fn new(source: Option<String>) -> Self {
        BorrowContext {
            borrow_source: source,
            borrows: HashSet::new(),
            mut_borrow: None,
        }
    }

    pub fn get_borrow_status(&self) -> BorrowStatus {
        if let Some(_) = self.mut_borrow {
            BorrowStatus::MutBorrowed
        } else if self.borrows.len() > 0 {
            BorrowStatus::Borrowed
        } else {
            BorrowStatus::None
        }
    }

    pub fn get_borrow_source(&self) -> Option<String> {
        self.borrow_source.clone()
    }

    pub fn can_borrow(&self) -> bool {
        self.mut_borrow == None
    }

    pub fn can_borrow_mut(&self) -> bool {
        self.can_borrow() && self.borrows.len() == 0
    }

    pub fn borrow_mut(&mut self, borrow_var: &String) {
        self.mut_borrow = Some(borrow_var.clone());
    }

    pub fn borrow(&mut self, borrow_var: &String) {
        self.borrows.insert(borrow_var.clone());
    }

    pub fn get_borrows(&self) -> Vec<String> {
        self.borrows.iter().map(|x| x.to_string()).collect()
    }

    fn remove_borrow(&mut self, borrow_var: &String) {
        if self.borrows.contains(borrow_var) && self.mut_borrow == Some(borrow_var.to_string()) {
            panic!("Trying to remove borrow but it is both mutably and immutably borrowed");
        } else if self.borrows.contains(borrow_var) {
            self.borrows.remove(borrow_var);
        } else if self.mut_borrow == Some(borrow_var.to_string()) {
            self.mut_borrow = None;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn removes_borrow_when_out_of_scope() {
        /* This test mimics the following case:

        let a = _;
        {
            let a_ref = &a;
        } <--- a no longer has references pointing to it

        */
        let mut scope = BorrowManager::new();

        scope.insert_entry("a".to_string());
        scope.enter_scope();

        scope.insert_borrow_entry("a_ref".to_string(), "a".to_string());

        assert_eq!(scope.get_borrowers(&"a".to_string()).len(), 1);

        scope.leave_scope();

        assert_eq!(scope.get_borrowers(&"a".to_string()).len(), 0);
    }
}
