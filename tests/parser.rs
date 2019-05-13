extern crate haru;

#[cfg(test)]
pub mod parser_tests {

    use haru::ast::ast;
    use haru::ast::grammar;

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

    // #region simple comments
    #[test]
    fn single_line_comment() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("// Test");
        assert_eq!(progast.len(), 0);
    }
    #[test]
    fn multiline_comment() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("/*
multiline
*/");
        assert_eq!(progast.len(), 0);
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
if 0 then 1
");
        let stmt = cast_box!(progast[0], ast::IfStatement);
        assert_eq!(cast_box!(stmt.expr, ast::IntLiteral).val, 0);
        assert_eq!(cast_box!(cast_box!(stmt.then, ast::ExprStatement).expr, ast::IntLiteral).val, 1);
    }

    #[test]
    fn if_else_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
if 0 then 1
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
while 0 then 1
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
        assert_eq!(stmt.id, "i");
        assert_eq!(cast_box!(stmt.from, ast::IntLiteral).val, 0);
        assert_eq!(cast_box!(stmt.to, ast::IntLiteral).val, 100);
    }
    #[test]
    fn for_stmt_with_if() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!(r#"
for i=0 to 100 begin
    if i mod 3 == 0 and i mod 5 == 0 then print("Fizzbuzz\n")
    else if i mod 3 == 0 then print("Fizz\n")
    else if i mod 5 == 0 then print("Buzz\n")
    else print(i, "\n")
end
"#);
        cast_box!(progast[0], ast::ForStatement);
    }
    // #endregion

    // #region try statement
    #[test]
    fn try_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
try
    0
case Int as a
end
");
        let stmt = cast_box!(progast[0], ast::TryStatement);
        let then_stmt = cast_box!(stmt.stmts[0], ast::ExprStatement);
        assert_eq!(cast_box!(then_stmt.expr, ast::IntLiteral).val, 0);
        assert_eq!(cast_box!(stmt.cases[0].etype, ast::Identifier).val, "Int");
        assert!(stmt.cases[0].id.is_some());
    }

    #[test]
    fn try_stmt_multiple_cases() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
try
case Int as a
case String as a
case Float as a
end
");
        let stmt = cast_box!(progast[0], ast::TryStatement);
        assert!(stmt.cases.len() == 3);
    }
    // #endregion

    // #region function statement
    #[test]
    fn fn_stmt() {
        let progast : Vec<std::boxed::Box<ast::AST>> = parse_ast_statement!("
function X(y) begin

end
");
        let stmt = cast_box!(progast[0], ast::FunctionStatement);
        assert_eq!(stmt.def().id, Some("X".to_string()));
        assert_eq!(stmt.def().args.len(), 1);
        assert_eq!(stmt.def().args[0], "y");
    }
    // #endregion

    // #region nested
    #[test]
    fn nested_stmt() {
        parse_ast_statement!("
function X(y) begin
    if x == 0 begin

    end
end
");
    }

    #[test]
    fn nested_stmt_2() {
        parse_ast_statement!("
function X(y) begin
    if x == 0 begin
        if x == 0 then 1
    end
end
");
    }

    #[test]
    fn nested_fn() {
        parse_ast_statement!("
function outer() begin
    function inner() begin
    end
    inner()
end
outer()
");
    }
    // #endregion

}
