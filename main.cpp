#include "src/scriptparser.h"
#include <iostream>

int main() {
    Hana::ScriptParser p;
    p.loads("print(\"Hello World!\")");
    auto ast = p.parse();
    Hana::Environment e;
    e.set_var("print", new Hana::Value::NativeFunction([](Hana::Environment *e){
        std::cout << e->pop().to_string() << "\n";
    }));
    e.execute(ast);
}
