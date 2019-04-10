#pragma once
#include <map>
#include <memory>
#include <vector>

namespace Hana {

namespace AST {

enum Type {

    NONE,
    CONSTANT, IDENTIFIER, MEMBER_EXPR, CALL_EXPR,
    BINARY_EXPR, UNARY_EXPR,
    ASSIGNMENT_STMT, IF_STMT, WHILE_STMT, BLOCK_STMT,
    FUNCTION_STMT, STRUCT_STMT, EXPR_STMT, RETURN_STMT,
    FOR_STMT,
    BLOCK,

};

#define TYPE(x) \
inline const Type type() override { return x; }

struct AST {
    virtual ~AST() {};
    virtual const Type type() { return NONE; }
    virtual void print(int indent=0) {}
};

// Constant
struct StrLiteral : AST {
    TYPE(CONSTANT)
    std::string str;
    StrLiteral(std::string str) : str(str) {};
    void print(int indent=0) override;
};
struct IntLiteral : AST {
    TYPE(CONSTANT)
    int i;
    IntLiteral(int i) : i(i) {};
    void print(int indent=0) override;
};
struct FloatLiteral : AST {
    TYPE(CONSTANT)
    float f;
    FloatLiteral(float f) : f(f) {};
    void print(int indent=0) override;
};

struct Identifier : AST {
    TYPE(IDENTIFIER)
    std::string id;
    Identifier(std::string id) : id(id) {};
    void print(int indent=0) override;
};

// Expressions
struct Expression : AST {};

struct UnaryExpression : Expression {
    TYPE(UNARY_EXPR)
    enum OpType {
        NONE,
        NEG, POS, NOT
    } op;
    std::unique_ptr<AST> body;
    UnaryExpression(OpType op, AST *body) : op(op), body(body) {};
    void print(int indent=0) override;
};

struct MemberExpression : Expression {
    TYPE(MEMBER_EXPR)
    std::unique_ptr<AST> left, right;
    MemberExpression(AST *left, AST *right) : left(left), right(right) {};
    void print(int indent=0) override;
};

struct CallExpression : Expression {
    TYPE(CALL_EXPR)
    std::unique_ptr<AST> callee;
    std::vector<std::unique_ptr<AST>> arguments;
    CallExpression(AST *callee) : callee(callee) {}
    void print(int indent=0) override;
};

struct BinaryExpression : Expression {
    TYPE(BINARY_EXPR)
    std::unique_ptr<AST> left, right;
    enum OpType {
        NONE,
        ADD, SUB, MUL, DIV, MOD,
        AND, OR,
        EQ, NEQ, GT, LT, GEQ, LEQ,
        SET, ADDS, SUBS, MULS, DIVS,
    } op;
    BinaryExpression() : op(NONE) {};
    BinaryExpression(AST *left, AST *right, OpType op) : left(left), right(right), op(op) {};
    void print(int indent=0) override;
};

// Statements
struct Statement : AST {};

struct IfStatement : Statement {
    TYPE(IF_STMT)
    std::unique_ptr<AST> condition;
    std::unique_ptr<AST> statement, alt;
    IfStatement(AST *condition, AST *statement) : condition(condition), statement(statement)
    {};
    void print(int indent=0) override;
};

struct WhileStatement : Statement {
    TYPE(WHILE_STMT)
    std::unique_ptr<AST> condition;
    std::unique_ptr<AST> statement;
    WhileStatement(AST *condition, AST *statement) : condition(condition), statement(statement)
    {};
    void print(int indent=0) override;
};

struct ForStatement : Statement {
    TYPE(FOR_STMT)
    std::string id;
    std::unique_ptr<AST> from, to, step;
    int stepN;
    std::unique_ptr<AST> statement;
    ForStatement(const std::string &id, AST *from, AST *to, AST *step, AST *statement) : id(id), from(from), to(to), step(step), statement(statement) {}
    ForStatement(const std::string &id, AST *from, AST *to, const int stepN, AST *statement) : id(id), from(from), to(to), stepN(stepN), statement(statement) {}
    void print(int indent=0) override;
};

struct FunctionStatement : Statement {
    TYPE(FUNCTION_STMT)
    std::string id;
    std::unique_ptr<AST> statement;
    std::vector<std::string> arguments;
    FunctionStatement(std::string &id) : id(id) {}
    void print(int indent=0) override;
};

struct StructStatement : Statement {
    TYPE(STRUCT_STMT)
    std::map<std::string,std::string> dict; // name : type
    std::string id;
    StructStatement(std::string &id) : id(id) {}
    void print(int indent=0) override;
};

struct ExpressionStatement : Statement {
    TYPE(EXPR_STMT)
    std::unique_ptr<AST> expression;
    ExpressionStatement(AST *expression) : expression(expression) {}
    void print(int indent=0) override;
};

struct ReturnStatement : Statement {
    TYPE(RETURN_STMT)
    std::unique_ptr<AST> expression;
    ReturnStatement() {}
    ReturnStatement(AST *expression) : expression(expression) {}
    void print(int indent=0) override;
};

// Block
struct Block : AST {
    TYPE(BLOCK)
    std::vector<std::unique_ptr<AST>> statements;
    void print(int indent=0) override;
};
struct BlockStatement : Statement {
    TYPE(BLOCK_STMT)
    std::vector<std::unique_ptr<AST>> statements;
    BlockStatement() {};
    void print(int indent=0) override;
};

#undef TYPE

}

}
