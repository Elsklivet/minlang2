use std::collections::BTreeMap;

use crate::ast::*;

pub(crate) const DEFAULT_TABLE_SIZE: usize = 256;

#[derive(Debug)]
pub(crate) struct Table {
    array: Vec<isize>,
    curr: usize,
    saved: usize,
}

impl Table {
    pub(crate) fn new(size: usize) -> Table {
        let mut array: Vec<isize> = Vec::new();
        array.resize(size, 0isize);
        Table {
            array,
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