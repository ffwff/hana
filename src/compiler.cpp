#include "compiler.h"
#include "error.h"

// scoping
void Hana::Compiler::set_local(const std::string &id) {
    locals.emplace_back((Local){
        .id = id,
        .scope = nscope,
        .slot = locals.size()
    });
    assert(locals.size() < 65535);
    slotsize++;
}
const Hana::Compiler::Local *Hana::Compiler::get_local(const std::string &id) const {
    for(auto it = locals.rbegin(); it != locals.rend(); it++) {
        if((*it).id == id) {
            return &*it;
        }
    }
    return nullptr;
}
void Hana::Compiler::scope() {
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
size_t Hana::Compiler::nslots() const {
    return locals.size();
}

// exceptions
