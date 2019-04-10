#include <iostream>
#include "src/scriptparser.h"

int main(int argc, char **argv) {
    if(argc != 2) {
        std::cout << "expects at least 2!";
        return 1;
    }
    Hana::ScriptParser p;
    p.loadf(argv[1]);
    auto ast = p.parse();
    std::cout << ast << '\n';
    ast->print();
}
