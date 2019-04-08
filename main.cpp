#include "src/scriptparser.h"
#include "src/types.h"
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

    // types
    /*typedef std::variant<
     T ype::Value, int, float, *std::string, Identifier,
     Array, Dictionary, Struct,
     Ref, IFunction*> Variant;*/
    e.set_var("int", new Hana::Type([](Hana::Environment *e){
        e->stack.emplace_back(e->pop().to_int());
    }));
    e.set_var("float", new Hana::Type([](Hana::Environment *e){
        e->stack.emplace_back(e->pop().to_float());
    }));
    e.set_var("string", new Hana::Type([](Hana::Environment *e){
        e->stack.emplace_back(e->pop().to_string());
    }));
    e.set_var("array", new Hana::Type([](Hana::Environment *e){
        e->stack.emplace_back(Hana::Value::Array());
    }));
    e.set_var("record", new Hana::Type([](Hana::Environment *e){
        e->stack.emplace_back(Hana::Value::Dictionary());
    }));
    e.set_var("function", new Hana::Type([](Hana::Environment *e){
        e->stack.emplace_back(new Hana::Value::NativeFunction([](Hana::Environment *e){
        }));
    }));

    // stdlib
    e.set_var("print", new Hana::Value::NativeFunction([](Hana::Environment *e){
        auto size = e->stack.size();
        for(size_t i = 0; i < size; i++) std::cout << e->stack[i].to_string();
        std::cout << '\n';
    }));
    e.execute(ast);
}
