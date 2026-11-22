use crate::lexer::{LexError, Lexer, Token};

// Error

#[derive(Debug)]
pub enum ParseError {
    Lex(LexError),
    Unexpected {
        expected: String,
        got: Token,
        pos: (u32, u32),
    },
    UnexpectedToken(Token, (u32, u32)),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Lex(e) => write!(f, "{e}"),
            Self::Unexpected { expected, got, pos } => {
                write!(f, "{}:{} Expected {expected}, got {got:?}", pos.0, pos.1)
            }
            Self::UnexpectedToken(t, pos) => {
                write!(f, "{}:{} Unexpected token: {t:?}", pos.0, pos.1)
            }
        }
    }
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        Self::Lex(e)
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

// AST
#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}
#[derive(Debug)]
pub enum UnaryOp {
    Not,
    Neg,
    Inc,
    Dec,
}
#[derive(Debug)]
pub enum AssignOp {
    Eq,
    AddEq,
    SubEq,
    MulEq,
    DivEq,
    Inc,
    Dec,
}

#[derive(Debug)]
pub enum Expr {
    IntNum(i64),
    FloatNum(f64),
    Bool(bool),
    Str(String),
    Var(String),
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    VarDecl {
        is_mut: bool,
        is_ref: bool,
        typ: Type,
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
        op: AssignOp,
        value: Expr,
    },
    Return(Expr),
    ReturnVoid,
    ImplicitReturn(Expr),
    If {
        cond: Expr,
        body: Vec<Stmt>,
        elif: Vec<(Expr, Vec<Stmt>)>,
        els: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    Expr(Expr),
}

#[derive(Debug)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    
    U8,
    U16,
    U32,

    F32,
    F64,
    
    Bool,
    
    Char,
    Str,
    
    Inferred, // let x = ...
    Void,     // for fns
}

#[derive(Debug)]
pub struct Func {
    pub name: String,
    pub params: Vec<(Type, String)>,
    pub ret: Type,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Func>,
}

// Parser

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> ParseResult<Self> {
        let current = lexer.next_token()?;
        Ok(Self { lexer, current })
    }

    fn advance(&mut self) -> ParseResult<Token> {
        let next = self.lexer.next_token()?;
        Ok(std::mem::replace(&mut self.current, next))
    }

    fn eat(&mut self, expected: &Token) -> ParseResult<()> {
        if &self.current != expected {
            return Err(ParseError::Unexpected {
                expected: format!("{expected:?}"),
                got: self.current.clone(),
                pos: self.pos(),
            });
        }
        self.advance()?;
        Ok(())
    }

    fn pos(&self) -> (u32, u32) {
        (self.lexer.line, self.lexer.col)
    }

    // public entry

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut functions = Vec::with_capacity(16);
        while self.current != Token::Eof {
            let function = self.parse_fn()?;
            functions.push(function);
        }

        Ok(Program { functions })
    }

    // help parsers

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match &self.current {
            Token::Let | Token::Mut | Token::Amp => self.parse_var_decl(),
            Token::I8 | Token::I16 | Token::I32 | Token::I64 => self.parse_var_decl(),
            Token::U8 | Token::U16 | Token::U32 => self.parse_var_decl(),
            Token::Bool | Token::Char | Token::Str => self.parse_var_decl(),

            Token::Return => self.parse_return(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Loop => self.parse_loop(),

            Token::Ident(..) => self.parse_assign_or_expr(),

            _ => self.parse_expr_stmt(),
        }
    }
    fn parse_expr_stmt(&mut self) -> ParseResult<Stmt> {
        let e = self.parse_expr()?;
        self.eat(&Token::Semicolon)?;
        Ok(Stmt::Expr(e))
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        match &self.advance()? {
            Token::I8 => Ok(Type::I8),
            Token::I16 => Ok(Type::I16),
            Token::I32 => Ok(Type::I32),
            Token::I64 => Ok(Type::I64),

            Token::U8 => Ok(Type::U8),
            Token::U16 => Ok(Type::U16),
            Token::U32 => Ok(Type::U32),

            Token::F32 => Ok(Type::F32),
            Token::F64 => Ok(Type::F64),

            Token::Bool => Ok(Type::Bool),

            Token::Char => Ok(Type::Char),
            Token::Str => Ok(Type::Str),

            t => Err(ParseError::Unexpected {
                expected: "type".into(),
                got: t.clone(),
                pos: self.pos(),
            }),
        }
    }

    fn parse_assign_or_expr(&mut self) -> ParseResult<Stmt> {
        let name = match self.advance()? {
            Token::Ident(s) => s,
            t => return Err(ParseError::UnexpectedToken(t, self.pos())),
        };

        match &self.current {
            Token::Eq => {
                self.advance()?;
                let value = self.parse_expr()?;
                self.eat(&Token::Semicolon)?;
                Ok(Stmt::Assign {
                    name,
                    op: AssignOp::Eq,
                    value,
                })
            }
            Token::PlusEq => {
                self.advance()?;
                let value = self.parse_expr()?;
                self.eat(&Token::Semicolon)?;
                Ok(Stmt::Assign {
                    name,
                    op: AssignOp::AddEq,
                    value,
                })
            }
            Token::MinusEq => {
                self.advance()?;
                let value = self.parse_expr()?;
                self.eat(&Token::Semicolon)?;
                Ok(Stmt::Assign {
                    name,
                    op: AssignOp::SubEq,
                    value,
                })
            }
            Token::MulEq => {
                self.advance()?;
                let value = self.parse_expr()?;
                self.eat(&Token::Semicolon)?;
                Ok(Stmt::Assign {
                    name,
                    op: AssignOp::MulEq,
                    value,
                })
            }
            Token::DivEq => {
                self.advance()?;
                let value = self.parse_expr()?;
                self.eat(&Token::Semicolon)?;
                Ok(Stmt::Assign {
                    name,
                    op: AssignOp::DivEq,
                    value,
                })
            }
            Token::PlusPlus => {
                self.advance()?;
                self.eat(&Token::Semicolon)?;
                // i++ === i += 1
                Ok(Stmt::Assign {
                    name,
                    op: AssignOp::AddEq,
                    value: Expr::IntNum(1),
                })
            }
            Token::MinusMinus => {
                self.advance()?;
                self.eat(&Token::Semicolon)?;
                Ok(Stmt::Assign {
                    name,
                    op: AssignOp::SubEq,
                    value: Expr::IntNum(1),
                })
            }

            _ => {
                let expr = self.finish_expr_from_ident(name)?;
                match &self.current {
                    Token::Semicolon => {
                        self.advance()?;
                        Ok(Stmt::Expr(expr))
                    }
                    Token::RBrace => Ok(Stmt::ImplicitReturn(expr)),

                    t => Err(ParseError::Unexpected {
                        expected: "\';\' or \'}\'".into(),
                        got: t.clone(),
                        pos: self.pos(),
                    }),
                }
            }
        }
    }

    fn finish_expr_from_ident(&mut self, name: String) -> ParseResult<Expr> {
        let atom = if self.current == Token::LParen {
            self.eat(&Token::LParen)?;
            let mut args = Vec::with_capacity(8);
            loop {
                if self.current == Token::RParen {
                    break;
                }

                args.push(self.parse_expr()?);
                if self.current == Token::Comma {
                    self.advance()?;
                } else {
                    break;
                }
            }

            self.eat(&Token::RParen)?;
            Expr::Call { name, args }
        } else {
            Expr::Var(name)
        };

        self.finish_expr_from(atom)
    }

    fn finish_expr_from(&mut self, left: Expr) -> ParseResult<Expr> {
        let mut left = left;
        loop {
            let op = match &self.current {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                Token::Mul => BinOp::Mul,
                Token::Div => BinOp::Div,
                Token::EqEq => BinOp::Eq,
                Token::BangEq => BinOp::NotEq,
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,
                Token::AmpAmp => BinOp::And,
                Token::PipePipe => BinOp::Or,
                _ => break,
            };

            self.advance()?;
            let right = self.parse_term()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_var_decl(&mut self) -> ParseResult<Stmt> {
        let is_ref = self.current == Token::Amp;
        if is_ref {
            self.advance()?;
        }

        let is_mut;
        let typ = match &self.current {
            Token::Let => {
                self.advance()?;
                is_mut = false;

                Type::Inferred
            }
            Token::Mut => {
                self.advance()?;
                is_mut = true;

                match &self.current {
                    Token::Ident(..) => Type::Inferred,
                    _ => self.parse_type()?,
                }
            }
            _ => {
                is_mut = false;
                self.parse_type()?
            }
        };

        let name = match self.advance()? {
            Token::Ident(s) => s,
            t => {
                return Err(ParseError::Unexpected {
                    expected: "variable name".into(),
                    got: t,
                    pos: self.pos(),
                });
            }
        };

        self.eat(&Token::Eq)?;
        let value = self.parse_expr()?;
        self.eat(&Token::Semicolon)?;

        Ok(Stmt::VarDecl {
            is_mut,
            is_ref,
            typ,
            name,
            value,
        })
    }

    fn parse_block(&mut self) -> ParseResult<Vec<Stmt>> {
        self.eat(&Token::LBrace)?;
        let mut stmts = Vec::with_capacity(32);

        loop {
            if matches!(self.current, Token::RBrace | Token::Eof) {
                break;
            }
            
            match &self.current {
                Token::Let | Token::Mut | Token::Amp
                | Token::I8  | Token::I16 | Token::I32 | Token::I64
                | Token::U8  | Token::U16 | Token::U32
                | Token::F32 | Token::F64
                | Token::Bool | Token::Char | Token::Str
                | Token::Return | Token::If | Token::While | Token::Loop
                | Token::Ident(_) => {
                    stmts.push(self.parse_stmt()?);
                }
                _ => {
                    let expr = self.parse_expr()?;
                    match &self.current {
                        Token::Semicolon => {
                            self.advance()?;
                            stmts.push(Stmt::Expr(expr));
                        }
                        Token::RBrace => {
                            stmts.push(Stmt::ImplicitReturn(expr));
                            break;
                        }
                        t => {
                            return Err(ParseError::Unexpected {
                                expected: "\';\' or \'}\'".into(),
                                got: t.clone(),
                                pos: self.pos(),
                            });
                        }
                    }
                }
            };
        }

        self.eat(&Token::RBrace)?;
        Ok(stmts)
    }

    fn parse_return(&mut self) -> ParseResult<Stmt> {
        self.eat(&Token::Return)?;
        if self.current == Token::Semicolon {
            self.advance()?;
            return Ok(Stmt::ReturnVoid);
        }
        let value = self.parse_expr()?;
        self.eat(&Token::Semicolon)?;

        Ok(Stmt::Return(value))
    }

    fn parse_if(&mut self) -> ParseResult<Stmt> {
        self.eat(&Token::If)?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;

        let mut elifs = Vec::new();
        let mut else_body = None;

        loop {
            match &self.current {
                Token::Elif => {
                    self.advance()?;
                    let cond = self.parse_expr()?;
                    let body = self.parse_block()?;
                    elifs.push((cond, body));
                }
                Token::Else => {
                    self.advance()?;
                    if self.current == Token::If {
                        self.advance()?;
                        let cond = self.parse_expr()?;
                        let body = self.parse_block()?;
                        elifs.push((cond, body));
                    } else {
                        else_body = Some(self.parse_block()?);
                        break;
                    }
                }
                _ => break,
            }
        }

        Ok(Stmt::If {
            cond,
            body,
            elif: elifs,
            els: else_body,
        })
    }

    fn parse_while(&mut self) -> ParseResult<Stmt> {
        self.eat(&Token::While)?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;

        Ok(Stmt::While { cond, body })
    }

    fn parse_loop(&mut self) -> ParseResult<Stmt> {
        self.eat(&Token::Loop)?;
        let body = self.parse_block()?;

        Ok(Stmt::Loop { body })
    }

    // main parsers

    fn parse_fn(&mut self) -> ParseResult<Func> {
        self.eat(&Token::Fn)?;

        // name
        let name = match self.advance()? {
            Token::Ident(s) => s,
            t => {
                return Err(ParseError::Unexpected {
                    expected: "function name".into(),
                    got: t,
                    pos: self.pos(),
                });
            }
        };

        // params
        self.eat(&Token::LParen)?;
        let mut params: Vec<(Type, String)> = Vec::with_capacity(8);
        loop {
            if self.current == Token::RParen {
                break;
            }

            let typ = self.parse_type()?;
            let name = match self.advance()? {
                Token::Ident(s) => s,
                t => {
                    return Err(ParseError::Unexpected {
                        expected: "function name".into(),
                        got: t,
                        pos: self.pos(),
                    });
                }
            };

            params.push((typ, name));

            if self.current == Token::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        self.eat(&Token::RParen)?; // closing )

        // return type
        let ret = match &self.current {
            Token::I8
            | Token::I16
            | Token::I32
            | Token::I64
            | Token::U8
            | Token::U16
            | Token::U32
            | Token::F32
            | Token::F64
            | Token::Bool
            | Token::Char
            | Token::Str => self.parse_type()?,
            _ => Type::Void,
        };

        // body
        let body = self.parse_block()?;

        Ok(Func {
            name,
            params,
            ret,
            body,
        })
    }

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_and()?;

        loop {
            let op = match &self.current {
                Token::Pipe => BinOp::Or,
                Token::PipePipe => BinOp::Or,

                _ => break,
            };
            self.advance()?;
            let right = self.parse_and()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = match &self.current {
                Token::Amp => BinOp::And,
                Token::AmpAmp => BinOp::And,

                _ => break,
            };
            self.advance()?;
            let right = self.parse_comparison()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_addition()?;

        loop {
            let op = match &self.current {
                Token::EqEq => BinOp::Eq,
                Token::BangEq => BinOp::NotEq,

                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,

                _ => break,
            };
            self.advance()?;
            let right = self.parse_addition()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_term()?;

        loop {
            let op = match &self.current {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,

                _ => break,
            };
            self.advance()?;
            let right = self.parse_term()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match &self.current {
                Token::Mul => BinOp::Mul,
                Token::Div => BinOp::Div,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_unary()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> ParseResult<Expr> {
        match &self.current {
            Token::Bang => {
                self.advance()?;
                let expr = self.parse_atom()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            Token::Minus => {
                self.advance()?;
                let expr = self.parse_atom()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }

            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> ParseResult<Expr> {
        match self.advance()? {
            Token::IntNum(v) => Ok(Expr::IntNum(v)),
            Token::FloatNum(v) => Ok(Expr::FloatNum(v)),
            Token::StrData(v) => Ok(Expr::Str(v)),
            Token::BoolData(v) => Ok(Expr::Bool(v)),
            Token::Ident(s) => {
                if self.current == Token::LParen {
                    self.eat(&Token::LParen)?;

                    let mut args: Vec<Expr> = Vec::with_capacity(8);

                    loop {
                        if self.current == Token::RParen {
                            break;
                        }

                        let arg = self.parse_expr()?;
                        args.push(arg);

                        if self.current == Token::Comma {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }

                    self.eat(&Token::RParen)?;

                    Ok(Expr::Call {
                        name: s,
                        args: args,
                    })
                } else {
                    Ok(Expr::Var(s))
                }
            }

            Token::LParen => {
                let e = self.parse_expr()?;
                self.eat(&Token::RParen)?;
                Ok(e)
            }

            t => Err(ParseError::UnexpectedToken(t, self.pos())),
        }
    }
}
