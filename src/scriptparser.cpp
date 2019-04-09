#include "scriptparser.h"
#include <utility>
#include <functional>
#include <memory>

using namespace Hana;

/*
program = block "." .

block = { } statement .

struct_stmt = ident ident NL

statement = [ ident "=" expression NL
              | "function" ident "(" { ident "," } ")" statement NL
              | "record" ident struct_stmt { struct_stmt } "end"
              | "if" condition NL statement ["else" statement]
              | "while" condition statement NL
              | "for" ident "=" condition ("to"|"downto") condition
                {"step" condition} statement NL
              | "begin" statement { statement } "end" NL
            ]

logical = condition ("and" | "or") logical
condition = expression ("=="|"!="|">"|"<"|">="|"<=") expression
expression = [ "+"|"-"] term { ("+"|"-") term}
term = factor {("*"|"/") factor}
factor = ident | number | "(" expression ")"
 */

Parser::Token ScriptParser::next() {
    char c;
    const auto skip_un = [&]() {
        while((c = f.peek()) != EOF) {
            if(c == ' ' || c == '\t') {
                c = f.get();
            } else if (c == '/') {
                const auto pos = f.tellg();
                f.get();
                if(f.get() == '/') {
                    while((c = f.peek()) != EOF && c != '\n')
                        f.get();
                } else {
                    f.seekg(pos);
                    return; // this is an operator, so it should be handled below
                }
            } else break;
        }
    };
    skip_un();
    std::string token;
    LOG("c=",c);
    if(c == EOF) {
        LOG("EOF");
        f.get();
        return Token();
    } else if(c == '\'') {
        f.get();
        std::string str;
        while((c = f.peek()) != EOF) {
            if(c == '\'') {
                f.get();
                break;
            } else if(c == '\\') {
                f.get();
            }
            str += f.get();
        }
        LOG("LIT", str);
        return Token(str, Token::Type::STRLITERAL);
    } else if(c == '\n') {
        f.get();
        lines++;
        while((c = f.peek()) != EOF) {
            skip_un();
            if(c == '\n') {
                f.get();
                lines++;
            } else break;
        }
        return Token(true);
    } else if(isdigit(c)) {
        while((c = f.peek()) != EOF && isdigit(c))
            token += f.get();
        if(f.peek() == '.') {
            token += f.get();
            while((c = f.peek()) != EOF && isdigit(c))
                token += f.get();
            return Token(std::stof(token));
        }
        return Token(std::stoi(token));
    }
#define is_onech_op(c) \
    (c == '=' || c == '!' || c == '<' || c == '>' || c == '(' || c == ')' || \
    c == '+' || c == '-' || c == '/' || c == '*' || c == '{' || c == '}' || \
    c == '.' || c == ',')
    else if(is_onech_op(c)) {
        // "=="|"!="|">"|"<"|">="|"<="|"+"|"-"|"/"|"*"
        c = f.get();
        const auto prev = f.tellg();
        const char nc = f.get();
#define twoch(str) \
        if(c == str[0] && nc == str[1]) return Token(str, Token::Type::OPERATOR);

        twoch("==")
        else twoch("!=")
        else twoch(">=")
        else twoch("<=")
        else twoch("+=")
        else twoch("-=")
        else {
            f.seekg(prev);
            token = c;
            return Token(token, Token::Type::OPERATOR);
        }

    } else {
        while((c = f.peek()) != EOF && !isspace(c) && !is_onech_op(c))
            token += f.get();
        return Token(token);
    }
}

AST::AST *ScriptParser::parse_factor() {
    auto token = next();
    if(token.type == Token::Type::OPERATOR && token.strv == "(") {
        LOG("(");
        auto expr = parse_expression();
        nextop(")");
        return expr;
    } else if(token.type == Token::Type::STRING) {
        return new AST::Identifier(token.strv);
    } else if(token.type == Token::Type::STRLITERAL) {
        return new AST::Constant(Value(token.strv));
    } else if(token.type == Token::Type::INTEGER) {
        return new AST::Constant(Value(token.intv));
    } else if(token.type == Token::Type::FLOAT) {
        return new AST::Constant(Value(token.floatv));
    }
    LOG(token.type);
    FATAL("Parser error", "Expected factor token: ", lines); // TODO
}

AST::AST *ScriptParser::parse_call() {
    auto factor = parse_factor();

    fsave();
    auto op = next();
    if(op.type == Token::Type::OPERATOR &&
        (op.strv == "(" || op.strv == ".")) {
        AST::AST *expr = factor;
        LOG(op.strv);
        do {

            if(op.strv == "(") {
                fpop();
                expr = new AST::CallExpression(factor);

                fsave();
                op = next();
                if(op.type == Token::Type::OPERATOR && op.strv == ")") {
                    fpop();
                    goto next;
                } else {
                    fload();
                    static_cast<AST::CallExpression*>(expr)->arguments.push_back(std::unique_ptr<AST::AST>(parse_expression()));
                }

                while(!f.eof()) {
                    op = next();
                    if(op.type == Token::Type::OPERATOR) {
                        if(op.strv == ",") {
                            static_cast<AST::CallExpression*>(expr)->arguments.push_back(std::unique_ptr<AST::AST>(parse_expression()));
                        } else if(op.strv == ")") {
                            LOG("break", f.tellg());
                            goto next;
                        } else
                            FATAL("Parser error", "Expected , or ) operator");
                    } else
                        FATAL("Parser error", "Expected operator");
                }
            } else if(op.strv == ".") {
                LOG("member");
                fpop();
                const auto id = nexts();
                expr = new AST::MemberExpression(expr, new AST::Constant(Value(id)));
            } else {
                LOG("LOAD");
                fload();
                return expr;
            }

        next:
            LOG("save");
            fsave();
            op = next();

        } while(!f.eof());

        fload();
        return expr;
    } else {
        fload();
        return factor;
    }
#if 0




if(op.strv == "(") {
    expr = new AST::CallExpression();
}

auto expr = new

fpop();

expr->arguments.push_back(std::unique_ptr<AST::AST>(parse_expression()));

while(!f.eof()) {
    token = next();
    LOG(token.strv);
    if(token.type == Token::Type::OPERATOR) {
        if(token.strv == ",") {
            expr->arguments.push_back(std::unique_ptr<AST::AST>(parse_expression()));
} else if(token.strv == ")") {
    break;
} else
    FATAL("Parser error", "Expected , or ) operator");
} else
    FATAL("Parser error", "Expected operator");
}

return expr;
}/* else if(op.type == Token::Type::OPERATOR && op.strv == ".") {
LOG("member expression");

// member expression
fpop();
auto memberexpr = new AST::MemberExpression();
memberexpr->id.push_back(token.strv);
memberexpr->id.push_back(nexts());
while(!f.eof()) {
    fsave();
    token = next();
    if(token.type == Token::Type::OPERATOR && token.strv == ".") {
        fpop();
        memberexpr->id.push_back(nexts());
} else {
    fload();
    break;
}
}
return memberexpr;

} */
#endif
}

AST::AST *ScriptParser::parse_unary() {
    fsave();
    auto token = next();
    AST::UnaryExpression::OpType ot = AST::UnaryExpression::OpType::NONE;
    if(token.type == Token::Type::OPERATOR) {
        if(token.strv == "+")
            ot = AST::UnaryExpression::OpType::POS;
        else if(token.strv == "-")
            ot = AST::UnaryExpression::OpType::NEG;
        else goto non_unary;
    } else if (token.type == Token::Type::STRING) {
        if(token.strv == "not")
            ot = AST::UnaryExpression::OpType::NOT;
        else goto non_unary;
    } else goto non_unary;
    return new AST::UnaryExpression(ot, parse_call());

non_unary:
    fload();
    return parse_call();
}

AST::AST *ScriptParser::parse_expression() {
    LOG("expr");
    return parse_assignment();
}

AST::AST *ScriptParser::parse_assignment() {
    fsave();
    auto token = next();
    const auto optype = [](const std::string &op) {
        if(op == "=") return AST::BinaryExpression::OpType::SET;
        else if(op == "+=") return AST::BinaryExpression::OpType::ADDS;
        else if(op == "-=") return AST::BinaryExpression::OpType::SUBS;
        else if(op == "*=") return AST::BinaryExpression::OpType::MULS;
        else if(op == "/=") return AST::BinaryExpression::OpType::DIVS;
        return AST::BinaryExpression::OpType::NONE;
    };
    if(token.type == Token::Type::STRING) {
        floadn();
        LOG("assign!!");
        auto left = parse_call();
        auto op = next();
        AST::BinaryExpression::OpType opt;
        if((opt = optype(op.strv)) != AST::BinaryExpression::OpType::NONE) {
            LOG("assignment");
            if (left->type() != AST::Type::MEMBER_EXPR &&
                left->type() != AST::Type::IDENTIFIER)
                FATAL("Parser error", "Invalid left-hand side argument");
            fpop();
            auto right = parse_expression();
            return new AST::BinaryExpression(left, right, opt);
        }
    }

    fload();
    LOG(f.tellg());
    return parse_binexpr();
}

AST::AST *ScriptParser::parse_binexpr() {
#define build_binary_expression(name, down, optype) \
    [&]() -> AST::AST* { \
        LOG("binexpr: ", name); \
        AST::AST *left = down(); \
        fsave(); \
        Token token = next(); \
        AST::BinaryExpression::OpType ot;  \
        LOG("type ", token.type);\
        if ((ot = optype(token.strv)) != AST::BinaryExpression::OpType::NONE) { \
            LOG("operator", token.strv); \
            fpop(); \
            AST::BinaryExpression *binexpr = new AST::BinaryExpression(); \
            binexpr->left = std::unique_ptr<AST::AST>(left); \
            binexpr->op = ot; \
            binexpr->right = std::unique_ptr<AST::AST>(down()); \
            while(!f.eof()) { \
                fsave(); \
                token = next(); \
                LOG("operator", token.strv); \
                if ((ot = optype(token.strv)) != AST::BinaryExpression::OpType::NONE) { \
                    fpop(); \
                    auto right = down(); \
                    binexpr = new AST::BinaryExpression(binexpr, right, ot); \
                } else { \
                    fload(); \
                    break; \
                } \
            } \
            return binexpr; \
        } else { \
            fload(); \
            return left; \
        } \
    }

    const auto optype_fn = [](const std::initializer_list<std::pair<std::string, AST::BinaryExpression::OpType>> list) {
        return [list](const std::string &str) -> AST::BinaryExpression::OpType {
            for(auto &pair : list) {
                LOG(str, pair.first);
                if(str == pair.first)
                    return pair.second;
            }
            return AST::BinaryExpression::OpType::NONE;
        };
    };

    const auto parse_term = build_binary_expression("term",
                                              [&]() { return parse_unary(); },
                                              optype_fn({
                                                {"mod", AST::BinaryExpression::OpType::MOD},
                                                {"*", AST::BinaryExpression::OpType::MUL},
                                                {"/", AST::BinaryExpression::OpType::DIV},
                                              }));
    const auto parse_addition = build_binary_expression("addition",
                                              [&]() { return parse_term(); },
                                              optype_fn({
                                                {"+", AST::BinaryExpression::OpType::ADD},
                                                {"-", AST::BinaryExpression::OpType::SUB}
                                              }));
    const auto parse_comparison = build_binary_expression("comparison",
                                              [&]() { return parse_addition(); },
                                              optype_fn({
                                                {"==", AST::BinaryExpression::OpType::EQ},
                                                {"!=", AST::BinaryExpression::OpType::NEQ},
                                                {">", AST::BinaryExpression::OpType::GT},
                                                {"<", AST::BinaryExpression::OpType::LT},
                                                {">=", AST::BinaryExpression::OpType::GEQ},
                                                {"<=", AST::BinaryExpression::OpType::LEQ},
                                              }));
    const auto parse_logical = build_binary_expression("logical",
                                              [&]() { return parse_comparison(); },
                                              optype_fn({
                                                {"and", AST::BinaryExpression::OpType::AND},
                                                {"or", AST::BinaryExpression::OpType::OR},
                                              }));
    return parse_logical();

#undef build_binary_expression
}

std::vector<std::string> ScriptParser::parse_function_arguments() {
    std::vector<std::string> arguments;
    nextop("(");
    auto token = next();
    if(token.type == Token::Type::OPERATOR && token.strv == ")") {
        return arguments;
    } else if(token.type == Token::Type::STRING) {
        LOG(token.strv);
        arguments.push_back(token.strv);
        while(!f.eof()) {
            token = next();
            if(token.type == Token::Type::OPERATOR) {
                if(token.strv == ",") {
                    arguments.push_back(nexts());
                } else if(token.strv == ")") {
                    break;
                } else
                    FATAL("Parser error", "Expected , or ) operator");
            } else
                FATAL("Parser error", "Expected operator");
        }
        return arguments;
    } else
        FATAL("Parser error", "Expected function arguments");
}

AST::AST *ScriptParser::parse_statement() {
    fsave();
//     LOG("statement");
    Token token = next();
//     LOG("STR=[",token.strv,"]");
//     LOG(token.type);

    if(token.type == Token::Type::NONE) {
        ended = true;
        return nullptr;
    } else if(token.type == Token::Type::NEWLINE) {
        fpop();
        return nullptr;
    } else if(token.type == Token::Type::STRING) {
        if(token.strv == "end") {
            FATAL("Parser error", "end is not in block statement");
            return nullptr;
        } else if(token.strv == "return") {
            fpop();
            fsave();
            auto token = next();
            if(token.type == Token::Type::NEWLINE) {
                fpop();
                return new AST::ReturnStatement();
            } else {
                fload();
                return new AST::ReturnStatement(parse_expression());
            }
        } else if(token.strv == "function") {
            fpop();
//             LOG("function");
            auto id = nexts();
            LOG("id: ", id);
            auto stmt = new AST::FunctionStatement(id);
            stmt->arguments = parse_function_arguments();
//             LOG("end arg");
            auto fstmt = parse_statement();
            if(fstmt != nullptr)
                stmt->statement = std::unique_ptr<AST::AST>(fstmt);
//             LOG("end function");
            return stmt;
        } else if(token.strv == "record") {
            LOG("struct");
            auto token = next();
            if(token.type != Token::Type::STRING) {
                goto expression_stmt;
            }
            fpop();
            auto id = token.strv;
            auto stmt = new AST::StructStatement(id);
            nextnl();
            while(!f.eof()) {
                fsave();
                auto token = next();
                LOG(token.strv);
                if(token.type == Token::Type::STRING) {
                    if(token.strv == "end") {
                        nextnl();
                        break;
                    } else {
                        stmt->dict[nexts()] = token.strv;
                        nextnl();
                    }
                } else
                    FATAL("Parser error", "expected struct statement");
            }
            return stmt;
        } else if(token.strv == "if") {
            fpop();
            LOG("conditional stmt");
            auto expr = parse_expression();
            auto stmt = parse_statement();
            auto ifstmt = new AST::IfStatement(expr, stmt);
            fsave();
            auto token = next();
            LOG(token.type, token.strv);
            if(token.type == Token::Type::STRING && token.strv == "else") {
                LOG("else");
                fpop();
                ifstmt->alt = std::unique_ptr<AST::AST>(parse_statement());
            } else {
                fload();
            }
            LOG("End if");
            return ifstmt;
        } else if(token.strv == "while") {
            fpop();

            auto expr = parse_expression();
            nextop(")");
            auto stmt = parse_statement();
            return new AST::WhileStatement(expr, stmt);
        } else if(token.strv == "for") {
            fpop();

            auto id = nexts();
            nextop("=");
            auto from = parse_expression();
            auto dir = nexts();
            auto to = parse_expression();

            fsave();
            auto token = next();
            if(token.type == Token::Type::STRING && token.strv == "step") {
                fpop();
                return new AST::ForStatement(id, from, to, parse_expression(), parse_statement());
            } else {
                fload();
                if(dir == "to")
                    return new AST::ForStatement(id, from, to, 1, parse_statement());
                else if(dir == "downto")
                    return new AST::ForStatement(id, from, to, -1, parse_statement());
                else FATAL("Parser error", "Expected to/downto");
            }

        } else if(token.strv == "begin") {
            fpop();
            // block statement
//             LOG("Block statement");
            nextnl();
            auto blk = new AST::BlockStatement();
            while(!f.eof()) {
                fsave();
                auto token = next();
                //LOG(token.strv);
                if(token.type == Token::Type::STRING && token.strv == "end") {
//                     LOG("END BLOCK");
                    fpop();
                    nextnl();
                    break;
                } else {
                    fload();
                    auto stmt = parse_statement();
                    if(stmt != nullptr)
                        blk->statements.emplace_back(stmt);
                }
            }
            return blk;
        } else {
            goto expression_stmt;
        }
    } else {
    expression_stmt:
        // expression statement
        fload();
        LOG("expression stmt");
        auto expr = parse_expression();
        LOG("end expr");
        nextnl();
        return new AST::ExpressionStatement(expr);
    }
}

AST::AST *ScriptParser::parse_block() {
    auto block = new AST::Block();
    while(!ended) {
        auto stmt = parse_statement();
        if(stmt != nullptr)
            block->statements.emplace_back(stmt);
    }
    return block;
}

AST::AST *ScriptParser::parse() {
    return parse_block();
}
