use std::iter::Peekable;

use crate::ast::*;
use crate::lexer::*;
use crate::program::DEFAULT_TABLE_SIZE;

#[derive(Debug)]
pub(crate) enum ParseError {
    ExpectedToken(TokenKind, Token),
    ExpectedNumber(Token),
    ExpectedParameter(Token),
    UnexpectedTableToken(Token),
    UnexpectedToken(Token),
    UnexpectedEof(usize, usize),
    ExpectedStatement(usize, usize),
}

type ParseResult<T> = Result<T, ParseError>;

pub(crate) struct Parser {
    tokens: Peekable<TokenStream>,
    line: usize,
    col: usize,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens: TokenStream::new(tokens).peekable(), line: 1, col: 1 }
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        let next = self.tokens.next();

        if next.is_some() { 
            self.line = next.as_ref().unwrap().loc.line;
            self.col = next.as_ref().unwrap().loc.col + next.as_ref().unwrap().loc.len - 1;
        }

        next
    }

    fn expect_number(&mut self) -> ParseResult<usize> {
        if let Some(tok) = self.tokens.next() {
            match tok.kind {
                TokenKind::Number(val) => {
                    Ok(val)
                },
                _ => {
                    Err(ParseError::ExpectedNumber(tok))
                }
            }
        } else {
            Err(ParseError::UnexpectedEof(self.line, self.col))
        }
    }

    fn expect_param(&mut self) -> ParseResult<ParameterKind> {
        if let Some(tok) = self.tokens.next() {
            match tok.kind {
                TokenKind::Number(val) => {
                    Ok(ParameterKind::Numeric(val))
                },
                TokenKind::Save => {
                    Ok(ParameterKind::Saved)
                }
                _ => {
                    Err(ParseError::ExpectedParameter(tok))
                }
            }
        } else {
            Err(ParseError::UnexpectedEof(self.line, self.col))
        }
    }

    fn expect_token(&mut self, kind: TokenKind) -> ParseResult<()> {
        if let Some(tok) = self.tokens.next() {
            if tok.kind == kind {
                self.line = tok.loc.line;
                self.col = tok.loc.col + tok.loc.len - 1;
                Ok(())
            } else {
                Err(ParseError::ExpectedToken(kind, tok))
            }
        } else {
            Err(ParseError::UnexpectedEof(self.line, self.col))
        }
    }

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

    fn parse_table_stmt(&mut self) -> ParseResult<Option<Statement>> {
        // Check if there is a table statement
        let table_stmt = if let Some(tok) = self.tokens.peek() {
            match tok.kind {
                TokenKind::Table(val) => {
                    // Return the statement
                    Some(Statement::new(StatementKind::Table(val), tok.clone()))
                }
                _ => {
                    None
                }
            }
        } else { 
            None
        };

        if table_stmt.is_some() {
            self.next();
        }

        Ok(table_stmt)
    }

    fn parse_copy_stmt(&mut self, token: Token) -> ParseResult<Statement> {
        // Caller ate the equal sign
        // CopyStmt = "=", "(", number | "$", ")" ;

        // "("
        self.expect_token(TokenKind::LParen)?;

        // number | "$"
        let param = self.expect_param()?;

        // ")"
        self.expect_token(TokenKind::RParen)?;

        Ok(Statement::new(StatementKind::Copy(param), token))
    }

    fn parse_goto_stmt(&mut self, token: Token) -> ParseResult<Statement> {
        // Caller ate the @ sign
        // GotoStmt = "@", "(", number | "$", ")" ;

        // "("
        self.expect_token(TokenKind::LParen)?;

        // number | "$"
        let param = self.expect_param()?;

        // ")"
        self.expect_token(TokenKind::RParen)?;

        Ok(Statement::new(StatementKind::Goto(param), token))
    }

    fn parse_defn_stmt(&mut self, token: Token) -> ParseResult<Statement> {
        // Caller ate the "["
        // DefStmt = "[", number, "]" ;
        let num = self.expect_number()?;
        
        // "]"
        self.expect_token(TokenKind::EndDefine)?;

        Ok(Statement::new(StatementKind::Define(num), token))
    }

    fn parse_stmt(&mut self) -> ParseResult<Statement> {
        if let Some(tok) = self.next() {
            match tok.kind {
                TokenKind::Table(size) => { 
                    // Should not encounter a table after the first line
                    return Err(ParseError::UnexpectedTableToken(tok.clone())); 
                },
                TokenKind::Inc => {
                    return Ok(Statement::new(StatementKind::Inc, tok));
                },
                TokenKind::Dec => {
                    return Ok(Statement::new(StatementKind::Dec, tok))
                },
                TokenKind::Mul => {
                    return Ok(Statement::new(StatementKind::Mul, tok));
                },
                TokenKind::Div => {
                    return Ok(Statement::new(StatementKind::Div, tok));
                },
                TokenKind::MovR => {
                    return Ok(Statement::new(StatementKind::MovR, tok));
                },
                TokenKind::MovL => {
                    return Ok(Statement::new(StatementKind::MovL, tok));
                },
                TokenKind::Print => {
                    return Ok(Statement::new(StatementKind::Print, tok));
                },
                TokenKind::StartLoop => {
                    // Need to parse a loop
                    todo!()
                },
                TokenKind::StartDefine => {
                    // Need to parse a define
                    return self.parse_defn_stmt(tok.clone());
                },
                TokenKind::Question => 
                {
                    // Need to parse an if
                    todo!()
                },
                TokenKind::Goto => {
                    // Need to parse a goto
                    return self.parse_goto_stmt(tok.clone());
                },
                TokenKind::Save => {
                    return Ok(Statement::new(StatementKind::Save, tok));
                },
                TokenKind::PrintAscii => {
                    return Ok(Statement::new(StatementKind::PrintAscii, tok));
                },
                TokenKind::CopyTo => {
                    // Need to parse a copy
                    return self.parse_copy_stmt(tok.clone());
                },
                TokenKind::Modulo => {
                    return Ok(Statement::new(StatementKind::Modulo, tok));
                },
                TokenKind::FuncDef => {
                    // Need to parse a func def
                    todo!()
                },
                TokenKind::FuncCall => {
                    // Need to parse a func call
                    todo!()
                },
                TokenKind::Newline => {
                    return Ok(Statement::new(StatementKind::PrintNewline, tok));
                },
                TokenKind::Tilde => {
                    return Ok(Statement::new(StatementKind::FlipSign, tok));
                },
                _ => { 
                    return Err(ParseError::UnexpectedToken(tok));
                 }
            }
        }

        return Err(ParseError::ExpectedStatement(self.line, self.col));
    }
    
}