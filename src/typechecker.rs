use crate::parser::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeError {
    pub msg: String,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Type error: {}", self.msg)
    }
}

pub type TypeResult<T> = Result<T, TypeError>;

pub struct Scope {
    vars: Vec<HashMap<String, Type>>,
}
impl Scope {
    pub fn new() -> Self {
        Self {
            vars: vec![HashMap::with_capacity(8)]
        }
    }

    pub fn push(&mut self) {
        self.vars.push(HashMap::with_capacity(8));
    }

    pub fn pop(&mut self) {
        self.vars.pop();
    }

    pub fn insert(&mut self, name: String, typ: Type) {
        self.vars.last_mut()
            .unwrap()
            .insert(name, typ);
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.vars.iter().rev() {
            if let Some(typ) = scope.get(name) {
                return Some(typ);
            }
        }

        None
    }
}

pub struct TypeChecker {
    functions: HashMap<String, Type>,
    scope: Scope,
}
impl TypeChecker {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            scope: Scope::new(),
        }
    }
    
    fn infer_expr(&self, expr: &Expr) -> TypeResult<Type> {
        match expr {
            Expr::IntNum(_) => Ok(Type::I32),
            Expr::FloatNum(_) => Ok(Type::F64),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::Str(_) => Ok(Type::Str),

            Expr::Var(name) => {
                self.scope.lookup(name)
                    .cloned()
                    .ok_or_else(|| TypeError {
                        msg: format!("Undefined variable: {name}")
                    })
            }

            Expr::BinOp { left, right, .. } => {
                let l = self.infer_expr(left)?;
                let r = self.infer_expr(right)?;
                if l == r {
                    Ok(l)
                } else {
                    Err(TypeError {
                        msg: format!("Type mismatch: {l:?} vs {r:?}")
                    })
                }
            }

            Expr::UnaryOp { expr, .. } => self.infer_expr(expr),

            Expr::Call { name, .. } => {
                self.functions.get(name)
                    .cloned()
                    .ok_or_else(|| TypeError {
                        msg: format!("Undefined function: {name}"),
                    })
            }

            Expr::Match { arms, .. } => {
                match arms.first() {
                    None => Ok(Type::Void),
                    Some(arm) => match &arm.body {
                        MatchBody::Expr(e) => self.infer_expr(e),
                        MatchBody::Block(_) => Ok(Type::Void),
                    }
                }
            }
        }
    }

    pub fn check_program(&mut self, program: &mut Program) -> TypeResult<()> {
        for func in &program.functions {
            self.functions.insert(func.name.clone(), func.ret.clone());
        }

        for func in &mut program.functions {
            self.check_func(func)?;
        }

        Ok(())
    }

    fn check_func(&mut self, func: &mut Func) -> TypeResult<()> {
        self.scope.push();

        for (ty, name) in &func.params {
            self.scope.insert(name.clone(), ty.clone());
        }

        self.check_block(&mut func.body, &func.ret)?;

        self.scope.pop();
        Ok(())
    }

    fn check_block(&mut self, stmts: &mut Vec<Stmt>, ret_ty: &Type) -> TypeResult<()> {
        self.scope.push();
        for stmt in stmts {
            self.check_stmt(stmt, ret_ty)?;
        }
        self.scope.pop();
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &mut Stmt, ret_ty: &Type) -> TypeResult<()> {
        match stmt {
            Stmt::VarDecl { typ, name, value, .. } => {
                if let Some(val_expr) = value {
                    let inferred = self.infer_expr(val_expr)?;
                    if *typ == Type::Inferred {
                        *typ = inferred.clone();
                    }
                    self.scope.insert(name.clone(), inferred);
                } else {
                    if *typ == Type::Inferred {
                        return Err(TypeError {
                            msg: format!("Cannot infer type of uninitialized variable: {name}"),
                        });
                    }
                    self.scope.insert(name.clone(), typ.clone());
                }
            }

            Stmt::Return(expr) | Stmt::ImplicitReturn(expr) => {
                let ty = self.infer_expr(expr)?;
                if &ty != ret_ty {
                    return Err(TypeError {
                        msg: format!("Expected return type {ret_ty:?}, got {ty:?}"),
                    });
                }
            }

            Stmt::If { body, elif, els, .. } => {
                self.check_block(body, ret_ty)?;
                for (_, b) in elif { self.check_block(b, ret_ty)?; }
                if let Some(b) = els { self.check_block(b, ret_ty)?; }
            }

            Stmt::While { body, .. } | Stmt::Loop { body } => {
                self.check_block(body, ret_ty)?;
            }

            Stmt::Assign { value, .. } => {
                self.infer_expr(value)?;
            }

            _ => {}
        }
        Ok(())
    }
}