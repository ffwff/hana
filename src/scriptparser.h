#include "parser.h"
#include "ast.h"

namespace Hana {

class ScriptParser : public Parser {

private:
    Token next() override;
    void nextop(const std::string &s) {
        Token t = next();
        if(t.type != Token::OPERATOR)
            FATAL("Lexer error", "Expected operator value ", s, " (line: ", lines, ")", " got ", t.type);
        assert(t.strv == s);
    };

    AST::AST *parse_call();
    AST::AST *parse_factor();
    AST::AST *parse_unary();
    AST::AST *parse_expression();
    AST::AST *parse_assignment();
    AST::AST *parse_binexpr();
    AST::AST *parse_conditional_expr();
    AST::AST *parse_block();
    AST::AST *parse_record(bool is_expr=false);
    std::vector<std::string> parse_function_arguments();
    AST::AST *parse_statement();
    void parse_procedure();

public:
    AST::AST *parse();

};

}
