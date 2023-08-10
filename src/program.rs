use crate::ast::*;

pub(crate) const DEFAULT_TABLE_SIZE: usize = 256;

pub(crate) struct Table {
    array: Vec<isize>,
    curr: usize,
    saved: usize,
}

pub(crate) struct Program {
    statements: Vec<Statement>,
    table: Table,
    line: usize,
    col: usize,
}