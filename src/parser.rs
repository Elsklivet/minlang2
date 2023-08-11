use std::collections::BTreeMap;
use std::fmt::Display;
use std::iter::Peekable;

use crate::ast::*;
use crate::lexer::*;
use crate::program::DEFAULT_TABLE_SIZE;
use crate::program::Program;
use crate::program::Table;

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

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ExpectedToken(kind, token) => {
                return f.write_fmt(format_args!("Expected token of type '{}' at line {} col {}, got '{}'.", 
                    kind, token.loc.line, token.loc.col, token.loc.span
                ));
            },
            ParseError::ExpectedNumber(token) => {
                return f.write_fmt(format_args!("Expected a number at line {} col {}, got '{}'.",
                    token.loc.line, token.loc.col, token.loc.span
                ));
            },
            ParseError::ExpectedParameter(token) => {
                return f.write_fmt(format_args!("Expected a number or '$' at line {} col {}, got '{}'.",
                    token.loc.line, token.loc.col, token.loc.span
                ));
            },
            ParseError::UnexpectedTableToken(token) => {
                return f.write_fmt(format_args!("Unexpected table token '{}' at line {} col {}.",
                    token.loc.span, token.loc.line, token.loc.col
                ));
            },
            ParseError::UnexpectedToken(token) => {
                return f.write_fmt(format_args!("Unexpected token '{}' at line {} col {}.",
                    token.loc.span, token.loc.line, token.loc.col
                ));
            },
            ParseError::UnexpectedEof(line, col) => {
                return f.write_fmt(format_args!("Unexpected EOF at line {} col {}.",
                    line, col
                ));
            },
            ParseError::ExpectedStatement(line, col) => {
                return f.write_fmt(format_args!("Expected a statement at line {} col {}.",
                    line, col
                ));
            },
        }
    }
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

    pub(crate) fn peek(&mut self) -> Option<Token> {
        self.tokens.peek().cloned()
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

    fn parse_call_fn_stmt(&mut self, token: Token) -> ParseResult<Statement> {
        // Caller ate the "^"
        // CallFnStmt = "^", "(", number, ")" ;

        self.expect_token(TokenKind::LParen)?;

        let function_id = self.expect_number()?;

        self.expect_token(TokenKind::RParen)?;

        Ok(Statement::new(StatementKind::CallFn(function_id), token))
    }

    fn parse_define_fn_stmt(&mut self, token: Token) -> ParseResult<Statement> {
        // Caller ate the ":"
        // DefFnStmt = ":", "(", number, ")", Block, Eos ;
        // This is not going to be enjoyable

        self.expect_token(TokenKind::LParen)?;

        let function_id = self.expect_number()?;

        self.expect_token(TokenKind::RParen)?;

        let mut stmts: Vec<Statement> = Vec::new();

        // Parse a whole bunch of statements
        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Eos => {
                    // End of the function
                    // Break and expect EOS.
                    break;
                },
                _ => {
                    stmts.push(self.parse_stmt()?);
                }
            }
        }

        self.expect_token(TokenKind::Eos)?;

        Ok(Statement::new(StatementKind::DefineFn(function_id, stmts), token.clone()))
    }

    fn parse_loop_stmt(&mut self, token: Token) -> ParseResult<Statement> {
        // Caller ate "{"
        // LoopStmt = "{", Block, "}", ["(", number | "$", ")"] ;

        let mut stmts: Vec<Statement> = Vec::new();

        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::EndLoop => {
                    // End of the loop
                    // Break and expect EOS.
                    break;
                },
                _ => {
                    stmts.push(self.parse_stmt()?);
                }
            }
        }

        self.expect_token(TokenKind::EndLoop)?;

        // Check if the next token is a paren, in which case, expect number
        let param = if let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::LParen => {
                    // Eat paren
                    self.expect_token(TokenKind::LParen)?;

                    let temp_param = self.expect_param()?;

                    self.expect_token(TokenKind::RParen)?;

                    Some(temp_param)
                },
                _ => {
                    // Do nothing
                    None
                }
            }
        } else { None };

        Ok(Statement::new(StatementKind::Loop(stmts, param), token))
    }

    fn parse_if_stmt(&mut self, token: Token) -> ParseResult<Statement> {
        // Caller ate "?"
        // IfStmt = "?", "(", number | "$", ")", Block, Eos ;

        self.expect_token(TokenKind::LParen)?;

        let param = self.expect_param()?;

        self.expect_token(TokenKind::RParen)?;

        // Parse a block
        let mut stmts: Vec<Statement> = Vec::new();

        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Eos => {
                    // End of the if block
                    // Break and expect EOS.
                    break;
                },
                _ => {
                    stmts.push(self.parse_stmt()?);
                }
            }
        }

        self.expect_token(TokenKind::Eos)?;

        Ok(Statement::new(StatementKind::If(param, stmts), token.clone()))
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
                    return self.parse_loop_stmt(tok.clone());
                },
                TokenKind::StartDefine => {
                    // Need to parse a define
                    return self.parse_defn_stmt(tok.clone());
                },
                TokenKind::Question => 
                {
                    // Need to parse an if
                    return self.parse_if_stmt(tok.clone());
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
                    return self.parse_define_fn_stmt(tok.clone());
                },
                TokenKind::FuncCall => {
                    // Need to parse a func call
                    return self.parse_call_fn_stmt(tok.clone());
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
    

    pub(crate) fn parse_program(&mut self) -> ParseResult<Program> {
        let mut statements: Vec<Statement> = Vec::new();
        let mut functions: BTreeMap<usize, Statement> = BTreeMap::new();
        
        // Start by parsing the table
        let wrapped_table = self.parse_table_stmt()?;

        // Create the table out of the statement. Discard the statement thereafter, it is useless.
        let table = if let Some(table_stmt) = wrapped_table {
            let table_kind = table_stmt.kind;
            match table_kind {
                StatementKind::Table(size) => {
                    Table::new(size)
                },
                _ => {
                    Table::new(DEFAULT_TABLE_SIZE)
                }
            }
        } else { 
            Table::new(DEFAULT_TABLE_SIZE)
        };

        // Continually parse statements
        while let Some(_) = self.peek() {
            let stmt = self.parse_stmt()?;

            // Some statements need special parsing
            match stmt.kind {
                StatementKind::DefineFn(id, _) => {
                    functions.insert(id, stmt.clone());
                },
                _ => {
                    // Do nothing.
                }
            }

            // Add to global vector of statements
            statements.push(stmt);
        }

        Ok(Program::new(statements, functions, table))
    }
}

#[allow(unused_imports)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_it_works() {
        let mut lexer = Lexer::new("[32]\n:(0)++>?(0)-<;;:(1)&_._;++^(0)^(1)>++^(1)****^(1)".into());
        let lex_result = lexer.lex();

        assert!(lex_result.is_ok());

        let mut parser = Parser::new(lex_result.unwrap().tokens);

        let prog_result = parser.parse_program();

        println!("{:?}", prog_result.ok().unwrap());
    }
}