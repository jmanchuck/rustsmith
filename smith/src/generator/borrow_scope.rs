#![allow(dead_code)]
use std::collections::HashSet;

use crate::program::types::BorrowStatus;

#[derive(Clone, Debug)]
pub struct BorrowContext {
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

    pub fn is_mut_borrowed(&self) -> bool {
        self.mut_borrow != None
    }

    pub fn can_mut_borrow(&self) -> bool {
        !self.is_mut_borrowed() && self.borrows.len() == 0
    }

    pub fn mut_borrow(&mut self, borrow_var: &String) -> Option<String> {
        let prev_borrow = self.mut_borrow.clone();
        self.mut_borrow = Some(borrow_var.clone());

        prev_borrow
    }

    pub fn borrow(&mut self, borrow_var: &String) {
        self.borrows.insert(borrow_var.clone());
    }

    pub fn get_mut_borrow(&self) -> Option<String> {
        self.mut_borrow.clone()
    }

    pub fn get_borrows(&self) -> Vec<String> {
        self.borrows.iter().map(|x| x.to_string()).collect()
    }

    pub fn remove_borrow(&mut self, borrow_var: &String) {
        if self.borrows.contains(borrow_var) && self.mut_borrow == Some(borrow_var.to_string()) {
            panic!("The same variable has mutably and immutably borrowed from this variable");
        } else if self.borrows.contains(borrow_var) {
            self.borrows.remove(borrow_var);
        } else if self.mut_borrow == Some(borrow_var.to_string()) {
            self.mut_borrow = None;
        }
    }
}
