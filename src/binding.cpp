#include "binding.h"
#include "ast.h"
#include "compiler.h"
#include "scriptparser.h"

struct hana_ast *hana_parse(const char *str) {
    Hana::ScriptParser p;
    std::string s(str);
    p.loads(s);
    return (hana_ast*)p.parse();
}

void hana_ast_emit(struct hana_ast *ast, struct vm *vm) {
    Hana::Compiler c;
    static_cast<Hana::AST::AST *>((void*)ast)->emit(vm, &c);
}

void hana_free_ast(struct hana_ast *ast) {
    delete static_cast<Hana::AST::AST *>((void*)ast);
}
