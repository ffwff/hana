#include "ast.h"
#include "error.h"
#include <array>

using namespace Hana;

static void pindent(int levels) {
    while(levels--)
        std::cout << " ";
}

// Literals
void AST::StrLiteral::print(int indent) {
    pindent(indent);
    std::cout << "\"" << str << "\"\n";
}
void AST::IntLiteral::print(int indent) {
    pindent(indent);
    std::cout << i << '\n';
}
void AST::FloatLiteral::print(int indent) {
    pindent(indent);
    std::cout << f << '\n';
}
void AST::Identifier::print(int indent) {
    pindent(indent);
    std::cout << id << '\n';
}

//
void AST::UnaryExpression::print(int indent) {
    pindent(indent);
    std::cout << "unaryexpr\n";
    body->print(indent+1);
}
void AST::MemberExpression::print(int indent) {
    pindent(indent);
    std::cout << "memexpr\n";
    left->print(indent+1);
    right->print(indent+1);
}
void AST::CallExpression::print(int indent) {
    pindent(indent);
    std::cout << "callexpr\n";
    callee->print(indent+1);
    for(auto &arg : arguments)
        arg->print(indent+1);
}
void AST::BinaryExpression::print(int indent) {
    pindent(indent);
    std::cout << "binexpr\n";
    left->print(indent+1);
    pindent(indent+1);
    if(op == ADD) std::cout << "+";
    else if(op == SUB) std::cout << "-";
    else if(op == MUL) std::cout << "*";
    else if(op == DIV) std::cout << "/";
    else if(op == MOD) std::cout << "mod";
    else if(op == AND) std::cout << "and";
    else if(op == OR) std::cout << "or";
    else if(op == EQ) std::cout << "==";
    else if(op == NEQ) std::cout << "!=";
    else if(op == GT) std::cout << ">";
    else if(op == LT) std::cout << "<";
    else if(op == GEQ) std::cout << ">=";
    else if(op == LEQ) std::cout << "<=";
    else if(op == SET) std::cout << "=";
    else if(op == ADDS) std::cout << "+=";
    else if(op == SUBS) std::cout << "-=";
    else if(op == MULS) std::cout << "*=";
    else if(op == DIVS) std::cout << "/=";
    std::cout << "\n";
    right->print(indent+1);
}

void AST::IfStatement::print(int indent) {
    pindent(indent);
    std::cout << "if\n";
    condition->print(indent+1);
    statement->print(indent+1);
    if(alt) alt->print(indent+1);
}
void AST::WhileStatement::print(int indent) {
    pindent(indent);
    std::cout << "while\n";
    condition->print(indent+1);
    statement->print(indent+1);
}
void AST::ForStatement::print(int indent) {
    pindent(indent);
    std::cout << "for\n";
    from->print(indent+1);
    to->print(indent+1);
    if(step) step->print(indent+1);
    else {
        pindent(indent+1);
        std::cout << stepN << "\n";
    }
    statement->print(indent+1);
}
void AST::FunctionStatement::print(int indent) {
    pindent(indent);
    std::cout << "function " <<id<< "\n";
    for(auto &arg : arguments) {
        pindent(indent);
        std::cout << arg << "\n";
    }
    statement->print(indent+1);
}
void AST::StructStatement::print(int indent) {
    pindent(indent);
    std::cout << "struct\n";
}
void AST::ExpressionStatement::print(int indent) {
    pindent(indent);
    std::cout << "expr stmt\n";
    expression->print(indent+1);
}
void AST::ReturnStatement::print(int indent) {
    pindent(indent);
    std::cout << "return\n";
    if(expression) expression->print(indent+1);
}

void AST::Block::print(int indent) {
    pindent(indent);
    std::cout << "block\n";
    for(auto &s : statements)
        s->print(indent);
}
void AST::BlockStatement::print(int indent) {
    pindent(indent);
    std::cout << "block\n";
    for(auto &s : statements)
        s->print(indent+1);
}
