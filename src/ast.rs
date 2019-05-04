use crate::compiler;

// ast
#[allow(unused_variables)]
pub mod ast {
    use std::fmt;
    use std::any::Any;
    use super::compiler;

    macro_rules! as_any {
        () => (fn as_any(&self) -> &dyn Any { self });
    }

    pub trait AST : fmt::Debug {
        fn as_any(&self) -> &dyn Any;
        fn emit(&self, c : &mut compiler::Compiler);
    }

    // # values
    // ## identifier
    pub struct Identifier {
        pub val : String
    }
    impl fmt::Debug for Identifier {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"identifier\": \"{}\"}}", self.val)
        }
    }
    impl AST for Identifier {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }
    // ## strings
    pub struct StrLiteral {
        pub val : String
    }
    impl fmt::Debug for StrLiteral {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"string\": \"{}\"}}", self.val)
        }
    }
    impl AST for StrLiteral {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }
    // ## ints
    pub struct IntLiteral {
        pub val : i64
    }
    impl fmt::Debug for IntLiteral {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"integer\": {}}}", self.val)
        }
    }
    impl AST for IntLiteral {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }
    // ## floats
    pub struct FloatLiteral {
        pub val : f64
    }
    impl fmt::Debug for FloatLiteral {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"float\": {}}}", self.val)
        }
    }
    impl AST for FloatLiteral {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // # expressions
    // ## cond expr
    pub struct CondExpr {
        pub cond : std::boxed::Box<AST>,
        pub then : std::boxed::Box<AST>,
        pub alt  : std::boxed::Box<AST>,
    }
    impl fmt::Debug for CondExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"cond\": {:?}, \"then\": {:?}, \"alt\": {:?}, \"op\": \"cond\"}}",
                self.cond, self.then, self.alt)
        }
    }
    impl AST for CondExpr {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }
    // ## binexpr
    #[derive(Debug, PartialEq)]
    pub enum BinOp {
        Add, Sub, Mul, Div, Mod,
        And, Or,
        Eq, Neq, Gt, Lt, Geq, Leq,
        Assign, Adds, Subs, Muls, Divs, Mods,
    }
    pub struct BinExpr {
        pub left : std::boxed::Box<AST>,
        pub right : std::boxed::Box<AST>,
        pub op: BinOp,
    }
    impl fmt::Debug for BinExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"left\": {:?}, \"right\": {:?}, \"op\": \"{}\"}}",
                self.left, self.right, match &self.op {
                    BinOp::Add => "+",   BinOp::Sub => "-",
                    BinOp::Mul => "*",   BinOp::Div => "/", BinOp::Mod => "%",
                    BinOp::And => "and", BinOp::Or  => "or",
                    BinOp::Eq  => "==",  BinOp::Neq  => "!=",
                    BinOp::Gt  => ">",   BinOp::Geq  => ">=",
                    BinOp::Lt  => "<",   BinOp::Leq  => "<=",
                    BinOp::Assign => "=", BinOp::Adds => "+=", BinOp::Subs => "-=",
                    BinOp::Muls => "*=",  BinOp::Divs => "/=", BinOp::Mods => "%=",
                })
        }
    }
    impl AST for BinExpr {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // ## member expr
    pub struct MemExpr {
        pub left : std::boxed::Box<AST>,
        pub right : std::boxed::Box<AST>,
    }
    impl fmt::Debug for MemExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"left\": {:?}, \"right\": {:?}, \"op\": \"memexpr\"}}",
                self.left, self.right)
        }
    }
    impl AST for MemExpr {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // ## call expr
    pub struct CallExpr {
        pub callee : std::boxed::Box<AST>,
        pub args : Vec<std::boxed::Box<AST>>,
    }
    impl fmt::Debug for CallExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"callee\": {:?}, \"args\": {:?}, \"op\": \"call\"}}",
                self.callee, self.args)
        }
    }
    impl AST for CallExpr {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // #region statement
    // ## control flows
    // ### if
    pub struct IfStatement {
        pub expr : std::boxed::Box<AST>,
        pub then : std::boxed::Box<AST>,
        pub alt  : Option<std::boxed::Box<AST>>,
    }
    impl fmt::Debug for IfStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            const FMT_END : &'static str = "\"type\": \"ifstmt\"";
            match &self.alt {
                Some(alt) =>
                    write!(f, "{{\"expr\": {:?}, \"then\": {:?}, \"alt\": {:?}, {}}}",
                        self.expr, self.then, alt, FMT_END),
                None =>
                    write!(f, "{{\"expr\": {:?}, \"then\": {:?}, {}}}",
                        self.expr, self.then, FMT_END)
            }
        }
    }
    impl AST for IfStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // ### while
    pub struct WhileStatement {
        pub expr : std::boxed::Box<AST>,
        pub then : std::boxed::Box<AST>,
    }
    impl fmt::Debug for WhileStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"expr\": {:?}, \"then\": {:?}, \"type\": \"whilestmt\"}}",
                    self.expr, self.then)
        }
    }
    impl AST for WhileStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // ### for
    pub struct ForStatement {
        pub id   : std::boxed::Box<AST>,
        pub from : std::boxed::Box<AST>,
        pub to   : std::boxed::Box<AST>,
        pub step : std::boxed::Box<AST>,
        pub stmt : std::boxed::Box<AST>,
        pub is_down : bool
    }
    impl fmt::Debug for ForStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{
                \"id\": {:?},
                \"from\": {:?},
                \"to\": {:?},
                \"step\": {:?},
                \"is_down\": {},
                \"statement\": {:?},
                \"type\": \"forstmt\"}}",
                    self.id, self.from, self.to, self.step, self.is_down, self.stmt)
        }
    }
    impl AST for ForStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // ## other
    // ### fn
    pub struct FunctionStatement {
        pub id : String,
        pub args : Vec<String>,
        pub stmt : std::boxed::Box<AST>,
    }
    impl fmt::Debug for FunctionStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut args : String = "[".to_string();
            let argsv : Vec<String> = self.args.iter().map(|x| format!("\"{}\"", x)).collect();
            args += &argsv.join(",");
            args += "]";
            write!(f, "{{
                \"id\": \"{}\",
                \"args\": {},
                \"stmt\": {:?},
                \"type\": \"fnstmt\"}}",
                    self.id, args, self.stmt)
        }
    }
    impl AST for FunctionStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }
    // #### return
    pub struct ReturnStatement {
        pub expr : std::boxed::Box<AST>,
    }
    impl fmt::Debug for ReturnStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for ReturnStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // ### try
    pub struct TryStatement {
        pub stmts : Vec<std::boxed::Box<AST>>,
        pub cases : Vec<std::boxed::Box<CaseStatement>>,
    }
    impl fmt::Debug for TryStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for TryStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }
    // #### case
    pub struct CaseStatement {
        pub etype : std::boxed::Box<AST>,
        pub id    : Option<std::boxed::Box<AST>>,
        pub stmts : Vec<std::boxed::Box<AST>>,
    }
    impl fmt::Debug for CaseStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for CaseStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // ### expr
    pub struct ExprStatement {
        pub expr : std::boxed::Box<AST>,
    }
    impl fmt::Debug for ExprStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"expr\": {:?}, \"type\": \"exprstmt\"}}",
                    self.expr)
        }
    }
    impl AST for ExprStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }

    // ### block
    pub struct BlockStatement {
        pub stmts : Vec<std::boxed::Box<AST>>,
    }
    impl fmt::Debug for BlockStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"stmts\": {:?}, \"type\": \"blockstmt\"}}",
                    self.stmts)
        }
    }
    impl AST for BlockStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }
    // #endregion

}

// parser
pub mod grammar {
    include!(concat!(env!("OUT_DIR"), "/parser.rs"));
}