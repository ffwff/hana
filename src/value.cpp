#include "value.h"
#include "error.h"
#include "ast.h"
#include <vector>
#include <string>

using namespace Hana;

// Function
void Value::Function::execute(Environment *env) {
    if(is_variable) {
        // TODO variadic
    }
    try {
        body->evaluate(env);
    } catch(const AST::ReturnStatement *ret) {
        if(ret->expression != nullptr)
            ret->expression->evaluate(env);
        return;
    }
}


// Arith
#define arith_op(op, custom) \
Value Value::operator op(const Value &r) const { \
    if(std::holds_alternative<int>(v)) { \
        if(std::holds_alternative<int>(r.v)) { \
            return Value(get<int>() op r.get<int>()); \
        } else if(std::holds_alternative<float>(r.v)) { \
            return Value((float)(get<int>()) op r.get<int>()); \
        } else \
            goto error; \
    } else if (std::holds_alternative<float>(v)) { \
        if(std::holds_alternative<float>(r.v)) { \
            return Value(get<float>() op (float)(r.get<int>())); \
        } else if(std::holds_alternative<float>(r.v)) { \
            return Value(get<float>() op r.get<float>()); \
        } else \
            goto error; \
    } custom \
    error: \
    FATAL("Value error", "Cannot add 2 values together!"); \
}

arith_op(+,
         else if(std::holds_alternative<std::string>(v) &&
                 std::holds_alternative<std::string>(r.v)) {
             return Value(get<std::string>() + r.get<std::string>());
         }
)
arith_op(-,)
arith_op(*,)
arith_op(/,)
#undef arith_op

Value::operator bool() const {
    if(std::holds_alternative<int>(v)) return get<int>();
    else if(std::holds_alternative<float>(v)) return get<float>();
    else if(std::holds_alternative<std::string>(v)) return !get<std::string>().empty();
    else if(std::holds_alternative<Array>(v)) return !get<Array>().empty();
    else if(std::holds_alternative<Dictionary>(v)) return true;
    else if(std::holds_alternative<Struct>(v)) return true;
    else if(std::holds_alternative<IFunction*>(v)) return true;
    return false;
}


// unary op
Value Value::operator-() const {
    if(std::holds_alternative<int>(v)) return -get<int>();
    else if(std::holds_alternative<float>(v)) return -get<float>();
    FATAL("Value error", "Cannot negate value of wrong type!");
}

Value Value::operator+() const {
    if(std::holds_alternative<int>(v)) return +get<int>();
    else if(std::holds_alternative<float>(v)) return +get<float>();
    FATAL("Value error", "Cannot negate value of wrong type!");
}

// string
std::string Value::to_string() const {
    if(std::holds_alternative<int>(v)) return std::to_string(get<int>());
    else if(std::holds_alternative<float>(v)) return std::to_string(get<float>());
    else if(std::holds_alternative<std::string>(v)) return get<std::string>();
    else if(std::holds_alternative<Array>(v)) return "[array]";
    else if(std::holds_alternative<Dictionary>(v)) return "[dictionary]";
    else if(std::holds_alternative<Struct>(v)) return "[struct]";
    else if(std::holds_alternative<IFunction*>(v)) return "[function]";
    return "nil";

}
