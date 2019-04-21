#include "binding.h"
#include "ast.h"
#include "compiler.h"
#include "scriptparser.h"

Hana::AST::AST *hanayo_parse(const char *str) {
    Hana::ScriptParser p;
    std::string s(str);
    p.loads(s);
    return p.parse();
}

void hanayo_ast_emit(Hana::AST::AST *ast, struct vm *vm) {
    Hana::Compiler c;
    ast->emit(vm, &c);
}

void hanayo_free_ast(Hana::AST::AST *ast) {
    delete ast;
}
