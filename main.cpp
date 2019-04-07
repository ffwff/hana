#include "src/scriptparser.h"
#include <iostream>

int main(int argc, char **argv) {
    if(argc != 2) {
        std::cout << "expects at least 2!";
        return 1;
    }
    Hana::ScriptParser p;
    p.loadf(argv[1]);
    auto ast = p.parse();
    Hana::Environment e;
    e.set_var("record", new Hana::Value::NativeFunction([](Hana::Environment *e){
        e->stack.emplace_back(Hana::Value::Dictionary());
    }));
    e.set_var("print", new Hana::Value::NativeFunction([](Hana::Environment *e){
        auto size = e->stack.size();
        for(size_t i = 0; i < size; i++) std::cout << e->stack[i].to_string();
        std::cout << '\n';
    }));
    e.execute(ast);
}
