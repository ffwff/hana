#pragma once
#include <string>
#include <vector>
#include <cstddef>

namespace Hana {

class Compiler {

public:
    struct Local {
        std::string id;
        size_t scope, slot;
    };
    struct Loop {
        std::vector<uint32_t> fill_continue, fill_break;
    };
    void set_local(const std::string &id);
    Local *get_local(const std::string &id);
    void scope();
    void unscope();
    size_t nscope = 0, slotsize=0; // 0 = global scope
    size_t nslots();
    std::vector<Loop> loop_stmts;

private:
    std::vector<Local> locals;

};

}
