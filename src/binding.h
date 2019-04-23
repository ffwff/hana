#pragma once

struct vm;
namespace Hana {

namespace AST {
struct AST;
}

}
Hana::AST::AST *hanayo_parse(const char *str);
void hanayo_ast_emit(Hana::AST::AST *, struct vm*);
void hanayo_free_ast(Hana::AST::AST *);
