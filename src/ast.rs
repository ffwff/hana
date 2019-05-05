use crate::compiler;
use crate::vmbindings::vm::VmOpcode;

// ast
#[allow(unused_variables)]
pub mod ast {
    use std::fmt;
    use std::any::Any;
    use super::compiler;
    use super::VmOpcode;

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
            c.emit_get_var(self.val.clone());
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
            c.vm.code.push(VmOpcode::OP_PUSHSTR);
            c.vm.cpushs(self.val.clone());
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
            c.vm.code.push(VmOpcode::OP_PUSH64);
            unsafe {
                c.vm.cpush64(std::mem::transmute::<i64, u64>(self.val));
            }
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
            c.vm.code.push(VmOpcode::OP_PUSH64);
            c.vm.cpushf64(self.val);
        }
    }

    // # expressions
    // ## unary expr
    pub enum UnaryOp {
        Not, Neg
    }
    pub struct UnaryExpr {
        pub op : UnaryOp,
        pub val : std::boxed::Box<AST>,
    }
    impl fmt::Debug for UnaryExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for UnaryExpr {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            unimplemented!()
        }
    }
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
            macro_rules! arithop_do {
                ($x:expr) => {{
                    self.left.emit(c);
                    c.vm.code.push($x);
                }};
            }
            self.right.emit(c);
            match self.op {
                // assignment
                BinOp::Assign => {
                    let any = self.left.as_any();
                    if let Some(id) = any.downcast_ref::<Identifier>() {
                        c.emit_set_var(id.val.clone());
                    } else if let Some(memexpr) = any.downcast_ref::<MemExpr>() {
                        memexpr.left.emit(c);
                        let any = memexpr.right.as_any();
                        // optimize static member vars
                        if let Some(id) = any.downcast_ref::<Identifier>() {
                            c.vm.code.push(VmOpcode::OP_MEMBER_SET);
                            c.vm.cpushs(id.val.clone());
                        } else if let Some(str) = any.downcast_ref::<StrLiteral>() {
                            c.vm.code.push(VmOpcode::OP_MEMBER_SET);
                            c.vm.cpushs(str.val.clone());
                        } else { // otherwise, do OP_INDEX_SET as normal
                            memexpr.right.emit(c);
                            c.vm.code.push(VmOpcode::OP_INDEX_SET);
                        }
                    }
                },
                // basic manip operators
                BinOp::Add => arithop_do!(VmOpcode::OP_ADD),
                BinOp::Sub => arithop_do!(VmOpcode::OP_SUB),
                BinOp::Mul => arithop_do!(VmOpcode::OP_MUL),
                BinOp::Div => arithop_do!(VmOpcode::OP_DIV),
                BinOp::And => arithop_do!(VmOpcode::OP_AND),
                BinOp::Or  => arithop_do!(VmOpcode::OP_OR ),
                BinOp::Eq  => arithop_do!(VmOpcode::OP_EQ ),
                BinOp::Neq => arithop_do!(VmOpcode::OP_NEQ),
                BinOp::Gt  => arithop_do!(VmOpcode::OP_GT ),
                BinOp::Lt  => arithop_do!(VmOpcode::OP_LT ),
                BinOp::Geq => arithop_do!(VmOpcode::OP_GEQ),
                BinOp::Leq => arithop_do!(VmOpcode::OP_LEQ),
                _ => unimplemented!()
            }
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
            // Pseudo code of the generated bytecode
            //   [condition]
            //   jncond [else]
            //   [statement]
            //   jmp done
            //   [else]
            //   [done]
            self.expr.emit(c);
            c.vm.code.push(VmOpcode::OP_JNCOND); // TODO: maybe do peephole opt?
            let else_label = c.reserve_label();
            self.then.emit(c);
            if let Some(alt) = &self.alt {
                c.vm.code.push(VmOpcode::OP_JMP);
                let done_label = c.reserve_label();
                c.fill_label(else_label, c.vm.code.len());
                alt.emit(c);
                c.fill_label(done_label, c.vm.code.len());
            } else {
                c.fill_label(else_label, c.vm.code.len());
            }
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
            self.expr.emit(c);
            c.vm.code.push(VmOpcode::OP_POP);
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