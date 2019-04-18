#include "compiler.h"
#include "error.h"

void Hana::Compiler::set_local(const std::string &id) {
    locals.emplace_back((Local){
        .id = id,
        .scope = nscope,
        .slot = locals.size()
    });
    slotsize++;
}
Hana::Compiler::Local *Hana::Compiler::get_local(const std::string &id) {
    for(auto it = locals.rbegin(); it != locals.rend(); it++) {
        if((*it).id == id) {
            return &*it;
        }
    }
    return nullptr;
}

void Hana::Compiler::scope() {
    LOG("scope");
    nscope++;
}
void Hana::Compiler::unscope() {
    Local local;
    while(!locals.empty() && (local = locals.back()).scope == nscope) {
        locals.pop_back();
        slotsize--;
    }
    nscope--;
}
