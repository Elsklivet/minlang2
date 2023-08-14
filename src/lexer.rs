use std::{iter::Peekable, str::Chars, fmt::Display};

#[derive(Clone, Debug)]
pub(crate) struct TokenLocation {
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) len: usize,
    pub(crate) span: String,
}

impl<'source> TokenLocation {
    pub(crate) fn new(text: &String, line: usize, col: usize, start: usize, end: usize) -> TokenLocation {
        TokenLocation { line: line, col: col, len: end-start+1, span: text[start..=end].into() }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum TokenKind {
    Table(usize),               // [size]
    Inc,                        // +
    Dec,                        // -
    Mul,                        // *
    Div,                        // /
    MovR,                       // >
    MovL,                       // <
    Print,                      // .
    StartLoop,                  // {
    EndLoop,                    // }
    StartDefine,                // [
    EndDefine,                  // ]
    LParen,                     // (
    RParen,                     // )
    Question,                   // ?
    Goto,                       // @
    Save,                       // $
    PrintAscii,                 // &
    CopyTo,                     // =
    Modulo,                     // %
    FuncDef,                    // :
    FuncCall,                   // ^
    Newline,                    // _
    Eos,                        // ;
    Tilde,                      // ~
    Number(usize),              // numeric
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.clone() {
            Self::Table(num) => return f.write_fmt(format_args!("{}", num)),
            Self::Inc => return f.write_fmt(format_args!("+")),
            Self::Dec => return f.write_fmt(format_args!("-")),
            Self::Mul => return f.write_fmt(format_args!("*")),
            Self::Div => return f.write_fmt(format_args!("/")),
            Self::MovR => return f.write_fmt(format_args!(">")),
            Self::MovL => return f.write_fmt(format_args!("<")),
            Self::Print => return f.write_fmt(format_args!(".")),
            Self::StartLoop => return f.write_fmt(format_args!("{{")),
            Self::EndLoop => return f.write_fmt(format_args!("}}")),
            Self::StartDefine => return f.write_fmt(format_args!("[")),
            Self::EndDefine => return f.write_fmt(format_args!("]")),
            Self::LParen => return f.write_fmt(format_args!("(")),
            Self::RParen => return f.write_fmt(format_args!(")")),
            Self::Question => return f.write_fmt(format_args!("?")),
            Self::Goto => return f.write_fmt(format_args!("@")),
            Self::Save => return f.write_fmt(format_args!("$")),
            Self::PrintAscii => return f.write_fmt(format_args!("&")),
            Self::CopyTo => return f.write_fmt(format_args!("=")),
            Self::Modulo => return f.write_fmt(format_args!("%")),
            Self::FuncDef => return f.write_fmt(format_args!(":")),
            Self::FuncCall => return f.write_fmt(format_args!("^")),
            Self::Newline => return f.write_fmt(format_args!("_")),
            Self::Eos => return f.write_fmt(format_args!(";")),
            Self::Tilde => return f.write_fmt(format_args!("~")),
            Self::Number(num) => return f.write_fmt(format_args!("{}", num)),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) loc: TokenLocation,
}

impl Token {
    pub(crate) fn new(kind: TokenKind, loc: TokenLocation) -> Token {
        Token { kind, loc }
    }
}

pub(crate) struct TokenStream {
    pub(crate) tokens: Vec<Token>,
    curr: usize,
}

impl TokenStream {
    pub(crate) fn new(tokens: Vec<Token>) -> TokenStream {
        TokenStream { tokens, curr: 0 }
    }
}

impl Iterator for TokenStream {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.tokens.len() {
            None
        } else {
            let temp_curr = self.curr;
            self.curr += 1;
            Some(self.tokens.get(temp_curr).unwrap().clone())
        }
    }
}

pub(crate) struct Lexer {
    text: String,
    line: usize,
    col: usize,
    curr_char: usize,
}

#[derive(Debug)]
pub(crate) enum LexError {
    UnexpectedToken(TokenKind, TokenLocation),
    ExpectedToken(TokenKind, TokenLocation),
}

type LexResult<'source> = Result<TokenStream, LexError>;

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.clone() {
            LexError::UnexpectedToken(_, loc) => {
                return f.write_fmt(format_args!(
                    "Unexpected token '{}' at line {} col {}.",
                    loc.span, loc.line, loc.col
                ));
            }
            LexError::ExpectedToken(kind, loc) => {
                return f.write_fmt(format_args!(
                    "Expected token '{}' at line {} col {}, got '{}'.",
                    kind, loc.line, loc.col, loc.span
                ));
            }
        }
    }
}

impl Lexer {
    pub(crate) fn new(text: String) -> Lexer {
        Lexer {
            text: text,
            line: 1,
            col: 1,
            curr_char: 0,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.text.chars().skip(self.curr_char).peekable().peek().copied()
    }

    fn next(&mut self) -> Option<char> {
        if let Some(chr) = self.text.chars().skip(self.curr_char).next() {
            self.curr_char += 1;
            match chr {
                '\n' => { self.line += 1; self.col = 0;}
                _ => { self.col += 1; }
            }
            return Some(chr);
        }

        None
    }

    fn emit(&self, kind: TokenKind) -> Token {
        Token { kind, loc: TokenLocation::new(&self.text, self.line, self.col, self.curr_char-1, self.curr_char-1) }
    }

    fn emit_large(&self, kind: TokenKind, col: usize, from: usize, to: usize) -> Token {
        Token { kind, loc: TokenLocation::new(&self.text, self.line, col, from, to) }
    }

    pub(crate) fn lex(&mut self) -> LexResult {
        let mut tokens: Vec<Token> = Vec::new();

        let table_size: usize = if let Some(char) = self.peek() {
            if char == '[' {
                self.next();
                if let Some(char2) = self.next() {
                    if char2.is_numeric() {
                        // Got a table size                        
                        // Build a string of the size
                        let mut num_string = String::new();
                        num_string.push(char2);

                        while let Some(nchar) = self.next() {
                            if nchar.is_numeric() {
                                num_string.push(nchar);
                            } else if nchar == ']' {
                                break;
                            } else if nchar != ']' {
                                return Err(LexError::ExpectedToken(TokenKind::EndDefine, TokenLocation::new(&self.text, self.line, self.col, self.curr_char, self.curr_char)));
                            }
                        }

                        str::parse::<usize>(&num_string).unwrap_or(256)
                    } else { return Err(LexError::UnexpectedToken(TokenKind::Eos, TokenLocation::new(&self.text, self.line, self.col, self.curr_char, self.curr_char))); }
                } else { 256 }
            } else { 256 }
        } else { 256 };

        // Add table size token.
        tokens.push(Token { kind: TokenKind::Table(table_size), loc: TokenLocation::new(&self.text, 1, 1, 0, self.curr_char - 1) });

        while let Some(char) = self.next() {
            match char {
                '+' => {
                    tokens.push(self.emit(TokenKind::Inc));
                },
                '-' => {
                    tokens.push(self.emit(TokenKind::Dec));
                },
                '*' => {
                    tokens.push(self.emit(TokenKind::Mul));
                },
                '/' => {
                    tokens.push(self.emit(TokenKind::Div));
                },
                '>' => {
                    tokens.push(self.emit(TokenKind::MovR));
                },
                '<' => {
                    tokens.push(self.emit(TokenKind::MovL));
                },
                '.' => {
                    tokens.push(self.emit(TokenKind::Print));
                },
                '{' => {
                    tokens.push(self.emit(TokenKind::StartLoop));
                },
                '}' => {
                    tokens.push(self.emit(TokenKind::EndLoop));
                },
                '[' => {
                    tokens.push(self.emit(TokenKind::StartDefine));
                },
                ']' => {
                    tokens.push(self.emit(TokenKind::EndDefine));
                },
                '(' =>{
                    tokens.push(self.emit(TokenKind::LParen));
                },
                ')' => {
                    tokens.push(self.emit(TokenKind::RParen));
                },
                '?' => {
                    tokens.push(self.emit(TokenKind::Question));
                },
                '@' => {
                    tokens.push(self.emit(TokenKind::Goto));
                },
                '$' => {
                    tokens.push(self.emit(TokenKind::Save));
                },
                '&' => {
                    tokens.push(self.emit(TokenKind::PrintAscii));
                },
                '=' => {
                    tokens.push(self.emit(TokenKind::CopyTo));
                },
                '%' => {
                    tokens.push(self.emit(TokenKind::Modulo));
                },
                ':' => {
                    tokens.push(self.emit(TokenKind::FuncDef));
                },
                '^' => {
                    tokens.push(self.emit(TokenKind::FuncCall));
                },
                '_' => {
                    tokens.push(self.emit(TokenKind::Newline));
                },
                ';' => {
                    tokens.push(self.emit(TokenKind::Eos));
                },
                '~' => {
                    tokens.push(self.emit(TokenKind::Tilde));
                },
                _ => {
                    if char.is_numeric() {
                        // Need to lex a number
                        let start_char = self.curr_char.checked_sub(1).unwrap_or(0);
                        let start_col = self.col;

                        let mut num_string = String::new();
                        num_string.push(char);
                        while let Some(nchar) = self.peek() {
                            if nchar.is_numeric() {
                                self.next();
                                num_string.push(nchar);
                            } else {
                                break;
                            }
                        }

                        tokens.push(Token { 
                            kind: TokenKind::Number(str::parse::<usize>(&num_string).unwrap_or(0)),
                            loc: TokenLocation::new(&self.text, self.line, start_col, start_char, self.curr_char-1)
                        });
                    }
                }
            }
        }

        return Ok(TokenStream { tokens, curr: 0 });
    }
}

#[allow(unused_imports)]
mod tests {
    use super::{Lexer, LexResult, Token, TokenKind, TokenLocation, TokenStream};

    #[test]
    fn test_it_works() {
        let mut lexer = Lexer::new("[1024]\n>>>+-*/(1234)".into());
        let lex_result = lexer.lex();

        assert!(lex_result.is_ok());
        println!("{:?}", lex_result.unwrap().tokens);
    }
}