use crate::compiler;
use crate::vmbindings::vm::VmOpcode;

// ast
#[allow(unused_variables)]
pub mod ast {
    use std::fmt;
    use std::any::Any;
    use super::compiler;
    use super::VmOpcode;

    // #region macros
    macro_rules! ast_impl {
        () => (
            fn as_any(&self) -> &dyn Any { self }
            fn span(&self) -> &Span { &self._span }
        );
    }

    macro_rules! emit_begin {
        ($self:ident, $c:ident) => (
            $c.smap.push(compiler::SourceMap {
                file: $self.span().clone(),
                fileno: if $c.files.len() == 0 { 0 }
                        else { $c.files.len() - 1 },
                bytecode: ($c.vm.code.len(), 0)
            });
        );
    }

    macro_rules! emit_end {
        ($c:ident, $smap:expr) => (
            $c.smap[$smap].bytecode.1 = $c.vm.code.len();
        );
    }

    // #endregion

    pub type Span = (usize, usize);
    pub trait AST : fmt::Debug {
        fn as_any(&self) -> &dyn Any;
        fn span(&self) -> &Span;
        fn emit(&self, c : &mut compiler::Compiler);
    }

    // # values
    // ## identifier
    pub struct Identifier {
        pub _span : Span,
        pub val : String
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for Identifier {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"identifier\": \"{}\"}}", self.val)
        }
    }
    impl AST for Identifier {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            c.emit_get_var(self.val.clone());
            emit_end!(c, _smap_begin);
        }
    }
    // ## strings
    pub struct StrLiteral {
        pub _span : Span,
        pub val : String,
        rawval : String
    }
    impl StrLiteral {
        #[cfg_attr(tarpaulin, skip)]
        pub fn new(str : &String, span: Span) -> StrLiteral {
            let mut s = "".to_string();
            let mut chars = str.chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    let next = chars.next();
                    match next {
                        Some('n') => s += "\n",
                        Some('r') => s += "\r",
                        Some('t') => s += "\t",
                        Some(x) => s += &x.to_string(),
                        _ => panic!("expected character, got eof")
                    }
                } else {
                    s += &c.to_string();
                }
            }
            StrLiteral { _span: span, val: s, rawval: str.clone() }
        }
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for StrLiteral {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"string\": \"{}\"}}", self.rawval)
        }
    }
    impl AST for StrLiteral {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            c.vm.code.push(VmOpcode::OP_PUSHSTR);
            c.vm.cpushs(self.val.clone());
            emit_end!(c, _smap_begin);
        }
    }
    // ## ints
    pub struct IntLiteral {
        pub _span : Span,
        pub val : i64
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for IntLiteral {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"integer\": {}}}", self.val)
        }
    }
    impl AST for IntLiteral {
        ast_impl!();
        #[cfg_attr(tarpaulin, skip)]
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            let n = self.val as u64;
            match n {
            0...0xff => {
                    c.vm.code.push(VmOpcode::OP_PUSH8);
                    c.vm.cpush8(n as u8);
                },
            0x100...0xffff => {
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
            emit_end!(c, _smap_begin);
        }
    }
    // ## floats
    pub struct FloatLiteral {
        pub _span : Span,
        pub val : f64
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for FloatLiteral {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"float\": {}}}", self.val)
        }
    }
    impl AST for FloatLiteral {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            c.vm.code.push(VmOpcode::OP_PUSHF64);
            c.vm.cpushf64(self.val);
            emit_end!(c, _smap_begin);
        }
    }
    // ## arrays
    pub struct ArrayExpr {
        pub _span : Span,
        pub exprs : Vec<std::boxed::Box<AST>>
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for ArrayExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"array\": {:?}}}", self.exprs)
        }
    }
    impl AST for ArrayExpr {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            for expr in &self.exprs { expr.emit(c); }
            c.vm.code.push(VmOpcode::OP_PUSH64);
            c.vm.cpush64(self.exprs.len() as u64);
            c.vm.code.push(VmOpcode::OP_ARRAY_LOAD);
            emit_end!(c, _smap_begin);
        }
    }
    // ### fn def
    pub struct FunctionDefinition {
        pub _span : Span,
        pub id : Option<String>,
        pub args : Vec<String>,
        pub stmt : std::boxed::Box<AST>,
    }
    #[cfg_attr(tarpaulin, skip)]
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
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            // definition
            c.vm.code.push(VmOpcode::OP_DEF_FUNCTION_PUSH);
            c.vm.cpush16(self.args.len() as u16);
            let function_end = c.reserve_label16();

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
            if self.id.is_some() {
                c.symbol.insert(c.vm.code.len() - 1, self.id.as_ref().unwrap().clone());
            }

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
            c.fill_label16(function_end, (c.vm.code.len() - function_end) as u16);
            emit_end!(c, _smap_begin);
        }
    }
    // ### record def
    pub struct RecordDefinition {
        pub _span : Span,
        pub id : Option<String>,
        pub stmts : Vec<std::boxed::Box<AST>>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for RecordDefinition {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for RecordDefinition {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
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
            emit_end!(c, _smap_begin);
        }
    }

    // # expressions
    // ## unary expr
    pub enum UnaryOp {
        Not, Neg
    }
    pub struct UnaryExpr {
        pub _span : Span,
        pub op : UnaryOp,
        pub val : std::boxed::Box<AST>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for UnaryExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for UnaryExpr {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            self.val.emit(c);
            match self.op {
                UnaryOp::Not => {
                    c.vm.code.push(VmOpcode::OP_NOT);
                },
                UnaryOp::Neg => {
                    c.vm.code.push(VmOpcode::OP_NEGATE);
                }
            }
            emit_end!(c, _smap_begin);
        }
    }
    // ## cond expr
    pub struct CondExpr {
        pub _span : Span,
        pub cond : std::boxed::Box<AST>,
        pub then : std::boxed::Box<AST>,
        pub alt  : std::boxed::Box<AST>,
    }
    impl CondExpr {
        fn _emit(&self, c: &mut compiler::Compiler, is_tail: bool) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            // Pseudo code of the generated bytecode
            //   [condition]
            //   jncond [else]
            //   [statement]
            //   jmp done
            //   [else]
            //   [done]
            self.cond.emit(c);
            c.vm.code.push(VmOpcode::OP_JNCOND); // TODO: maybe do peephole opt?
            let else_label = c.reserve_label16();

            if is_tail {
                if let Some(expr) = self.then.as_any().downcast_ref::<CallExpr>() {
                    expr._emit(c, true);
                } else if let Some(expr) = self.then.as_any().downcast_ref::<CondExpr>() {
                    expr._emit(c, true);
                } else { self.then.emit(c); }
            } else {
                self.then.emit(c);
            }

            c.vm.code.push(VmOpcode::OP_JMP);
            let done_label = c.reserve_label16();
            c.fill_label16(else_label, (c.vm.code.len() - else_label) as u16);

            if is_tail {
                if let Some(expr) = self.alt.as_any().downcast_ref::<CallExpr>() {
                    expr._emit(c, true);
                } else if let Some(expr) = self.then.as_any().downcast_ref::<CondExpr>() {
                    expr._emit(c, true);
                } else { self.alt.emit(c); }
            } else {
                self.alt.emit(c);
            }

            c.fill_label16(done_label, (c.vm.code.len() - done_label) as u16);
            emit_end!(c, _smap_begin);
        }
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for CondExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"cond\": {:?}, \"then\": {:?}, \"alt\": {:?}, \"op\": \"cond\"}}",
                self.cond, self.then, self.alt)
        }
    }
    impl AST for CondExpr {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            self._emit(c, false)
        }
    }
    // ## binexpr
    #[derive(Debug, PartialEq)]
    pub enum BinOp {
        Add, Sub, Mul, Div, Mod,
        And, Or,
        Eq, Neq, Gt, Lt, Geq, Leq,
        Assign, Adds, Subs, Muls, Divs, Mods,
        Of,
    }
    pub struct BinExpr {
        pub _span : Span,
        pub left : std::boxed::Box<AST>,
        pub right : std::boxed::Box<AST>,
        pub op: BinOp,
    }
    #[cfg_attr(tarpaulin, skip)]
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
                    BinOp::Of => "of"
                })
        }
    }
    impl AST for BinExpr {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
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
                        let function_end = c.reserve_label16();

                        c.set_local(callexpr.callee.as_any().downcast_ref::<Identifier>().unwrap().val.clone());
                        c.scope();

                        // body
                        c.vm.code.push(VmOpcode::OP_ENV_NEW);
                        let nslot_label = c.reserve_label16();
                        for arg in &callexpr.args {
                            c.set_local(arg.as_any().downcast_ref::<Identifier>().unwrap().val.clone());
                        }

                        if let Some(expr) =
                            self.right.as_any().downcast_ref::<CallExpr>() {
                            expr._emit(c, true);
                        } else if let Some(expr) =
                            self.right.as_any().downcast_ref::<CondExpr>() {
                            expr._emit(c, true);
                            c.vm.code.push(VmOpcode::OP_RET);
                        } else {
                            self.right.emit(c);
                            c.vm.code.push(VmOpcode::OP_RET);
                        }

                        // end
                        let nslots = c.unscope();
                        c.fill_label16(nslot_label, nslots);
                        c.fill_label16(function_end, (c.vm.code.len() - function_end) as u16);

                        let id = &callexpr.callee.as_any().downcast_ref::<Identifier>().unwrap().val;
                        if id != "_" {
                            // _ for id is considered a anonymous function decl
                            c.emit_set_var_fn(id.clone());
                        }
                    } else {
                        panic!("Invalid left hand side expression!");
                    }
                },
                BinOp::Adds | BinOp::Subs | BinOp::Muls | BinOp::Divs | BinOp::Mods => {
                    let opcode = match self.op {
                        BinOp::Adds => VmOpcode::OP_IADD,
                        BinOp::Subs => VmOpcode::OP_SUB,
                        BinOp::Muls => VmOpcode::OP_IMUL,
                        BinOp::Divs => VmOpcode::OP_DIV,
                        BinOp::Mods => VmOpcode::OP_MOD,
                        _ => unreachable!()
                    };
                    let any = self.left.as_any();
                    let mut in_place_addr = std::usize::MAX;
                    if let Some(id) = any.downcast_ref::<Identifier>() {
                        c.emit_get_var(id.val.clone());
                        self.right.emit(c);
                        c.vm.code.push(opcode.clone());
                        match opcode {
                            VmOpcode::OP_IADD | VmOpcode::OP_IMUL
                                => {
                                    in_place_addr = c.vm.code.len();
                                    c.vm.cpush8(0);
                                },
                            _ => {}
                        };
                        c.emit_set_var(id.val.clone());
                    } else if let Some(memexpr) = any.downcast_ref::<MemExpr>() {
                        memexpr.left.emit(c);
                        // optimize static member vars
                        let val = {
                            let any = memexpr.right.as_any();
                            if let Some(id) = any.downcast_ref::<Identifier>()
                            { Some(&id.val) }
                            else if let Some(str) = any.downcast_ref::<StrLiteral>()
                            { Some(&str.val) }
                            else { None }
                        };
                        // prologue
                        if val.is_some() && !memexpr.is_expr {
                            c.vm.code.push(VmOpcode::OP_MEMBER_GET_NO_POP);
                            c.vm.cpushs(val.unwrap().clone());
                        } else {
                            memexpr.right.emit(c);
                            c.vm.code.push(VmOpcode::OP_INDEX_GET_NO_POP);
                        }
                        // body
                        self.right.emit(c);
                        c.vm.code.push(opcode.clone());
                        match opcode {
                            VmOpcode::OP_IADD | VmOpcode::OP_IMUL
                                => {
                                    in_place_addr = c.vm.code.len();
                                    c.vm.cpush8(0);
                                },
                            _ => {}
                        };
                        // epilogue
                        if in_place_addr != std::usize::MAX {
                            // jmp here if we can do it in place
                            c.vm.code.as_mut_bytes()[in_place_addr]
                                = (c.vm.code.len() - in_place_addr) as u8;
                        }
                        if val.is_some() && !memexpr.is_expr {
                            c.vm.code.push(VmOpcode::OP_SWAP);
                            c.vm.code.push(VmOpcode::OP_MEMBER_SET);
                            c.vm.cpushs(val.unwrap().clone());
                        } else { // otherwise, do OP_INDEX_SET as normal
                            c.vm.code.push(VmOpcode::OP_SWAP);
                            memexpr.right.emit(c);
                            c.vm.code.push(VmOpcode::OP_INDEX_SET);
                        }
                        emit_end!(c, _smap_begin);
                        return;
                    } else {
                        panic!("Invalid left hand side expression!");
                    }
                    if in_place_addr != std::usize::MAX {
                        // jmp here if we can do it in place
                        c.vm.code.as_mut_bytes()[in_place_addr]
                            = (c.vm.code.len() - in_place_addr) as u8;
                    }
                },
                // basic manip operators
                BinOp::Add => arithop_do!(VmOpcode::OP_ADD),
                BinOp::Sub => arithop_do!(VmOpcode::OP_SUB),
                BinOp::Mul => arithop_do!(VmOpcode::OP_MUL),
                BinOp::Div => arithop_do!(VmOpcode::OP_DIV),
                BinOp::Mod => arithop_do!(VmOpcode::OP_MOD),
                BinOp::Eq  => arithop_do!(VmOpcode::OP_EQ ),
                BinOp::Neq => arithop_do!(VmOpcode::OP_NEQ),
                BinOp::Gt  => arithop_do!(VmOpcode::OP_GT ),
                BinOp::Lt  => arithop_do!(VmOpcode::OP_LT ),
                BinOp::Geq => arithop_do!(VmOpcode::OP_GEQ),
                BinOp::Leq => arithop_do!(VmOpcode::OP_LEQ),
                BinOp::Of  => arithop_do!(VmOpcode::OP_OF),
                // boolean operators
                BinOp::And => {
                    self.left.emit(c);
                    c.vm.code.push(VmOpcode::OP_JNCOND_NO_POP);
                    let label = c.reserve_label16();
                    c.vm.code.push(VmOpcode::OP_POP);
                    self.right.emit(c);
                    c.fill_label16(label, (c.vm.code.len() - label) as u16);
                },
                BinOp::Or  => {
                    self.left.emit(c);
                    c.vm.code.push(VmOpcode::OP_JCOND_NO_POP);
                    let label = c.reserve_label16();
                    self.right.emit(c);
                    c.fill_label16(label, (c.vm.code.len() - label) as u16);
                },
                //_ => panic!("not implemented: {:?}", self.op)
            }
            emit_end!(c, _smap_begin);
        }
    }

    // ## member expr
    pub struct MemExpr {
        pub _span : Span,
        pub left : std::boxed::Box<AST>,
        pub right : std::boxed::Box<AST>,
        pub is_expr: bool,
        pub is_namespace: bool,
    }
    impl MemExpr {
        fn _emit(&self, c : &mut compiler::Compiler, is_method_call : bool) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
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
            emit_end!(c, _smap_begin);
        }
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for MemExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"left\": {:?}, \"right\": {:?}, \"op\": \"memexpr\"}}",
                self.left, self.right)
        }
    }
    impl AST for MemExpr {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            self._emit(c, false)
        }
    }

    // ## call expr
    pub struct CallExpr {
        pub _span : Span,
        pub callee : std::boxed::Box<AST>,
        pub args : Vec<std::boxed::Box<AST>>,
    }
    impl CallExpr {
        fn _emit(&self, c: &mut compiler::Compiler, is_tail: bool) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            let op = if is_tail { VmOpcode::OP_RETCALL }
                     else { VmOpcode::OP_CALL };
            for arg in self.args.iter().rev() { arg.emit(c); }
            if let Some(memexpr) = self.callee.as_any().downcast_ref::<MemExpr>() {
                let right = memexpr.right.as_any();
                if memexpr.is_namespace {
                    memexpr._emit(c, false);
                    c.vm.code.push(op);
                    c.vm.cpush16(self.args.len() as u16);
                } else {
                    memexpr._emit(c, true);
                    c.vm.code.push(op);
                    c.vm.cpush16((self.args.len() as u16) + 1);
                }
            } else {
                self.callee.emit(c);
                c.vm.code.push(op);
                c.vm.cpush16(self.args.len() as u16);
            }
            emit_end!(c, _smap_begin);
        }
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for CallExpr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"callee\": {:?}, \"args\": {:?}, \"op\": \"call\"}}",
                self.callee, self.args)
        }
    }
    impl AST for CallExpr {
        ast_impl!();
        fn emit(&self, c: &mut compiler::Compiler) {
            self._emit(c, false);
        }
    }
    // util for parser
    pub enum CallExprArm {
        MemExprIden(std::boxed::Box<AST>),
        MemExprNs(std::boxed::Box<AST>),
        MemExpr(std::boxed::Box<AST>),
        CallExpr(Vec<std::boxed::Box<AST>>)
    }

    // #region statement
    // ## control flows
    // ### if
    pub struct IfStatement {
        pub _span : Span,
        pub expr : std::boxed::Box<AST>,
        pub then : std::boxed::Box<AST>,
        pub alt  : Option<std::boxed::Box<AST>>,
    }
    #[cfg_attr(tarpaulin, skip)]
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
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            // Pseudo code of the generated bytecode
            //   [condition]
            //   jncond [else]
            //   [statement]
            //   jmp done
            //   [else]
            //   [done]
            self.expr.emit(c);
            c.vm.code.push(VmOpcode::OP_JNCOND); // TODO: maybe do peephole opt?
            let else_label = c.reserve_label16();
            self.then.emit(c);
            if let Some(alt) = &self.alt {
                c.vm.code.push(VmOpcode::OP_JMP);
                let done_label = c.reserve_label16();
                c.fill_label16(else_label, (c.vm.code.len() as isize - else_label as isize) as u16);
                alt.emit(c);
                c.fill_label16(done_label, (c.vm.code.len() - done_label) as u16);
            } else {
                c.fill_label16(else_label, (c.vm.code.len() as isize - else_label as isize) as u16);
            }
            emit_end!(c, _smap_begin);
        }
    }

    // ### while
    pub struct WhileStatement {
        pub _span : Span,
        pub expr : std::boxed::Box<AST>,
        pub then : std::boxed::Box<AST>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for WhileStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"expr\": {:?}, \"then\": {:?}, \"type\": \"whilestmt\"}}",
                    self.expr, self.then)
        }
    }
    impl AST for WhileStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            // pseudocode of generated bytecode:
            //   begin: jmp condition
            //   [statement]
            //   [condition]
            //   jcond [begin]
            c.vm.code.push(VmOpcode::OP_JMP);
            let begin_label = c.reserve_label16();

            let then_label = c.vm.code.len();
            c.loop_start();
            self.then.emit(c);

            c.fill_label16(begin_label, (c.vm.code.len() - begin_label) as u16);

            let next_it_pos = c.vm.code.len();
            self.expr.emit(c);
            c.vm.code.push(VmOpcode::OP_JCOND);
            c.vm.cpush16((then_label as isize - c.vm.code.len() as isize) as u16);

            c.loop_end(next_it_pos, c.vm.code.len());
            emit_end!(c, _smap_begin);
        }
    }

    // ### for
    pub struct ForStatement {
        pub _span : Span,
        pub id    : String,
        pub from  : std::boxed::Box<AST>,
        pub to    : std::boxed::Box<AST>,
        pub step  : std::boxed::Box<AST>,
        pub stmt  : std::boxed::Box<AST>,
        pub is_up : bool
    }
    #[cfg_attr(tarpaulin, skip)]
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
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
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
            let begin_label = c.reserve_label16();

            let then_label = c.vm.code.len();
            c.loop_start();
            self.stmt.emit(c);

            // step
            c.emit_get_var(self.id.clone());
            self.step.emit(c);
            c.vm.code.push(if self.is_up { VmOpcode::OP_ADD }
                           else { VmOpcode::OP_SUB });
            c.emit_set_var(self.id.clone());
            c.vm.code.push(VmOpcode::OP_POP);

            c.fill_label16(begin_label, (c.vm.code.len() - begin_label) as u16);

            // condition
            let next_it_pos = c.vm.code.len();
            c.emit_get_var(self.id.clone());
            self.to.emit(c);
            c.vm.code.push(if self.is_up { VmOpcode::OP_LT }
                           else { VmOpcode::OP_GT });
            c.vm.code.push(VmOpcode::OP_JCOND);
            c.vm.cpush16((then_label as isize - c.vm.code.len() as isize) as u16);

            c.loop_end(next_it_pos, c.vm.code.len());
            emit_end!(c, _smap_begin);
        }
    }
    // ### for in
    pub struct ForInStatement {
        pub _span : Span,
        pub id    : String,
        pub expr  : std::boxed::Box<AST>,
        pub stmt  : std::boxed::Box<AST>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for ForInStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{
                \"id\": {:?},
                \"expr\": {:?},
                \"statement\": {:?},
                \"type\": \"forinstmt\"}}",
                    self.id, self.expr, self.stmt)
        }
    }
    impl AST for ForInStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            // TODO: OP_FOR (pushes val onto stack)
            // stack: [int iterator pos]
            // code:
            //  [PUSH array]
            //  next_it: OP_FOR [end]
            //  set id
            //  [body]
            //  jmp [next_it]
            //  [end]
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;

            self.expr.emit(c);
            let next_it_label = c.vm.code.len();
            c.vm.code.push(VmOpcode::OP_FOR_IN);
            let end_label = c.reserve_label16();
            c.emit_set_var(self.id.clone());
            c.vm.code.push(VmOpcode::OP_POP);
            self.stmt.emit(c);
            c.vm.code.push(VmOpcode::OP_JMP);
            c.vm.cpush16((next_it_label as isize - c.vm.code.len() as isize) as u16);
            c.fill_label16(end_label, (c.vm.code.len() - end_label) as u16);

            emit_end!(c, _smap_begin);
        }
    }
    // ### continue statement
    pub struct ContinueStatement {
        pub _span : Span,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for ContinueStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for ContinueStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            c.vm.code.push(VmOpcode::OP_JMP);
            c.loop_continue();
            emit_end!(c, _smap_begin);
        }
    }
    // ### break
    pub struct BreakStatement {
        pub _span : Span,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for BreakStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for BreakStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            c.vm.code.push(VmOpcode::OP_JMP);
            c.loop_break();
            emit_end!(c, _smap_begin);
        }
    }

    // ## other
    // #### fn
    pub struct FunctionStatement {
        pub _span : Span,
        def: FunctionDefinition
    }
    impl FunctionStatement {
        pub fn new(def : FunctionDefinition, span: Span) -> FunctionStatement {
            FunctionStatement { _span: span, def: def }
        }

        pub fn def(&self) -> &FunctionDefinition {
            &self.def
        }
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for FunctionStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for FunctionStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            self.def.emit(c);

            // set var
            c.emit_set_var_fn(self.def.id.as_ref().unwrap().clone());
            c.vm.code.push(VmOpcode::OP_POP);
        }
    }
    // #### return
    pub struct ReturnStatement {
        pub _span : Span,
        pub expr : Option<std::boxed::Box<AST>>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for ReturnStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for ReturnStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            if !c.is_in_function() {
                panic!("not in function!"); // TODO
            }
            match &self.expr {
                Some(expr) => {
                    if let Some(expr) = expr.as_any().downcast_ref::<CallExpr>() {
                        expr._emit(c, true);
                    } else if let Some(expr) = expr.as_any().downcast_ref::<CondExpr>() {
                        expr._emit(c, true);
                    } else {
                        expr.emit(c);
                    }
                },
                None => c.vm.code.push(VmOpcode::OP_PUSH_NIL)
            }
            c.vm.code.push(VmOpcode::OP_RET);
            emit_end!(c, _smap_begin);
        }
    }

    // ### record statement
    pub struct RecordStatement {
        pub _span : Span,
        def: RecordDefinition
    }
    impl RecordStatement {
        pub fn new(def : RecordDefinition, span: Span) -> RecordStatement {
            RecordStatement { _span: span, def: def }
        }

        pub fn def(&self) -> &RecordDefinition {
            &self.def
        }
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for RecordStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for RecordStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            self.def.emit(c);

            // set var
            c.emit_set_var(self.def.id.as_ref().unwrap().clone());
            c.vm.code.push(VmOpcode::OP_POP);
        }
    }

    // ### try
    pub struct TryStatement {
        pub _span : Span,
        pub stmts : Vec<std::boxed::Box<AST>>,
        pub cases : Vec<std::boxed::Box<CaseStatement>>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for TryStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for TryStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            c.vm.code.push(VmOpcode::OP_PUSH_NIL);
            let mut cases_to_fill : Vec<usize> = Vec::new();
            for case in &self.cases {
                // function will take in 1 arg if id is set
                c.vm.code.push(VmOpcode::OP_DEF_FUNCTION_PUSH);
                c.vm.cpush16(if case.id.is_some() { 1 } else { 0 });
                let body_start = c.reserve_label16();
                // id
                if let Some(id) = &case.id {
                    let id = id.as_any().downcast_ref::<Identifier>()
                        .unwrap().val.clone();
                    c.emit_set_var(id);
                    c.vm.code.push(VmOpcode::OP_POP);
                }
                // body
                for s in &case.stmts {
                    s.emit(c);
                }
                c.vm.code.push(VmOpcode::OP_EXFRAME_RET);
                cases_to_fill.push(c.reserve_label16());
                // end
                c.fill_label16(body_start, (c.vm.code.len() - body_start) as u16);
                // exception type
                case.etype.emit(c);
            }
            c.vm.code.push(VmOpcode::OP_TRY);
            for s in &self.stmts {
                s.emit(c);
            }
            for hole in cases_to_fill {
                c.fill_label16(hole, (c.vm.code.len() - hole) as u16);
            }
            emit_end!(c, _smap_begin);
        }
    }
    // #### case
    pub struct CaseStatement {
        pub _span : Span,
        pub etype : std::boxed::Box<AST>,
        pub id    : Option<std::boxed::Box<AST>>,
        pub stmts : Vec<std::boxed::Box<AST>>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for CaseStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for CaseStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            // this is already generated by try statement
            unreachable!()
        }
    }
    // #### raise
    pub struct RaiseStatement {
        pub _span : Span,
        pub expr : std::boxed::Box<AST>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for RaiseStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for RaiseStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            self.expr.emit(c);
            c.vm.code.push(VmOpcode::OP_RAISE);
        }
    }

    // ### expr
    pub struct ExprStatement {
        pub _span : Span,
        pub expr : std::boxed::Box<AST>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for ExprStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"expr\": {:?}, \"type\": \"exprstmt\"}}",
                    self.expr)
        }
    }
    impl AST for ExprStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            self.expr.emit(c);
            c.vm.code.push(VmOpcode::OP_POP);
            emit_end!(c, _smap_begin);
        }
    }

    // ### use statement
    pub struct UseStatement {
        pub _span : Span,
        pub path : String,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for UseStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            unimplemented!()
        }
    }
    impl AST for UseStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            c.vm.code.push(VmOpcode::OP_USE);
            c.vm.cpushs(self.path.clone());
            emit_end!(c, _smap_begin);
        }
    }

    // ### block
    pub struct BlockStatement {
        pub _span : Span,
        pub stmts : Vec<std::boxed::Box<AST>>,
    }
    #[cfg_attr(tarpaulin, skip)]
    impl fmt::Debug for BlockStatement {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{{\"stmts\": {:?}, \"type\": \"blockstmt\"}}",
                    self.stmts)
        }
    }
    impl AST for BlockStatement {
        ast_impl!();
        fn emit(&self, c : &mut compiler::Compiler) {
            emit_begin!(self, c); let _smap_begin = c.smap.len() - 1;
            for stmt in &self.stmts {
                stmt.emit(c);
            }
            emit_end!(c, _smap_begin);
        }
    }
    // #endregion

}

// parser
pub mod grammar {
    macro_rules! boxed {
        ($x:ident, $ps:expr, $pe:expr, $($key:ident: $value:expr),*) => (
            Box::new(ast::$x {
                _span: ($ps, $pe),
                $($key: $value),*
            })
        )
    }
    include!(concat!(env!("OUT_DIR"), "/parser.rs"));
}

// utils
pub fn pos_to_line(input: &str, pos: usize) -> (usize, usize) {
    let before = &input[..pos];
    let line = before.as_bytes().iter().filter(|&&c| c == b'\n').count() + 1;
    let col = before.chars().rev().take_while(|&c| c != '\n').count() + 1;
    (line, col)
}