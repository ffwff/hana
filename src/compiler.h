#pragma once
#include <string>
#include <vector>
#include <cstddef>
#include <memory>

namespace Hana {

namespace AST { struct AST; struct FunctionStatement; };

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

    // source mapping
    struct SourceMap {
        size_t start_byte, end_byte;
        size_t start_line, end_line;
        // TODO add file info
        SourceMap() {};
        SourceMap(size_t start_byte, size_t start_line, size_t end_line)
        : start_byte(start_byte), start_line(start_line), end_line(end_line) {};
    };
    std::vector<std::unique_ptr<SourceMap>> src_maps;
    SourceMap find_src_map(size_t bytecode_idx);

    // functions
    struct Function {
        AST::FunctionStatement *fn_ast;
        uint32_t body_start;
    };
    std::vector<Function> functions;

private:
    std::vector<Local> locals;

};

}
