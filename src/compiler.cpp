#include "compiler.h"
#include "error.h"
#include <algorithm>

// scoping
void Hana::Compiler::set_local(const std::string &id) {
    if(scopes.empty()) return;
    auto &scope = scopes.back();
    scope.ids.push_back(id);
}
const Hana::Compiler::Identifier Hana::Compiler::get_local(const std::string &id) const {
    size_t gscope = 0;
    for(auto it = scopes.rbegin();
        it != scopes.rend(); it++, gscope++) {
        LOG(id, " up ",gscope);
        auto scope = *it;
        auto iit = std::find(scope.ids.begin(), scope.ids.end(), id);
        if(iit != scope.ids.end()) {
            auto idx = std::distance(scope.ids.begin(), iit);
            return Identifier(idx, gscope);
        }
    }
    LOG("can't get ", id);
    return Identifier();
}
void Hana::Compiler::scope() {
    LOG("new scope");
    scopes.emplace_back();
}
uint16_t Hana::Compiler::unscope() {
    LOG("descope");
    auto sz = scopes.back().ids.size();
    scopes.pop_back();
    return sz;
}

// source map
Hana::Compiler::SourceMap Hana::Compiler::find_src_map(size_t bytecode_idx) {
    for(auto it = src_maps.rbegin(); it != src_maps.rend(); it++) {
        Hana::Compiler::SourceMap src_map = **it;
        if(bytecode_idx >= src_map.start_byte && bytecode_idx <= src_map.end_byte)
            return src_map;
    }
    return Hana::Compiler::SourceMap(0, -1, -1);
}
