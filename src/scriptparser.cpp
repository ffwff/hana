#include "scriptparser.h"
#include <utility>
#include <functional>
#include <memory>

using namespace Hana;

Parser::Token ScriptParser::next() {
    char c;
    const auto skip_un = [&]() {
        while((c = f.peek()) != EOF) {
            if(c == ' ' || c == '\t' || c == '\r') {
                c = f.get();
            } else if (c == '/') {
                const auto pos = f.tellg();
                f.get();
                char c = f.get();
                if(c == '/') {
                    while((c = f.peek()) != EOF && c != '\n')
                        f.get();
                } else if(c == '*') {
                    while((c = f.peek()) != EOF) {
                        f.get();
                        const char c1 = f.get();
                        if(c == '*' && c1 == '/')
                            break;
                        else f.unget();
                    }
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
    } else if(c == '\'' || c == '"') {
        const char start = c;
        f.get();
        std::string str;
        while((c = f.peek()) != EOF) {
            if(c == start) {
                f.get();
                break;
            } else if(c == '\\') {
                f.get();
                const char esc = f.get();
                if(esc == 'n') str += '\n';
                else if(esc == 'r') str += '\r';
                else str += esc;
                continue;
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
    (c == '=' || c == '<' || c == '>' || c == '(' || c == ')' || \
    c == '+' || c == '-' || c == '/' || c == '*' || c == '[' || c == ']' || \
    c == '.' || c == ',' || c == '?' || c == '!' || c == ':')
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
        token += f.get();
        while((c = f.peek()) != EOF && !isspace(c) && (!is_onech_op(c) || c == '?'))
            token += f.get();
        return Token(token);
    }
}

AST::AST *ScriptParser::parse_factor() {
    const auto token = next();
    if(token.type == Token::Type::OPERATOR) {
        if(token.strv == "(") {
            LOG("(");
            auto expr = parse_expression();
            nextop(")");
            return expr;
        } else if(token.strv == "[") {
            auto expr = new AST::Array();
            fsave();
            auto op = next();
            if(op.strv == "]")
                return expr;
            fload();
            expr->values.emplace_back(parse_expression());
            do {
                op = next();
                if(op.strv == ",") {
                    expr->values.emplace_back(parse_expression());
                } else if(op.strv == "]") {
                    break;
                }
            } while(!f.eof());
            return expr;
        }
    } else if(token.type == Token::Type::STRING) {
        if(token.strv == "record") {
            auto s = parse_record(true);
            if(s) return s;
        } else if(token.strv == "function") {
            auto s = parse_function(EXPR);
            if(s) return s;
        }
        LOG("NEW: ", token.strv);
        return new AST::Identifier(token.strv);
    } else if(token.type == Token::Type::STRLITERAL) {
        return new AST::StrLiteral(token.strv);
    } else if(token.type == Token::Type::INTEGER) {
        return new AST::IntLiteral(token.intv);
    } else if(token.type == Token::Type::FLOAT) {
        return new AST::FloatLiteral(token.floatv);
    }
    LOG(token.type);
    throw ParserError("Expected factor token: ", lines); // TODO
}

AST::AST *ScriptParser::parse_call() {
    auto factor = parse_factor();

    fsave();
    auto op = next();
    if(op.type == Token::Type::OPERATOR &&
        (op.strv == "(" || op.strv == "." || op.strv == "[")) {
        AST::AST *expr = factor;
        LOG(op.strv);
        do {

            if(op.strv == "(") {
                fpop();
                if(expr->type() == AST::Type::MEMBER_EXPR)
                    static_cast<AST::MemberExpression*>(expr)->is_called = true;
                expr = new AST::CallExpression(expr);

                fsave();
                op = next();
                if(op.type == Token::Type::OPERATOR && op.strv == ")") {
                    fpop();
                    goto next;
                } else {
                    fload();
                    static_cast<AST::CallExpression*>(expr)->arguments.emplace_back(parse_expression());
                }

                while(!f.eof()) {
                    op = next();
                    if(op.type == Token::Type::OPERATOR) {
                        if(op.strv == ",") {
                            static_cast<AST::CallExpression*>(expr)->arguments.emplace_back(parse_expression());
                        } else if(op.strv == ")") {
                            LOG("break", f.tellg());
                            goto next;
                        } else
                            throw ParserError("Expected , or ) operator");
                    } else
                        throw ParserError("Expected operator");
                }
            } else if(op.strv == ".") {
                // TODO
                LOG("member");
                fpop();
                auto id = nexts();
                expr = new AST::MemberExpression(expr, new AST::Identifier(id));
            } else if(op.strv == "[") {
                fpop();
                expr = new AST::MemberExpression(expr, parse_expression());
                static_cast<AST::MemberExpression*>(expr)->is_expr = true;
                nextop("]");
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
}

AST::AST *ScriptParser::parse_unary() {
    fsave();
    const auto token = next();
    AST::UnaryExpression::OpType ot = AST::UnaryExpression::OpType::NONE;
    if(token.type == Token::Type::OPERATOR) {
        LOG("UNARY");
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
    fpop();
    return new AST::UnaryExpression(ot, parse_call());

non_unary:
    fload();
    return parse_call();
}

AST::AST *ScriptParser::parse_expression() {
    LOG("expr");
    return parse_conditional_expr();
}

AST::AST *ScriptParser::parse_conditional_expr() {
    auto expr = parse_assignment();
    fsave();
    const auto token = next();
    if(token.strv == "?") {
        fpop();
        auto condition = expr;
        auto expression = parse_assignment();
        nextop(":");
        auto alt = parse_assignment();
        return new AST::ConditionalExpression(condition, expression, alt);
    }
    fload();
    return expr;
}

AST::AST *ScriptParser::parse_assignment() {
    fsave();
    const auto token = next();
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
        const auto op = next();
        AST::BinaryExpression::OpType opt;
        if((opt = optype(op.strv)) != AST::BinaryExpression::OpType::NONE) {
            LOG("assignment");
            fpop();
            auto right = parse_expression();
            return new AST::BinaryExpression(left, right, opt);
        } else delete left;
    }

    fload();
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
                    throw ParserError("Expected , or ) operator");
            } else
                throw ParserError("Expected operator");
        }
        return arguments;
    } else
        throw ParserError("Expected function arguments");
}

AST::AST *ScriptParser::parse_record(bool is_expr) {
    LOG("struct");
    std::string id;
    if(!is_expr) {
        fsave();
        const auto token = next();
        if(token.type != Token::Type::STRING) {
            return nullptr;
        }
        fpop();
        id = token.strv;
    }
    auto stmt = new AST::StructStatement(id);
    stmt->is_expr = is_expr;
    fsave();
    const auto token = next();
    if(token.type != Token::NEWLINE) {
        fload();
        delete stmt;
        return nullptr;
    } else fpop();
    while(!f.eof()) {
        fsave();
        auto token = next();
        LOG(token.strv);
        if(token.type == Token::Type::STRING) {
            if(token.strv == "end") {
                fpop();
                break;
            } else if(token.strv == "function") {
                fpop();
                auto fstmt = parse_function(RECORD);
                stmt->statements.emplace_back(fstmt);
                if(static_cast<AST::FunctionStatement*>(fstmt)->statement->type() != AST::Type::BLOCK_STMT)
                    nextnl();
            } else {
                fload();
                auto left = parse_call();
                if (left->type() != AST::Type::MEMBER_EXPR &&
                    left->type() != AST::Type::IDENTIFIER)
                    throw ParserError("expected member expression or identifier");
                nextop("=");
                auto right = parse_expression();
                stmt->statements.emplace_back(new AST::ExpressionStatement(
                    new AST::BinaryExpression(left, right, AST::BinaryExpression::OpType::SET)
                ));
                nextnl();
            }
            continue;
        }
        throw ParserError("expected end or struct statement");
    }
    return stmt;
}

AST::AST *ScriptParser::parse_function(const ScriptParser::fn_parse_type type) {
    fsave();
    auto token = next();
    AST::FunctionStatement *stmt = nullptr;
    if(type == EXPR) {
        if(token.type == Token::Type::OPERATOR && token.strv == "(") {
            fload();
            stmt = new AST::FunctionStatement("");
        } else throw ParserError("expected function arguments");
    } else {
        if(token.type == Token::Type::STRING) {
            fpop();
            stmt = new AST::FunctionStatement(token.strv);
        } else {
            throw ParserError("expected function id");
        }
    }
    stmt->arguments = parse_function_arguments();
    AST::AST *fstmt = nullptr;
    if(type == EXPR) {
        fsave();
        token = next();
        if(token.strv == "begin") {
            LOG("begin?\n");
            // this is here because blocks in function expressions behave like
            // expressions than statements: we don't require that it ends with a newline
            fpop();
            nextnl();
            auto blk = new AST::BlockStatement();
            while(!f.eof()) {
                fsave();
                const auto token = next();
                if(token.type == Token::Type::STRING && token.strv == "end") {
                    fpop();
                    break;
                } else {
                    fload();
                    auto stmt = parse_statement();
                    if(stmt != nullptr)
                        blk->statements.emplace_back(stmt);
                }
            }
            fstmt = blk;
        } else {
            fload();
            fstmt = parse_statement();
        }
    } else {
        fstmt = parse_statement();
    }
    if(fstmt != nullptr)
        stmt->statement = std::unique_ptr<AST::AST>(fstmt);
    else
        throw ParserError("expected function statement");
    if(type == EXPR || type == RECORD)
        stmt->record_fn = true;
    return stmt;
}

AST::AST *ScriptParser::parse_statement() {
    fsave();
    const auto token = next();
    if(token.type == Token::Type::NONE) {
        ended = true;
        return nullptr;
    } else if(token.type == Token::Type::NEWLINE) {
        fpop();
        return nullptr;
    } else if(token.type == Token::Type::STRING) {
        if(token.strv == "end") {
            throw ParserError("end is not in block statement");
            return nullptr;
        } else if(token.strv == "return") {
            fpop();
            fsave();
            const auto token = next();
            if(token.type == Token::Type::NEWLINE) {
                fpop();
                return new AST::ReturnStatement();
            } else {
                fload();
                auto expr = new AST::ReturnStatement(parse_expression());
                nextnl();
                return expr;
            }
        } else if(token.strv == "function") {
            fpop();
            return parse_function(STATEMENT);
        } else if(token.strv == "record") {
            //fpop();
            auto record = parse_record();
            if(record == nullptr) {
                fload();
                goto expression_stmt;
            } else
                return record;
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
            auto stmt = parse_statement();
            return new AST::WhileStatement(expr, stmt);
        } else if(token.strv == "for") {
            fpop();

            auto id = nexts();
            nextop("=");
            auto from = parse_expression();
            auto dir = nexts();
            if(dir != "to" && dir != "downto")
                throw ParserError("Expected to/downto");
            auto to = parse_expression();

            fsave();
            auto token = next();
            const int stepN = dir == "to" ? 1 : -1;
            if(token.type == Token::Type::STRING && token.strv == "step") {
                fpop();
                return new AST::ForStatement(id, from, to, parse_expression(), stepN, parse_statement());
            } else {
                fload();
                return new AST::ForStatement(id, from, to, stepN, parse_statement());
            }

        } else if(token.strv == "begin") {
            fpop();
            nextnl();
            auto blk = new AST::BlockStatement();
            while(!f.eof()) {
                fsave();
                const auto token = next();
                if(token.type == Token::Type::STRING && token.strv == "end") {
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
