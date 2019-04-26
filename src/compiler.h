#pragma once
#include <string>
#include <vector>
#include <cstddef>

namespace Hana {

class Compiler {

public:
    // scoping
    struct Local {
        std::string id;
        size_t scope, slot;
    };
    void set_local(const std::string &id);
    const Local *get_local(const std::string &id) const;
    void scope();
    void unscope();
    size_t nscope = 0, slotsize=0; // 0 = global scope
    size_t nslots() const;

    // loops
    struct Loop {
        std::vector<uint32_t> fill_continue, fill_break;
    };
    std::vector<Loop> loop_stmts;

    // exceptions

private:
    std::vector<Local> locals;

};

}
