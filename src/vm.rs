use crate::program::{Program, Table};
use crate::ast::{Statement, StatementKind};

pub(crate) struct Vm {
    pub(crate) program: Program,
}

impl Vm {
    pub(crate) fn new(program: Program) -> Vm {
        Vm { program }
    }

    fn callfn(&mut self, defn: &Statement) {
        match defn.kind.clone() {
            StatementKind::DefineFn(_, stmts) => {
                for stmt in stmts {
                    self.execute(&stmt);
                }
            },
            _ => {
                // No?
                unreachable!();
            }
        }
    }

    pub(crate) fn execute(&mut self, statement: &Statement) {
        match statement.kind.clone() {
            crate::ast::StatementKind::Inc => {
                // Increment current table cell
                let curr = self.program.table.curr;
                self.program.table[curr] = self.program.table[curr].checked_add(1).unwrap_or(self.program.table[curr]);
            },
            crate::ast::StatementKind::Dec => {
                // Decrement current table cell
                let curr = self.program.table.curr;
                self.program.table[curr] = self.program.table[curr].checked_sub(1).unwrap_or(self.program.table[curr]);
            },
            crate::ast::StatementKind::Mul => {
                // Double current table cell
                let curr = self.program.table.curr;
                self.program.table[curr] = self.program.table[curr].checked_mul(2).unwrap_or(self.program.table[curr]);
            },
            crate::ast::StatementKind::Div => {
                // Halve current table cell
                let curr = self.program.table.curr;
                self.program.table[curr] = self.program.table[curr].checked_div(2).unwrap_or(self.program.table[curr]);
            },
            crate::ast::StatementKind::MovR => {
                // Move table value to the right if possible
                if self.program.table.curr < self.program.table.size.checked_sub(1).unwrap_or(0) {
                    self.program.table.curr = self.program.table.curr + 1;
                }
            },
            crate::ast::StatementKind::MovL => {
                // Move table value to the left if possible
                if self.program.table.curr > 0 {
                    self.program.table.curr = self.program.table.curr.checked_sub(1).unwrap_or(self.program.table.curr);
                }
            },
            crate::ast::StatementKind::Print => {
                let curr = self.program.table.curr;
                print!("{}", self.program.table[curr]);
            },
            crate::ast::StatementKind::Loop(stmts, cndt) => {
                let condition_param = cndt.unwrap_or(crate::ast::ParameterKind::Numeric(0));
                let condition = match condition_param {
                    crate::ast::ParameterKind::Numeric(val) => val,
                    crate::ast::ParameterKind::Saved => self.program.table[self.program.table.saved] as usize
                };

                let mut check_value = self.program.table[self.program.table.curr];
                while check_value != condition as isize {
                    for stmt in &stmts {
                        self.execute(stmt);
                    }
                    check_value = self.program.table[self.program.table.curr];
                }
            },
            crate::ast::StatementKind::Define(val) => {
                let curr = self.program.table.curr;
                self.program.table[curr] = val as isize;
            },
            crate::ast::StatementKind::If(condition_param, stmts) => {
                let condition = match condition_param {
                    crate::ast::ParameterKind::Numeric(val) => val,
                    crate::ast::ParameterKind::Saved => self.program.table[self.program.table.saved] as usize
                };
                
                if self.program.table[self.program.table.curr] == condition as isize {
                    for stmt in &stmts {
                        self.execute(stmt);
                    }
                }
            },
            crate::ast::StatementKind::Goto(param) => {
                match param {
                    crate::ast::ParameterKind::Numeric(idx) => {
                        self.program.table.curr = idx;
                    },
                    crate::ast::ParameterKind::Saved => {
                        self.program.table.curr = self.program.table.saved;
                    },
                }
            },
            crate::ast::StatementKind::Save => {
                self.program.table.saved = self.program.table.curr;
            }
            crate::ast::StatementKind::PrintAscii => {
                print!("{}", self.program.table[self.program.table.curr] as u8 as char);
            },
            crate::ast::StatementKind::Copy(param) => {
                let curr = self.program.table.curr;
                match param {
                    crate::ast::ParameterKind::Numeric(idx) => {
                        self.program.table[curr] = self.program.table[idx];
                    },
                    crate::ast::ParameterKind::Saved => {
                        self.program.table[curr] = self.program.table[self.program.table.saved];
                    },
                }
            },
            crate::ast::StatementKind::Modulo => {
                // Mod current table cell by 2
                let curr = self.program.table.curr;
                self.program.table[curr] = self.program.table[curr] % 2;
            },
            crate::ast::StatementKind::CallFn(id) => {
                // There is no semantics phase!!! Hope your ID is valid!
                let funcdef = self.program.functions.get(&id).unwrap().clone();
                self.callfn(&funcdef);
            },
            crate::ast::StatementKind::PrintNewline => {
                print!("\n");
            },
            crate::ast::StatementKind::FlipSign => {
                let curr = self.program.table.curr;
                self.program.table[curr] *= -1;
            },
            _ => {
                // Unknown statement kind, do nothing.
            }
        }
    }

    pub(crate) fn run(&mut self, show_registers: bool) {
        // Iterate over all statements of the program and run them
        let mut pc = 0usize;
        while pc < self.program.statements.len() {
            let statement = self.program.statements.get(pc).unwrap().clone();
            self.execute(&statement);
            pc += 1;
            // println!("{:?} / {:?} ", pc, self.program.statements.len());
        }

        if show_registers {
            println!("{:?}", self.program.table.array);
        }
    }
}