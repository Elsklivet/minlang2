use std::collections::BTreeMap;

use crate::ast::*;

pub(crate) const DEFAULT_TABLE_SIZE: usize = 256;

#[derive(Debug)]
pub(crate) struct Table {
    pub(crate) array: Vec<isize>,
    pub(crate) size: usize,
    pub(crate) curr: usize,
    pub(crate) saved: usize,
}

impl std::ops::Index<usize> for Table {
    type Output = isize;

    fn index(&self, index: usize) -> &Self::Output {
        self.array.get(index).expect(format_args!("Index {} is out of bounds for table of size {}.", index, self.array.len()).to_string().as_str())
    }
}

impl std::ops::IndexMut<usize> for Table {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let length = self.array.len();
        self.array.get_mut(index).expect(format_args!("Index {} is out of bounds for table of size {}.", index, length).to_string().as_str())
    }
}

impl Table {
    pub(crate) fn new(size: usize) -> Table {
        let mut array: Vec<isize> = Vec::new();
        array.resize(size, 0isize);
        Table {
            array,
            size,
            curr: 0,
            saved: usize::MAX,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Program {
    pub(crate) statements: Vec<Statement>,
    pub(crate) functions: BTreeMap<usize, Statement>,
    pub(crate) table: Table,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl Program {
    pub(crate) fn new(statements: Vec<Statement>, functions: BTreeMap<usize, Statement>, table: Table) -> Program {
        Program { statements, functions, table, line: 1, col: 1 }
    }
}