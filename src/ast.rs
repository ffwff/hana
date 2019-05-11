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
        pub val : String,
        rawval : String
    }
    impl StrLiteral {
        pub fn new(str : &String) -> StrLiteral {
            let mut s = "".to_string();
            let mut chars = str.chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    let next = chars.next();
                    match next {
                        Some('n') => s += "\n",
                        Some('r') => s += "\r",
                        _ => panic!("expected character")
                    }
                } else {
                    s += &c.to_string();
                }
            }
            StrLiteral { val: s, rawval: str.clone() }
        }
    }
    impl fmt::Debug for StrLiteral {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"string\": \"{}\"}}", self.rawval)
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
            unsafe {
                let n = std::mem::transmute::<i64, u64>(self.val);
                match n {
                0...0xff =>  {
                        c.vm.code.push(VmOpcode::OP_PUSH8);
                        c.vm.cpush8(n as u8);
                    },
                0x100...0xffff =>  {
                        c.vm.code.push(VmOpcode::OP_PUSH16);
                        c.vm.cpush16(n as u16);
                    },
                0x10000...0xffffffff =>  {
                        c.vm.code.push(VmOpcode::OP_PUSH32);
                        c.vm.cpush32(n as u32);
                    },
                _ => {
                        c.vm.code.push(VmOpcode::OP_PUSH64);
                        c.vm.cpush64(n);
                    }
                }
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
    // ## arrays
    pub struct ArrayExpr {
        pub exprs : Vec<std::boxed::Box<AST>>
    }
    impl fmt::Debug for ArrayExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for ArrayExpr {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            for expr in &self.exprs { expr.emit(c); }
            c.vm.code.push(VmOpcode::OP_PUSH64);
            c.vm.cpush64(self.exprs.len() as u64);
            c.vm.code.push(VmOpcode::OP_ARRAY_LOAD);
        }
    }
    // ### fn def
    pub struct FunctionDefinition {
        pub id : Option<String>,
        pub args : Vec<String>,
        pub stmt : std::boxed::Box<AST>,
    }
    impl fmt::Debug for FunctionDefinition {
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
                    self.id.as_ref().map_or("".to_string(), |x| x.clone()),
                    args, self.stmt)
        }
    }
    impl AST for FunctionDefinition {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            // definition
            c.vm.code.push(VmOpcode::OP_DEF_FUNCTION_PUSH);
            c.vm.cpush16(self.args.len() as u16);
            let function_end = c.reserve_label();

            if self.id.is_some() {
                c.set_local(self.id.as_ref().unwrap().clone());
            }
            c.scope();

            // body
            c.vm.code.push(VmOpcode::OP_ENV_NEW);
            let nslot_label = c.reserve_label16();
            for arg in &self.args {
                c.set_local(arg.clone());
            }
            self.stmt.emit(c);

            // default return
            match c.vm.code.top() {
                VmOpcode::OP_RET | VmOpcode::OP_RETCALL => {},
                _ => {
                    c.vm.code.push(VmOpcode::OP_PUSH_NIL);
                    c.vm.code.push(VmOpcode::OP_RET);
                }
            };

            // end
            let nslots = c.unscope();
            c.fill_label16(nslot_label, nslots);
            c.fill_label(function_end, c.vm.code.len());
        }
    }
    // ### record def
    pub struct RecordDefinition {
        pub id : Option<String>,
        pub stmts : Vec<std::boxed::Box<AST>>,
    }
    impl fmt::Debug for RecordDefinition {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for RecordDefinition {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            c.vm.code.push(VmOpcode::OP_PUSH_NIL);
            for stmt in &self.stmts {
                let any = stmt.as_any();
                if let Some(stmt) = any.downcast_ref::<FunctionStatement>() {
                    stmt.def().emit(c);
                    c.vm.code.push(VmOpcode::OP_PUSHSTR);
                    c.vm.cpushs(stmt.def().id.as_ref().unwrap().clone());
                } else if let Some(stmt) = any.downcast_ref::<RecordStatement>() {
                    stmt.def().emit(c);
                    c.vm.code.push(VmOpcode::OP_PUSHSTR);
                    c.vm.cpushs(stmt.def().id.as_ref().unwrap().clone());
                } else if let Some(stmt) = any.downcast_ref::<ExprStatement>() {
                    let binexpr = stmt.expr.as_any().downcast_ref::<BinExpr>().unwrap();
                    let id = binexpr.left.as_any().downcast_ref::<Identifier>()
                        .unwrap_or_else(|| panic!("left hand side must be identifier"));
                    binexpr.right.emit(c);
                    c.vm.code.push(VmOpcode::OP_PUSHSTR);
                    c.vm.cpushs(id.val.clone());
                }
            }
            c.vm.code.push(VmOpcode::OP_DICT_LOAD);
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
            // Pseudo code of the generated bytecode
            //   [condition]
            //   jncond [else]
            //   [statement]
            //   jmp done
            //   [else]
            //   [done]
            self.cond.emit(c);
            c.vm.code.push(VmOpcode::OP_JNCOND); // TODO: maybe do peephole opt?
            let else_label = c.reserve_label();
            self.then.emit(c);
            c.vm.code.push(VmOpcode::OP_JMP);
            let done_label = c.reserve_label();
            c.fill_label(else_label, c.vm.code.len());
            self.alt.emit(c);
            c.fill_label(done_label, c.vm.code.len());
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
                    self.right.emit(c);
                    c.vm.code.push($x);
                }};
            }
            match self.op {
                // assignment
                BinOp::Assign => {
                    let any = self.left.as_any();
                    if let Some(id) = any.downcast_ref::<Identifier>() {
                        self.right.emit(c);
                        c.emit_set_var(id.val.clone());
                    } else if let Some(memexpr) = any.downcast_ref::<MemExpr>() {
                        self.right.emit(c);
                        memexpr.left.emit(c);
                        let any = memexpr.right.as_any();
                        // optimize static member vars
                        let val = {
                            if let Some(id) = any.downcast_ref::<Identifier>()
                            { Some(&id.val) }
                            else if let Some(str) = any.downcast_ref::<StrLiteral>()
                            { Some(&str.val) }
                            else { None }
                        };
                        if val.is_some() && !memexpr.is_expr {
                            let val = val.unwrap();
                            c.vm.code.push(VmOpcode::OP_MEMBER_SET);
                            c.vm.cpushs(val.clone());
                        } else { // otherwise, do OP_INDEX_SET as normal
                            memexpr.right.emit(c);
                            c.vm.code.push(VmOpcode::OP_INDEX_SET);
                        }
                    } else if let Some(callexpr) = any.downcast_ref::<CallExpr>() {
                        // definition
                        c.vm.code.push(VmOpcode::OP_DEF_FUNCTION_PUSH);
                        c.vm.cpush16(callexpr.args.len() as u16);
                        let function_end = c.reserve_label();

                        c.set_local(callexpr.callee.as_any().downcast_ref::<Identifier>().unwrap().val.clone());
                        c.scope();

                        // body
                        c.vm.code.push(VmOpcode::OP_ENV_NEW);
                        let nslot_label = c.reserve_label16();
                        for arg in &callexpr.args {
                            c.set_local(arg.as_any().downcast_ref::<Identifier>().unwrap().val.clone());
                        }
                        self.right.emit(c);
                        c.vm.code.push(VmOpcode::OP_RET);

                        // end
                        let nslots = c.unscope();
                        c.fill_label16(nslot_label, nslots);
                        c.fill_label(function_end, c.vm.code.len());

                        c.emit_set_var_fn(callexpr.callee.as_any().downcast_ref::<Identifier>().unwrap().val.clone());
                    } else {
                        panic!("Invalid left hand side expression!");
                    }
                },
                BinOp::Adds | BinOp::Subs | BinOp::Muls | BinOp::Divs | BinOp::Mods => {
                    let opcode = match self.op {
                        BinOp::Adds => VmOpcode::OP_ADD,
                        BinOp::Subs => VmOpcode::OP_SUB,
                        BinOp::Muls => VmOpcode::OP_MUL,
                        BinOp::Divs => VmOpcode::OP_DIV,
                        BinOp::Mods => VmOpcode::OP_MOD,
                        _ => unreachable!()
                    };
                    let any = self.left.as_any();
                    if let Some(id) = any.downcast_ref::<Identifier>() {
                        self.right.emit(c);
                        c.emit_get_var(id.val.clone());
                        c.vm.code.push(opcode);
                        c.emit_set_var(id.val.clone());
                    } else if let Some(memexpr) = any.downcast_ref::<MemExpr>() {
                        self.right.emit(c);
                        memexpr.left.emit(c);
                        // optimize static member vars
                        let val = {
                            if let Some(id) = any.downcast_ref::<Identifier>()
                            { Some(&id.val) }
                            else if let Some(str) = any.downcast_ref::<StrLiteral>()
                            { Some(&str.val) }
                            else { None }
                        };
                        if val.is_some() && !memexpr.is_expr {
                            let val = val.unwrap();
                            c.vm.code.push(VmOpcode::OP_MEMBER_GET_NO_POP);
                            c.vm.cpushs(val.clone());
                            c.vm.code.push(opcode);
                            c.vm.code.push(VmOpcode::OP_MEMBER_SET);
                            c.vm.cpushs(val.clone());
                        } else { // otherwise, do OP_INDEX_SET as normal
                            memexpr.right.emit(c);
                            c.vm.code.push(VmOpcode::OP_INDEX_GET);
                            memexpr.left.emit(c);
                            memexpr.right.emit(c);
                            c.vm.code.push(VmOpcode::OP_INDEX_SET);
                        }
                    } else {
                        panic!("Invalid left hand side expression!");
                    }
                },
                // basic manip operators
                BinOp::Add => arithop_do!(VmOpcode::OP_ADD),
                BinOp::Sub => arithop_do!(VmOpcode::OP_SUB),
                BinOp::Mul => arithop_do!(VmOpcode::OP_MUL),
                BinOp::Div => arithop_do!(VmOpcode::OP_DIV),
                BinOp::And => arithop_do!(VmOpcode::OP_AND),
                BinOp::Mod => arithop_do!(VmOpcode::OP_MOD),
                BinOp::Or  => arithop_do!(VmOpcode::OP_OR ),
                BinOp::Eq  => arithop_do!(VmOpcode::OP_EQ ),
                BinOp::Neq => arithop_do!(VmOpcode::OP_NEQ),
                BinOp::Gt  => arithop_do!(VmOpcode::OP_GT ),
                BinOp::Lt  => arithop_do!(VmOpcode::OP_LT ),
                BinOp::Geq => arithop_do!(VmOpcode::OP_GEQ),
                BinOp::Leq => arithop_do!(VmOpcode::OP_LEQ),
                //_ => panic!("not implemented: {:?}", self.op)
            }
        }
    }

    // ## member expr
    pub struct MemExpr {
        pub left : std::boxed::Box<AST>,
        pub right : std::boxed::Box<AST>,
        pub is_expr: bool,
    }
    impl MemExpr {
        fn _emit(&self, c : &mut compiler::Compiler, is_method_call : bool) {
            self.left.emit(c);
            let get_op = if !is_method_call { VmOpcode::OP_MEMBER_GET }
                         else { VmOpcode::OP_MEMBER_GET_NO_POP };
            let any = self.right.as_any();
            // optimize static keys
            let val = {
                if let Some(id) = any.downcast_ref::<Identifier>()
                { Some(&id.val) }
                else if let Some(str) = any.downcast_ref::<StrLiteral>()
                { Some(&str.val) }
                else { None }
            };
            if val.is_some() && !self.is_expr {
                c.vm.code.push(get_op);
                c.vm.cpushs(val.unwrap().clone());
            } else {
                self.right.emit(c);
                c.vm.code.push(VmOpcode::OP_INDEX_GET);
            }
        }
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
            self._emit(c, false)
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
            for arg in self.args.iter().rev() { arg.emit(c); }
            if let Some(memexpr) = self.callee.as_any().downcast_ref::<MemExpr>() {
                memexpr._emit(c, true);
                c.vm.code.push(VmOpcode::OP_CALL);
                c.vm.cpush16((self.args.len() as u16) + 1);
            } else {
                self.callee.emit(c);
                c.vm.code.push(VmOpcode::OP_CALL);
                c.vm.cpush16(self.args.len() as u16);
            }
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
            // pseudocode of generated bytecode:
            //   begin: jmp condition
            //   [statement]
            //   [condition]
            //   jcond [begin]
            c.vm.code.push(VmOpcode::OP_JMP);
            let begin_label = c.reserve_label();

            let then_label = c.vm.code.len() as u32;
            c.loop_start();
            self.then.emit(c);

            c.fill_label(begin_label, c.vm.code.len());

            let next_it_pos = c.vm.code.len();
            self.expr.emit(c);
            c.vm.code.push(VmOpcode::OP_JCOND);
            c.vm.cpush32(then_label);

            c.loop_end(next_it_pos, c.vm.code.len());
        }
    }

    // ### for
    pub struct ForStatement {
        pub id   : String,
        pub from : std::boxed::Box<AST>,
        pub to   : std::boxed::Box<AST>,
        pub step : std::boxed::Box<AST>,
        pub stmt : std::boxed::Box<AST>,
        pub is_up : bool
    }
    impl fmt::Debug for ForStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{
                \"id\": {:?},
                \"from\": {:?},
                \"to\": {:?},
                \"step\": {:?},
                \"is_up\": {},
                \"statement\": {:?},
                \"type\": \"forstmt\"}}",
                    self.id, self.from, self.to, self.step, self.is_up, self.stmt)
        }
    }
    impl AST for ForStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            // pseudocode of generated bytecode:
            //   [start]
            //   [body]
            //   get [id]
            //   [to]
            //   neq
            //   jcond [done]
            //   [step]
            //   jmp [body]
            //   [done]

            // start
            self.from.emit(c);
            c.emit_set_var(self.id.clone());
            c.vm.code.push(VmOpcode::OP_POP);

            c.vm.code.push(VmOpcode::OP_JMP);
            let begin_label = c.reserve_label();

            let then_label = c.vm.code.len() as u32;
            c.loop_start();
            self.stmt.emit(c);

            // step
            c.emit_get_var(self.id.clone());
            self.step.emit(c);
            c.vm.code.push(if self.is_up { VmOpcode::OP_ADD }
                           else { VmOpcode::OP_SUB });
            c.emit_set_var(self.id.clone());
            c.vm.code.push(VmOpcode::OP_POP);

            c.fill_label(begin_label, c.vm.code.len());

            // condition
            let next_it_pos = c.vm.code.len();
            c.emit_get_var(self.id.clone());
            self.to.emit(c);
            c.vm.code.push(if self.is_up { VmOpcode::OP_LT }
                           else { VmOpcode::OP_GT });
            c.vm.code.push(VmOpcode::OP_JCOND);
            c.vm.cpush32(then_label);

            c.loop_end(next_it_pos, c.vm.code.len());
        }
    }
    // ### continue statement
    pub struct ContinueStatement {
    }
    impl fmt::Debug for ContinueStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for ContinueStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            c.vm.code.push(VmOpcode::OP_JMP);
            c.loop_continue();
        }
    }
    // ### break
    pub struct BreakStatement {
    }
    impl fmt::Debug for BreakStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for BreakStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            c.vm.code.push(VmOpcode::OP_JMP);
            c.loop_break();
        }
    }

    // ## other
    // #### fn
    pub struct FunctionStatement(FunctionDefinition);
    impl FunctionStatement {
        pub fn new(def : FunctionDefinition) -> FunctionStatement {
            FunctionStatement(def)
        }

        pub fn def(&self) -> &FunctionDefinition {
            &self.0
        }
    }
    impl fmt::Debug for FunctionStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for FunctionStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            self.0.emit(c);

            // set var
            c.emit_set_var_fn(self.0.id.as_ref().unwrap().clone());
            c.vm.code.push(VmOpcode::OP_POP);
        }
    }
    // #### return
    pub struct ReturnStatement {
        pub expr : Option<std::boxed::Box<AST>>,
    }
    impl fmt::Debug for ReturnStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for ReturnStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            match &self.expr {
                Some(expr) => expr.emit(c),
                None => c.vm.code.push(VmOpcode::OP_PUSH_NIL)
            }
            c.vm.code.push(VmOpcode::OP_RET);
        }
    }

    // ### record statement
    pub struct RecordStatement(RecordDefinition);
    impl RecordStatement {
        pub fn new(def : RecordDefinition) -> RecordStatement {
            RecordStatement(def)
        }

        pub fn def(&self) -> &RecordDefinition {
            &self.0
        }
    }
    impl fmt::Debug for RecordStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for RecordStatement {
        as_any!();
        fn emit(&self, c : &mut compiler::Compiler) {
            self.0.emit(c);

            // set var
            c.emit_set_var(self.0.id.as_ref().unwrap().clone());
            c.vm.code.push(VmOpcode::OP_POP);
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
            for stmt in &self.stmts {
                stmt.emit(c);
            }
        }
    }
    // #endregion

}

// parser
pub mod grammar {
    include!(concat!(env!("OUT_DIR"), "/parser.rs"));
}