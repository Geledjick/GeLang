#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Data
    Ident(String),
    StrData(String),
    FloatNum(f64),
    IntNum(i128),
    CharData(char),
    BoolData(bool),
    
    // Types
    Usize, Isize,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64, F128,
    Byte,
    Bool, Bits,
    Char, Str,
    
    // Operators
    Plus, Minus, Mul, Div,
    PlusEq, MinusEq, MulEq, DivEq,
    PlusPlus, MinusMinus,
    Eq,
    DotDot,
    Lt, Gt, LtEq, GtEq, EqEq, BangEq,
    Amp, AmpAmp, Pipe, PipePipe,
    Bang,
    
    // Delimiters
    LParen, RParen,
    LBrace, RBrace,
    Semicolon, Colon, Comma,
    
    // Keywords
    Let, Mut,

    // functions
    Pub, Inline, 
    Fn, Return, 

    // conditions
    If, Else, Elif,
    Match,
    Emit,

    // loops
    While, Loop,
    For, In,
    Break, Continue,

    Arrow,
    
    // Meta
    Eof,
}

#[derive(Debug)]
pub struct LexError {
    pub msg: String,
    pub line: u32,
    pub col: u32,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Lex error at pos {}:{}: {}", self.line, self.col, self.msg)
    }
}

pub type LexResult = Result<Token, LexError>;

pub struct Lexer<'a> {
    input: &'a str,
    pos:   usize,
    pub line: u32,
    pub col: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0, line: 1, col: 1 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8(); // for non-ascii

        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        Some(c)
    }
    
    fn skip_whitespace(&mut self) {
        while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
            self.advance();
        }
    }

    fn skip_to_new_line(&mut self) {
        while self.peek().map(|c| c != '\n').unwrap_or(false) {
            self.advance();
        }
    }

    fn skip_to_next(&mut self) -> LexResult {
        loop {
            match self.peek() {
                None => return Err(LexError {
                    msg: "Unterminated block comment".into(),
                    line: self.line,
                    col: self.col,
                }),
                Some('*') => {
                    self.advance();
                    if self.peek() == Some('/') {
                        self.advance();
                        break;
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        self.next_token()
    }

    fn read_number(&mut self, first: char) -> LexResult {
        let start = self.pos - first.len_utf8();
        let mut has_dot = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '_' {
                self.advance();
            } else if c == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let s = &self.input[start..self.pos];
        if has_dot {
            s.parse::<f64>()
                .map(Token::FloatNum)
                .map_err(|_| LexError { msg: format!("Invalid float number: {s}"), line: self.line, col: self.col })
        } else {
            s.parse::<i128>()
                .map(Token::IntNum)
                .map_err(|_| LexError { msg: format!("Invalid int number: {s}"), line: self.line, col: self.col })
        }
    }

    fn read_ident(&mut self, first: char) -> Token {
        let start = self.pos - first.len_utf8();

        while self.peek().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
            self.advance();
        }

        match &self.input[start..self.pos] {
            "let" => Token::Let,
            "mut" => Token::Mut,

            "fn" => Token::Fn,
            "return" => Token::Return,

            "if" => Token::If,
            "elif" => Token::Elif,
            "else" => Token::Else,

            "while" => Token::While,
            "loop" => Token::Loop,

            "true" => Token::BoolData(true),
            "false" => Token::BoolData(false),

            "usize" => Token::Usize,
            "isize" => Token::Isize,
            "u64" => Token::U64,
            "f128" => Token::F128,
            "byte" => Token::Byte,
            "bits" => Token::Bits,
            
            "pub" => Token::Pub,
            "inline" => Token::Inline,
            "for" => Token::For,
            "in" => Token::In,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "match" => Token::Match,
            "switch" => Token::Match,
            "emit" => Token::Emit,
            
            "i8" => Token::I8,
            "i16" => Token::I16,
            "i32" => Token::I32,
            "i64" => Token::I64,

            "u8" => Token::U8,
            "u16" => Token::U16,
            "u32" => Token::U32,

            "f32" => Token::F32,
            "f64" => Token::F64,

            "bool" => Token::Bool,

            "char" => Token::Char,
            "str" => Token::Str,
            
            s => Token::Ident(s.to_string()),
        }
    }

    fn read_str(&mut self) -> LexResult {
        let start = self.pos;
        
        loop {
            match self.peek() {
                None => return Err(LexError {
                    msg: "Unterminated string".into(),
                    line: self.line, 
                    col: self.col
                }),
                Some('"') => break,
                _ => {
                    self.advance();
                }
            }
        }

        self.advance(); // skip closing "

        Ok(Token::StrData(self.input[start..(self.pos - 1)].to_string()))
    }
    
    pub fn next_token(&mut self) -> LexResult {
        self.skip_whitespace();

        let c = match self.advance() {
            Some(c) => c,
            None    => return Ok(Token::Eof),
        };

        match c {
            '+' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(Token::PlusEq)
                }
                Some('+') => {
                    self.advance();
                    Ok(Token::PlusPlus)
                }
                _ => Ok(Token::Plus)
            },
            '-' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(Token::MinusEq)
                }
                Some('-') => {
                    self.advance();
                    Ok(Token::MinusMinus)
                }
                _ => Ok(Token::Minus)
            },
            '*' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(Token::MulEq)
                }
                _ => Ok(Token::Mul)
            },
            '/' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(Token::DivEq)
                },
                Some('/') => {
                    self.skip_to_new_line();
                    self.next_token()
                },
                Some('*') => {
                    self.advance();
                    self.skip_to_next()
                }
                _ => Ok(Token::Div)
            },
            '<' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(Token::LtEq)
                }
                _ => Ok(Token::Lt)
            },
            '>' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(Token::GtEq)
                }
                _ => Ok(Token::Gt)
            },
            '&' => match self.peek() {
                Some('&') => {
                    self.advance();
                    Ok(Token::AmpAmp)
                }
                _ => Ok(Token::Amp)
            },
            '|' => match self.peek() {
                Some('|') => {
                    self.advance();
                    Ok(Token::PipePipe)
                }
                _ => Ok(Token::Pipe)
            },
            '.' => match self.peek() {
                Some('.') => {
                    self.advance();
                    Ok(Token::DotDot)
                }
                _ => Err(LexError {
                    msg: "Unexpected char: '.'".into(),
                    line: self.line,
                    col: self.col
                })
            }
            
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '{' => Ok(Token::LBrace),
            '}' => Ok(Token::RBrace),
            ';' => Ok(Token::Semicolon),
            ':' => Ok(Token::Colon),
            ',' => Ok(Token::Comma),
            '=' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(Token::EqEq)
                },
                Some('>') => {
                    self.advance();
                    Ok(Token::Arrow)
                },
                _ => Ok(Token::Eq)
            },
            '!' => match self.peek() {
                Some('=') => {
                    self.advance();
                    Ok(Token::BangEq)
                }
                _ => Ok(Token::Bang)
            },

            '"' => self.read_str(),
            '\''=> {
                if let Some(c) = self.advance() {
                    Ok(Token::CharData(c))
                } else {
                    Err(LexError {
                        msg: format!("The parentheses \'\' are not closed: {:?}", c),
                        line: self.line,
                        col: self.col })
                }
            }

            c if c.is_ascii_digit() => self.read_number(c),
            c if c.is_alphabetic() || c == '_' => Ok(self.read_ident(c)),

            c => Err(LexError {
                msg: format!("Unexpected char: {:?}", c),
                line: self.line,
                col: self.col
            }),
        }
    }
}