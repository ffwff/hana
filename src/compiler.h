#pragma once
#include <string>
#include <vector>
#include <cstddef>
#include <memory>

namespace Hana {

namespace AST { struct AST; struct FunctionStatement; };

extern std::vector<std::string> Files;

class Compiler {

public:
    // scoping
    struct Scope {
        std::vector<std::string> ids;
    };
    std::vector<Scope> scopes;
    void set_local(const std::string &id);
    struct Identifier {
        uint32_t idx; size_t relascope;
        Identifier() : relascope((size_t)-1) {}
        Identifier(uint32_t idx, size_t relascope) : idx(idx), relascope(relascope) {}
    };
    const Identifier get_local(const std::string &id) const;
    void scope();
    uint16_t unscope();

    // loops
    struct Loop {
        std::vector<uint32_t> fill_continue, fill_break;
    };
    std::vector<Loop> loop_stmts;

    // source mapping
    struct SourceMap {
        size_t start_byte, end_byte;
        size_t start_line, end_line;
        size_t fileno;
        // TODO add file info
        SourceMap(size_t start_byte, size_t start_line, size_t end_line)
        : start_byte(start_byte), start_line(start_line), end_line(end_line) {
            fileno = Hana::Files.size()-1;
        };
    };
    std::vector<std::unique_ptr<SourceMap>> src_maps;
    SourceMap find_src_map(size_t bytecode_idx);


};

}
