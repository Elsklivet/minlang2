// digit = "0".."9" ;
// number = { digit } ;

// TableStmt = "[", number, "]" ;
// IncStmt = "+" ;
// DecStmt = "-" ;
// MulStmt = "*" ;
// DivStmt = "/" ;
// PrintStmt = "." ; 
// MovRightStmt = ">" ;
// MovLeftStmt = "<" ;
// LoopStmt = "{", Block, "}", ["(", number | "$", ")"] ;
// DefStmt = "[", number, "]" ;
// IfStmt = "?", "(", number | "$", ")", Block, Eos ;
// GotoStmt = "@", "(", number | "$", ")" ;
// SaveStmt = "$" ;
// PrintAsciiStmt = "&" ;
// CopyStmt = "=", "(", number | "$", ")" ;
// ModuloStmt = "%" ;
// DefFnStmt = ":", "(", number, ")", Block, Eos ;
// CallFnStmt = "^", "(", number, ")" ;
// PrintNewlStmt = "_" ;
// FlipStmt = "~" ;
// Eos = ";" ;

// SourceFile = TableStmt, "\n" | "\r", Block ;
// Block = { Statement } ;
// Statement = TableStmt | IncStmt | DecStmt | MulStmt | DivStmt | PrintStmt | MovRightStmt | MovLeftStmt | LoopStmt | FlipStmt
//             | DefStmt | IfStmt | GotoStmt | SaveStmt | PrintAsciiStmt | CopyStmt | ModuloStmt | DefFnStmt | CallFnStmt | PrintNewlStmt ;

use crate::lexer::Token;

#[derive(Debug)]
pub(crate) enum ParameterKind {
    Numeric(usize),
    Saved,
}

#[derive(Debug)]
pub(crate) enum StatementKind {
    Table(usize),
    Inc,
    Dec,
    Mul,
    Div,
    MovR,
    MovL,
    Print,
    Loop(Vec<Statement>, Option<ParameterKind>),
    Define(usize),
    If(ParameterKind, Vec<Statement>),
    Goto(ParameterKind),
    Save,
    PrintAscii,
    Copy(ParameterKind),
    Modulo,
    DefineFn(usize, Vec<Statement>),
    CallFn(usize),
    PrintNewline,
    FlipSign,
}

#[derive(Debug)]
pub(crate) struct Statement {
    pub(crate) kind: StatementKind,
    pub(crate) token: Token,
}

impl Statement {
    pub(crate) fn new(kind: StatementKind, token: Token) -> Statement {
        Statement { kind, token }
    }
}