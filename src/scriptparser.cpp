#include "scriptparser.h"
#include <utility>
#include <functional>
#include <memory>

using namespace Hana;

// macros to record line information
#define START_GMR const auto _START_GMR = lines+1;
#define END_GMR const auto _END_GMR = lines+1;
static AST::AST *_wrap_line(AST::AST *ast, size_t start_line, size_t end_line) {
    ast->start_line = start_line;
    ast->end_line = end_line;
    return ast;
}
#define WRAP_LINE(ast) _wrap_line(ast, _START_GMR, lines+1)
static size_t _start_lines = 0;
static AST::AST *_wrap_block(ScriptParser *p, AST::AST *ast) {
    ast->start_line = _start_lines;
    ast->end_line = p->lines+1;
    return ast;
}
#define WRAP_RET2(ast) (_start_lines = _START_GMR, _wrap_block(this, ast))
#ifndef RELEASE
#define S1(x) #x
#define S2(x) S1(x)
#define fsave() (this->fsave_(std::string(__FUNCTION__) + ":" + std::string(S2(__LINE__))))
#define fpop() this->fpop_()
#else
#define dump_fposs()
#endif

// tokeniser
Hana::Parser::Token Hana::ScriptParser::next() {
    char c;
    const auto skip_un = [&]() {
        while((c = peek()) != EOF) {
            if(c == ' ' || c == '\t' || c == '\r') {
                c = getc();
            } else if (c == '/') {
                const auto pos = tellg();
                getc();
                char c = getc();
                if(c == '/') {
                    while((c = peek()) != EOF && c != '\n')
                        getc();
                } else if(c == '*') {
                    while((c = peek()) != EOF) {
                        char c = getc();
                        if(c == '\n') lines++;
                        const char c1 = getc();
                        if(c == '*' && c1 == '/')
                            break;
                        else unget();
                    }
                } else {
                    seekg(pos);
                    return; // this is an operator, so it should be handled below
                }
            } else break;
        }
    };
    skip_un();
    std::string token;
    if(c == EOF) {
        getc();
        return Token();
    } else if(c == '\'' || c == '"') {
        const char start = c;
        getc();
        std::string str;
        while((c = peek()) != EOF) {
            if(c == start) {
                getc();
                break;
            } else if(c == '\\') {
                getc();
                const char esc = getc();
                if(esc == 'n') str += '\n';
                else if(esc == 'r') str += '\r';
                else if(esc == 't') str += '\t';
                else str += esc;
            } else {
                auto c = getc();
                if(c == '\n') lines++;
                str += c;
            }
        }
        return Token(str, Token::Type::STRLITERAL);
    } else if(c == '\n') {
        getc();
        lines++;
        while((c = peek()) != EOF) {
            skip_un();
            if(c == '\n') {
                getc();
                lines++;
            } else break;
        }
        return Token(true);
    } else if(isdigit(c)) {
        while((c = peek()) != EOF && isdigit(c))
            token += getc();
        if(peek() == '.') {
            token += getc();
            while((c = peek()) != EOF && isdigit(c))
                token += getc();
            return Token(strtod(token.data(), nullptr));
        }
        return Token(std::stoi(token));
    }
#define is_onech_op(c) \
    (c == '=' || c == '<' || c == '>' || c == '(' || c == ')' || \
    c == '+' || c == '-' || c == '/' || c == '*' || c == '[' || c == ']' || \
    c == '.' || c == ',' || c == '?' || c == '!' || c == ':')
    else if(is_onech_op(c)) {
        // "=="|"!="|">"|"<"|">="|"<="|"+"|"-"|"/"|"*"
        c = getc();
        const auto prev = tellg();
        const char nc = getc();
#define twoch(str) \
        if(c == str[0] && nc == str[1]) return Token(str, Token::Type::OPERATOR);

        twoch("==")
        else twoch("!=")
        else twoch(">=")
        else twoch("<=")
        else twoch("+=")
        else twoch("-=")
        else twoch("*=")
        else twoch("/=")
        else twoch("%=")
        else twoch("::")
        else {
            seekg(prev);
            token = c;
            return Token(token, Token::Type::OPERATOR);
        }

    } else {
        token += getc();
        while((c = peek()) != EOF && !isspace(c) && (!is_onech_op(c) || c == '?' || c == '!'))
            token += getc();
        return Token(token);
    }
}

// Expressions
AST::AST *ScriptParser::parse_factor() {
    const auto token = next();
    if(token.type == Token::Type::OPERATOR) {
        if(token.strv == "(") {
            auto expr = parse_expression();
            nextop(")");
            return expr;
        } else if(token.strv == "[") {
            auto expr = new AST::Array();
            fsave();
            auto op = next();
            if(op.strv == "]") {
                fpop();
                return expr;
            }
            fload();
            expr->values.emplace_back(parse_expression());
            do {
                op = next();
                if(op.strv == ",") {
                    expr->values.emplace_back(parse_expression());
                } else if(op.strv == "]") {
                    break;
                }
            } while(!eof());
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
        return new AST::Identifier(token.strv);
    } else if(token.type == Token::Type::STRLITERAL) {
        return new AST::StrLiteral(token.strv);
    } else if(token.type == Token::Type::INTEGER) {
        return new AST::IntLiteral(token.intv);
    } else if(token.type == Token::Type::FLOAT) {
        return new AST::FloatLiteral(token.floatv);
    }
    throw ParserError("Expected factor token: ", lines); // TODO
}

AST::AST *ScriptParser::parse_call() {
    auto factor = parse_factor();

    fsave();
    auto op = next();
    if(op.type == Token::Type::OPERATOR &&
        (op.strv == "(" || op.strv == "." || op.strv == "::" || op.strv == "[")) {
        AST::AST *expr = factor;
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
                    auto callexpr = static_cast<AST::CallExpression*>(expr);
                    callexpr->arguments.emplace_back(parse_expression());
                    if(callexpr->arguments.size() > 65535)
                        throw ParserError("Only 65535 arguments are allowed");
                }

                while(!eof()) {
                    op = next();
                    if(op.type == Token::Type::OPERATOR) {
                        if(op.strv == ",") {
                            static_cast<AST::CallExpression*>(expr)->arguments.emplace_back(parse_expression());
                        } else if(op.strv == ")") {
                            LOG("break", tellg());
                            goto next;
                        } else
                            throw ParserError("Expected , or ) operator");
                    } else
                        throw ParserError("Expected operator");
                }
            } else if(op.strv == ".") {
                fpop();
                auto id = nexts();
                expr = new AST::MemberExpression(expr, new AST::Identifier(id));
            } else if(op.strv == "::") {
                fpop();
                auto id = nexts();
                expr = new AST::MemberExpression(expr, new AST::Identifier(id));
                static_cast<AST::MemberExpression*>(expr)->is_namespace = true;
            } else if(op.strv == "[") {
                fpop();
                expr = new AST::MemberExpression(expr, parse_expression());
                static_cast<AST::MemberExpression*>(expr)->is_expr = true;
                nextop("]");
            } else {
                fload();
                return expr;
            }

        next:
            fsave();
            op = next();

        } while(!eof());

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
        auto left = parse_call();
        const auto op = next();
        AST::BinaryExpression::OpType opt;
        if((opt = optype(op.strv)) != AST::BinaryExpression::OpType::NONE) {
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
        AST::AST *left = down(); \
        fsave(); \
        Token token = next(); \
        AST::BinaryExpression::OpType ot; \
        if ((ot = optype(token.strv)) != AST::BinaryExpression::OpType::NONE) { \
            fpop(); \
            AST::BinaryExpression *binexpr = new AST::BinaryExpression(); \
            binexpr->left = std::unique_ptr<AST::AST>(left); \
            binexpr->op = ot; \
            binexpr->right = std::unique_ptr<AST::AST>(down()); \
            while(!eof()) { \
                fsave(); \
                token = next(); \
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
        arguments.push_back(token.strv);
        while(!eof()) {
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

// Statements
AST::AST *ScriptParser::expect_statement() {
    auto stmt = parse_statement();
    if(stmt == nullptr) throw ParserError("Expected non-empty statement");
    return stmt;
}

AST::AST *ScriptParser::parse_record(bool is_expr) {
    START_GMR
    std::string id;
    if(!is_expr) {
        const auto token = next();
        if(token.type != Token::Type::STRING) {
            throw ParserError("Expected record identifier");
        }
        id = token.strv;
    }
    LOG("is expr");
    auto stmt = new AST::StructStatement(id);
    stmt->start_line = _START_GMR;
    stmt->is_expr = is_expr;
    fsave();
    const auto token = next();
    if(token.type != Token::NEWLINE) {
        fload();
        delete stmt;
        return nullptr;
    } else fpop();
    while(!eof()) {
        fsave();
        auto token = next();
        if(token.type == Token::Type::STRING) {
            if(token.strv == "end") {
                stmt->end_line = lines+1;
                fpop();
                break;
            } else if(token.strv == "function") {
                fpop();
                START_GMR
                auto fstmt = parse_function(RECORD);
                stmt->statements.emplace_back(WRAP_RET2(fstmt));
                if(static_cast<AST::FunctionStatement*>(fstmt)->statement->type() != AST::Type::BLOCK_STMT)
                    nextnl();
            } else if(token.strv == "record") {
                fpop();
                START_GMR
                auto ss = parse_record(false);
                static_cast<AST::StructStatement*>(ss)->is_expr = true;
                nextnl();
                stmt->statements.emplace_back(WRAP_RET2(ss));
            } else {
                fload();
                START_GMR
                auto left = parse_call();
                if (left->type() != AST::Type::MEMBER_EXPR &&
                    left->type() != AST::Type::IDENTIFIER)
                    throw ParserError("expected member expression or identifier");
                nextop("=");
                auto right = parse_expression();
                stmt->statements.emplace_back(WRAP_RET2(new AST::ExpressionStatement(
                    new AST::BinaryExpression(left, right, AST::BinaryExpression::OpType::SET)
                )));
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
    START_GMR
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
            printf("%d\n", token.type);
            throw ParserError("expected function id");
        }
    }

    stmt->arguments = parse_function_arguments();
    AST::AST *fstmt = nullptr;
    if(type == EXPR) {
        fsave();
        token = next();
        if(token.strv == "begin") {
            // this is here because blocks in function expressions behave like
            // expressions than statements: we don't require that it ends with a newline
            fpop();
            auto blk = new AST::BlockStatement();
            blk->start_line = _START_GMR;
            nextnl();
            while(!eof()) {
                fsave();
                const auto token = next();
                if(token.type == Token::Type::STRING && token.strv == "end") {
                    fpop();
                    blk->end_line = lines+1;
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
        stmt->is_expr = true;
    stmt->start_line = _START_GMR;
    stmt->end_line = fstmt->end_line;
    return stmt;
}

AST::AST *ScriptParser::parse_statement() {
    fsave();
    const auto token = next();
    if(token.type == Token::Type::NONE) {
        fpop();
        ended = true;
        return nullptr;
    } else if(token.type == Token::Type::NEWLINE) {
        fpop();
        return nullptr;
    } else if(token.type == Token::Type::STRING) {
        if(token.strv == "end")
            throw ParserError("end is not in block statement");
        else if(token.strv == "return") {
            fpop();
            START_GMR
            fsave();
            const auto stmt = WRAP_RET2(new AST::ReturnStatement());
            const auto token = next();
            if(token.type == Token::Type::NEWLINE) {
                fpop();
                return stmt;
            } else {
                fload();
                static_cast<AST::ReturnStatement*>(stmt)->expression = std::unique_ptr<AST::AST>(parse_expression());
                return stmt;
            }
        } else if(token.strv == "function") {
            fpop();
            return parse_function(STATEMENT);
        } else if(token.strv == "record") {
            fpop();
            START_GMR
            auto record = parse_record(false);
            if(record == nullptr) {
                fload();
                goto expression_stmt;
            } else
                return WRAP_LINE(record);
        } else if(token.strv == "if") {
            fpop();
            START_GMR
            auto expr = parse_expression();
            auto stmt = expect_statement();
            auto ifstmt = WRAP_LINE(new AST::IfStatement(expr, stmt));
            fsave();
            auto token = next();
            if(token.type == Token::Type::NEWLINE) {
                token = next();
            }
            if(token.type == Token::Type::STRING && token.strv == "else") {
                fpop();
                auto stmt = parse_statement();
                static_cast<AST::IfStatement*>(ifstmt)->alt = std::unique_ptr<AST::AST>(stmt);
                ifstmt->end_line = stmt->end_line;
            } else {
                fload();
            }
            return ifstmt;
        } else if(token.strv == "while") {
            fpop();

            START_GMR

            auto expr = parse_expression();
            auto stmt = expect_statement();

            auto wstmt = new AST::WhileStatement(expr, stmt);
            wstmt->start_line = _START_GMR;
            wstmt->end_line = stmt->end_line;
            return wstmt;
        } else if(token.strv == "for") {
            fpop();

            START_GMR

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
            AST::ForStatement *fstmt = nullptr;
            if(token.type == Token::Type::STRING && token.strv == "step") {
                fpop();
                fstmt = new AST::ForStatement(id, from, to, parse_expression(), stepN, expect_statement());
            } else {
                fload();
                fstmt = new AST::ForStatement(id, from, to, stepN, expect_statement());
            }
            fstmt->start_line = _START_GMR;
            fstmt->end_line = fstmt->statement->end_line;
            return fstmt;
        } else if(token.strv == "continue") {
            fpop();
            START_GMR
            const auto stmt = WRAP_RET2(new AST::ContinueStatement());
            nextnl();
            return stmt;
        } else if(token.strv == "break") {
            fpop();
            START_GMR
            const auto stmt = WRAP_RET2(new AST::BreakStatement());
            nextnl();
            return stmt;
        } else if(token.strv == "try") {
            fpop();
            START_GMR
            std::vector<std::unique_ptr<AST::AST>> statements;
            std::vector<std::unique_ptr<AST::CaseStatement>> cases;
            AST::CaseStatement *case_stmt = nullptr;
            while(!eof()) {
                fsave();
                auto token = next();
                if(token.strv == "case") {
                    LOG("case");
                    std::string id;
                    fpop();

                    // empty case statement
                    fsave();
                    auto token = next();
                    if(token.type == Token::Type::NEWLINE) {
                        fpop();
                        case_stmt = new AST::CaseStatement(nullptr, "");
                        case_stmt->start_line = lines;
                        cases.emplace_back(case_stmt);
                        continue;
                    }
                    fload();

                    auto etype = parse_expression();
                    token = next();
                    if(token.strv == "as") {
                        id = nexts();
                        nextnl();
                    } else if(token.type != Token::Type::NEWLINE) {
                        throw ParserError("Expected newline or as [id]");
                    }
                    case_stmt = new AST::CaseStatement(etype, id);
                    case_stmt->start_line = lines;
                    cases.emplace_back(case_stmt);
                } else {
                    if(token.strv == "end") {
                        fpop();
                        if(case_stmt == nullptr)
                            throw ParserError("Expected try block to have case statement");
                        case_stmt->end_line = lines+1;
                        nextnl();
                        break;
                    } else if(case_stmt == nullptr) { // still in "try"
                        fload();
                        auto stmt = parse_statement();
                        if(stmt) statements.emplace_back(stmt);
                    } else { // in case statement's block
                        fload();
                        auto stmt = parse_statement();
                        if(stmt) case_stmt->statements.emplace_back(stmt);
                    }
                }
            }
            auto trystmt = new AST::TryStatement();
            trystmt->statements = std::move(statements);
            trystmt->cases = std::move(cases);
            return WRAP_RET2(trystmt);
        } else if(token.strv == "raise") {
            fpop();
            START_GMR
            auto stmt = WRAP_RET2(new AST::RaiseStatement());
            fsave();
            const auto token = next();
            if(token.type == Token::Type::NEWLINE) {
                fpop();
                return stmt;
            } else {
                fload();
                static_cast<AST::RaiseStatement*>(stmt)->expression = std::unique_ptr<AST::AST>(parse_expression());
                return stmt;
            }
        } else if(token.strv == "begin") {
            fpop();
            START_GMR
            nextnl();
            auto blk = new AST::BlockStatement();
            while(!eof()) {
                fsave();
                const auto token = next();
                if(token.type == Token::Type::STRING && token.strv == "end") {
                    fpop();
                    blk = (AST::BlockStatement*)WRAP_RET2(blk);
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
        START_GMR
        auto stmt = new AST::ExpressionStatement(parse_expression());
        stmt->start_line = _START_GMR;
        stmt->end_line = lines+1;
        nextnl();
        return stmt;
    }
}

AST::AST *ScriptParser::parse_block() {
    auto block = new AST::Block();
    while(!ended) {
        auto stmt = parse_statement();
        if(stmt != nullptr) {
            if(stmt->type() == AST::RETURN_STMT)
                throw ParserError("Return statements are only possible inside functions");
            block->statements.emplace_back(stmt);
        }
    }
    if(block->statements.empty()) return nullptr;
    return block;
}

AST::AST *ScriptParser::parse() {
    try {
        return parse_block();
    } catch(ParserError &e) {
        std::cerr << "Parser error: " << e.what() << " at line " << lines+1 << "\n";
    } catch(LexerError &e) {
        std::cerr << "Lexer error: " << e.what() << " at line " << lines+1 << "\n";
    }
    return nullptr;
}
