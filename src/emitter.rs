use crate::parser::*;
use std::fmt::Write;

pub struct Emitter {
    output: String,
    indent: usize,
    tmp_counter: usize,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            tmp_counter: 0,
        }
    }

    pub fn finish(self) -> String {
        self.output
    }

    fn line(&mut self, s: &str) {
        let indent = "    ".repeat(self.indent);
        writeln!(self.output, "{indent}{s}").unwrap();
    }

    // Type

    fn emit_type(typ: &Type) -> &'static str {
        match typ {
            Type::Usize => "size_t",
            Type::Isize => "ssize_t",

            Type::I8 => "int8_t",
            Type::I16 => "int16_t",
            Type::I32 => "int32_t",
            Type::I64 => "int64_t",

            Type::U8 => "uint8_t",
            Type::U16 => "uint16_t",
            Type::U32 => "uint32_t",
            Type::U64 => "uint64_t",

            Type::F32 => "float",
            Type::F64 => "double",
            Type::F128 => "long double",

            Type::Bool => "char", // plug
            Type::Byte => "char", // plug
            Type::Bits => "char", // plug

            Type::Char => "char",
            Type::Str => "const char*",

            Type::Inferred => "void",
            Type::Void => "void",
        }
    }

    // Expressions

    fn emit_match_expr(&mut self, value: &Expr, arms: &[MatchArm]) -> String {
        let tmp = format!("_m{}", self.tmp_counter);
        self.tmp_counter += 1;

        let v = Self::emit_expr(value);

        self.line(&format!("size_t {tmp};"));
        self.line(&format!("switch({v}) {{"));
        self.indent += 1;

        for arm in arms {
            match &arm.pattern {
                MatchPattern::Literal(expr) => {
                    let p = Self::emit_expr(expr);
                    self.line(&format!("case {p}:"));
                }
                MatchPattern::Default => {
                    self.line("default:")
                }
                MatchPattern::Binding(name) => {
                    self.line("default:");
                    self.indent += 1;
                    self.line(&format!("size_t {name}={tmp};"));
                    self.indent -= 1;
                }
            }

            self.indent += 1;
            match &arm.body {
                MatchBody::Expr(expr) => {
                    let e = Self::emit_expr(expr);
                    self.line(&format!("{tmp}={e};"));
                    self.line("break;");
                }
                MatchBody::Block(stmts) => {
                    for s in stmts { self.emit_stmt(s); }
                    self.line(&format!("{tmp}=0;"));
                    self.line("break;");
                }
            }
            self.indent -= 1;
        }

        self.indent -= 1;
        self.line("}");
        
        tmp
    }
    
    fn emit_expr(expr: &Expr) -> String {
        match expr {
            Expr::Match { .. } => "__match__".into(),
            Expr::IntNum(v) => v.to_string(),
            Expr::FloatNum(v) => format!("{v}"),
            Expr::Bool(b) => {
                if *b {
                    "1".into()
                } else {
                    "0".into()
                }
            }
            Expr::Str(s) => format!("\"{s}\""),
            Expr::Var(name) => name.clone(),
            
            Expr::UnaryOp { op, expr } => {
                let e = Self::emit_expr(expr);
                match op {
                    UnaryOp::Neg => format!("(-{e})"),
                    UnaryOp::Not => format!("(!{e})"),
                    UnaryOp::Inc => format!("(++{e})"),
                    UnaryOp::Dec => format!("(--{e})"),
                }
            }

            Expr::BinOp { op, left, right } => {
                let l = Self::emit_expr(left);
                let r = Self::emit_expr(right);
                let op = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Eq => "==",
                    BinOp::NotEq => "!=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::LtEq => "<=",
                    BinOp::GtEq => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                format!("({l} {op} {r})")
            }

            Expr::Call { name, args } => {
                let args_str = args
                    .iter()
                    .map(Self::emit_expr)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{name}({args_str})")
            }
        }
    }

    // Println
    fn emit_println(args: &[Expr]) -> String {
        match args {
            [] => "printf(\"\\n\")".into(),

            [Expr::Str(s)] => format!("printf(\"{s}\\n\")"),

            [Expr::Str(fmt), rest @ ..] if fmt.contains("{}") => {
                let fmt_c = fmt.replace("{}", "%d");
                let rest_str = rest
                    .iter()
                    .map(Self::emit_expr)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("printf(\"{fmt_c}\\n\", {rest_str})")
            }

            [single] => {
                let e = Self::emit_expr(single);
                format!("printf(\"%d\\n\", {e})")
            }

            _ => {
                let args_str = args
                    .iter()
                    .map(Self::emit_expr)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("printf({args_str})")
            }
        }
    }

    // Statements
    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl {
                is_mut,
                is_ref,
                typ,
                name,
                value,
            } => {
                if let Some(Expr::Match { value: mv, arms }) = value.as_ref() {
                    let tmp = self.emit_match_expr(mv, arms);
                    let t = Self::emit_type(typ);
                    let cnst = if *is_mut { "" } else { "const " };
                    self.line(&format!("{cnst}{t} {name}={tmp};"));
                    return;
                }
            
                let t = Self::emit_type(typ);
                let val = match value {
                    Some(v) => format!("={}", Self::emit_expr(v)),
                    None => "".to_string(),
                };
                let cnst = if *is_mut { "" } else { "const " };
                let rf = if *is_ref { "*" } else { "" };
                self.line(&format!("{cnst}{t}{rf} {name}{val};"));
            }

            Stmt::Assign { name, op, value } => {
                if let Expr::Match { value: mv, arms } = value {
                    let tmp = self.emit_match_expr(mv, arms);
                    let op_str = match op {
                        AssignOp::Eq => "=",
                        AssignOp::AddEq => "+=",
                        AssignOp::SubEq => "-=",
                        AssignOp::MulEq => "*=",
                        AssignOp::DivEq => "/=",
                        AssignOp::Inc => "+=",
                        AssignOp::Dec => "-=",
                    };
                    self.line(&format!("{name} {op_str} {tmp};"));
                    return;
                }
            
                let val = Self::emit_expr(value);
                let op  = match op {
                    AssignOp::Eq => "=",
                    AssignOp::AddEq => "+=",
                    AssignOp::SubEq => "-=",
                    AssignOp::MulEq => "*=",
                    AssignOp::DivEq => "/=",
                    AssignOp::Inc => "+=",
                    AssignOp::Dec => "-=",
                };
                self.line(&format!("{name} {op} {val};"));
            }

            Stmt::Break => self.line("break;"),
            Stmt::Continue => self.line("continue;"),

            Stmt::Return(expr) | Stmt::ImplicitReturn(expr) => {
                let e = Self::emit_expr(expr);
                self.line(&format!("return {e};"));
            }
            Stmt::ReturnVoid => self.line("return;"),

            Stmt::Expr(expr) => {
                if let Expr::Call { name, args } = expr {
                    if name == "println" {
                        let s = Self::emit_println(args);
                        self.line(&format!("{s};"));
                        return;
                    }
                }
                let e = Self::emit_expr(expr);
                self.line(&format!("{e};"));
            }

            Stmt::If {
                cond,
                body,
                elif,
                els,
            } => {
                let c = Self::emit_expr(cond);
                self.line(&format!("if({c}) {{"));
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;

                for (elif_cond, elif_body) in elif {
                    let ec = Self::emit_expr(elif_cond);
                    self.line(&format!("}} else if({ec}) {{"));
                    self.indent += 1;
                    for s in elif_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                }

                if let Some(else_body) = els {
                    self.line("} else {");
                    self.indent += 1;
                    for s in else_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                }

                self.line("}");
            }

            Stmt::While { cond, body } => {
                let c = Self::emit_expr(cond);
                self.line(&format!("while({c}) {{"));
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.line("}");
            }

            Stmt::Loop { body } => {
                self.line("while(1) {");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.line("}");
            }
        }
    }

    // Functions
    fn emit_func(&mut self, func: &Func) {
        let ret = Self::emit_type(&func.ret);
        let params = func
            .params
            .iter()
            .map(|(t, n)| format!("{} {n}", Self::emit_type(t)))
            .collect::<Vec<_>>()
            .join(", ");

        let inline = if func.is_inline { "inline" } else { "" };

        self.line(&format!(
            "{inline} {ret} {name}({params}) {{",
            name = func.name
        ));
        self.indent += 1;
        for stmt in &func.body {
            self.emit_stmt(stmt);
        }
        self.indent -= 1;
        self.line("}");
        self.line(""); // blank line between functions
    }

    // Program
    pub fn emit_program(&mut self, program: &Program) {
        self.line("#include <stdio.h>");
        self.line("#include <stdint.h>");
        self.line("");

        for func in &program.functions {
            self.emit_func(func);
        }
    }
}
