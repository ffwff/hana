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

struct hana_ast *hana_parse_file(const char *str) {
    Hana::ScriptParser p;
    std::string s(str);
    Hana::Files.emplace_back(s);
    //printf("%s\n", Hana::Files.at(0).data());
    p.loadf(s);
    auto ptr = (hana_ast*)p.parse();
    if(ptr == nullptr) Hana::Files.pop_back();
    return ptr;
}

extern Hana::Compiler compiler;
void hana_ast_emit(struct hana_ast *ast, struct vm *vm) {
    static_cast<Hana::AST::AST *>((void*)ast)->emit(vm, &compiler);
}

void hana_free_ast(struct hana_ast *ast) {
    delete static_cast<Hana::AST::AST *>((void*)ast);
}
