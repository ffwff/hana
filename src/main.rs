use std::io::Read;

// ast
pub mod ast {
    use std::fmt;
    use std::any::Any;

    macro_rules! as_any {
        () => (fn as_any(&self) -> &dyn Any { self });
    }

    pub trait AST : fmt::Debug {
        fn as_any(&self) -> &dyn Any;
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
    }

    // # expressions
    // ## binexpr
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
    }

    // # statement
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
    }

}

// parser
mod grammar {
    include!(concat!(env!("OUT_DIR"), "/parser.rs"));
}

// unit testing
#[cfg(test)]
mod parser_tests {

    use super::ast;
    use super::grammar;

    macro_rules! parse_ast_statement {
        ($x:expr) => (grammar::start($x).unwrap());
    }
    macro_rules! cast_box {
        ($x:expr, $y:ty) => ($x.as_any().downcast_ref::<$y>().unwrap());
    }

    // #region simple values
    #[test]
    fn simple_id() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("a");
        assert_eq!(progast.len(), 1);
        let stmt = cast_box!(progast[0], ast::ExprStatement);
        assert_eq!(cast_box!(stmt.expr, ast::Identifier).val, "a".to_string());
    }

    #[test]
    fn simple_str() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("'a'");
        assert_eq!(progast.len(), 1);
        let stmt = cast_box!(progast[0], ast::ExprStatement);
        assert_eq!(cast_box!(stmt.expr, ast::StrLiteral).val, "a".to_string());
    }

    #[test]
    fn simple_int() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("125");
        assert_eq!(progast.len(), 1);
        let stmt = cast_box!(progast[0], ast::ExprStatement);
        assert_eq!(cast_box!(stmt.expr, ast::IntLiteral).val, 125);
    }

    #[test]
    fn simple_float() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("12.6");
        assert_eq!(progast.len(), 1);
        let stmt = cast_box!(progast[0], ast::ExprStatement);
        assert_eq!(cast_box!(stmt.expr, ast::FloatLiteral).val, 12.6);
    }
    // #endregion

    // #region member expr
    #[test]
    fn member_expr_dot() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("a.b");
        assert_eq!(progast.len(), 1);
        let stmt = cast_box!(progast[0], ast::ExprStatement);
        let memexpr = cast_box!(stmt.expr, ast::MemExpr);
        assert_eq!(cast_box!(memexpr.left, ast::Identifier).val, "a");
        assert_eq!(cast_box!(memexpr.right, ast::Identifier).val, "b");
    }

    #[test]
    fn member_expr_bracket() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("a['b']");
        assert_eq!(progast.len(), 1);
        let stmt = cast_box!(progast[0], ast::ExprStatement);
        let memexpr = cast_box!(stmt.expr, ast::MemExpr);
        assert_eq!(cast_box!(memexpr.left, ast::Identifier).val, "a");
        assert_eq!(cast_box!(memexpr.right, ast::StrLiteral).val, "b");
    }
    // #endregion

    // #region call expr
    #[test]
    fn call_expr() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("a(1,2)");
        assert_eq!(progast.len(), 1);
        let stmt = cast_box!(progast[0], ast::ExprStatement);
        let callexpr = cast_box!(stmt.expr, ast::CallExpr);
        assert_eq!(cast_box!(callexpr.callee, ast::Identifier).val, "a");
        assert_eq!(callexpr.args.len(), 2);
        assert_eq!(cast_box!(callexpr.args[0], ast::IntLiteral).val, 1);
        assert_eq!(cast_box!(callexpr.args[1], ast::IntLiteral).val, 2);
    }
    // #endregion

    // #region bin expr
    #[test]
    fn bin_expr() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("a + b");
        assert_eq!(progast.len(), 1);
        let stmt = cast_box!(progast[0], ast::ExprStatement);
        let binexpr = cast_box!(stmt.expr, ast::BinExpr);
        assert_eq!(cast_box!(binexpr.left, ast::Identifier).val, "a");
        assert_eq!(cast_box!(binexpr.right, ast::Identifier).val, "b");
        assert_eq!(binexpr.op, ast::BinOp::Add);
    }
    // #endregion

    // #region block statement
    #[test]
    fn block_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
begin
    1
    2
end
");
        let stmt = cast_box!(progast[0], ast::BlockStatement);
        assert_eq!(stmt.stmts.len(), 2);
        assert_eq!(cast_box!(cast_box!(stmt.stmts[0], ast::ExprStatement).expr, ast::IntLiteral).val, 1);
        assert_eq!(cast_box!(cast_box!(stmt.stmts[1], ast::ExprStatement).expr, ast::IntLiteral).val, 2);
    }
    // #endregion

    // #region if statement
    #[test]
    fn if_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
if 0 1
");
        let stmt = cast_box!(progast[0], ast::IfStatement);
        assert_eq!(cast_box!(stmt.expr, ast::IntLiteral).val, 0);
        assert_eq!(cast_box!(cast_box!(stmt.then, ast::ExprStatement).expr, ast::IntLiteral).val, 1);
    }

    #[test]
    fn if_else_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
if 0 1
else 2
");
        let stmt = cast_box!(progast[0], ast::IfStatement);
        assert_eq!(cast_box!(stmt.expr, ast::IntLiteral).val, 0);
        assert_eq!(cast_box!(cast_box!(stmt.then, ast::ExprStatement).expr, ast::IntLiteral).val, 1);
        assert!(stmt.alt.is_some());
    }
    // #endregion

    // #region while statement
    #[test]
    fn while_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
while 0 1
");
        let stmt = cast_box!(progast[0], ast::WhileStatement);
        assert_eq!(cast_box!(stmt.expr, ast::IntLiteral).val, 0);
        assert_eq!(cast_box!(cast_box!(stmt.then, ast::ExprStatement).expr, ast::IntLiteral).val, 1);
    }
    // #endregion

    // #region for statement
    #[test]
    fn for_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
for i=0 to 100 begin

end
");
        let stmt = cast_box!(progast[0], ast::ForStatement);
        assert_eq!(cast_box!(stmt.id, ast::Identifier).val, "i");
        assert_eq!(cast_box!(stmt.from, ast::IntLiteral).val, 0);
        assert_eq!(cast_box!(stmt.to, ast::IntLiteral).val, 100);
    }
    // #endregion

    // #region try statement
    /*
    #[test]
    fn try_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("

");
        let stmt = cast_box!(progast[0], ast::ForStatement);
        assert_eq!(cast_box!(stmt.id, ast::Identifier).val, "i");
        assert_eq!(cast_box!(stmt.from, ast::IntLiteral).val, 0);
        assert_eq!(cast_box!(stmt.to, ast::IntLiteral).val, 100);
    }*/
    // #endregion

    // #region function statement
    #[test]
    fn fn_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
function X(y) begin

end
");
        let stmt = cast_box!(progast[0], ast::FunctionStatement);
        assert_eq!(stmt.id, "X");
        assert_eq!(stmt.args.len(), 1);
        assert_eq!(stmt.args[0], "y");
    }
    // #endregion

}

fn main() {
    let args : Vec<String> = std::env::args().collect();
    let mut file = match std::fs::File::open(&args[1]) {
        Err(e) => { panic!("Error opening file: {}", e); }
        Ok(f) => f
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(e) => { panic!("Error reading file: {}", e); }
        Ok(_) => { }
    };
    match grammar::start(&s) {
        Ok(r) => println!("{:?}", r),
        Err(e) => println!("Parsed error: {}", e)
    };
}