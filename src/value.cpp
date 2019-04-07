#include "value.h"
#include "error.h"
#include "ast.h"
#include <vector>
#include <string>

using namespace Hana;

// Function
void Value::Function::execute(Environment *env) {
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
    if(is_type<int>()) { \
        if(r.is_type<int>()) { \
            return Value(get<int>() op r.get<int>()); \
        } else if(r.is_type<float>()) { \
            return Value((float)(get<int>()) op r.get<int>()); \
        } else \
            goto error; \
    } else if (is_type<float>()) { \
        if(r.is_type<float>()) { \
            return Value(get<float>() op (float)(r.get<int>())); \
        } else if(r.is_type<float>()) { \
            return Value(get<float>() op r.get<float>()); \
        } else \
            goto error; \
    } custom \
    error: \
    FATAL("Value error", "Cannot add 2 values together!"); \
}

arith_op(+,
         else if(is_type<std::string>() && r.is_type<std::string>()) {
             return Value(get<std::string>() + r.get<std::string>());
         }
)
arith_op(-,)
arith_op(*,)
arith_op(/,)
Value Value::operator %(const Value &r) const {
    if(is_type<int>()) {
        return Value(get<int>() % r.get<int>());
    } else {
        FATAL("Value error", "Cannot add 2 values together!");
    }
}
#undef arith_op

Value::operator bool() const {
    if(is_type<int>()) return get<int>();
    else if(is_type<float>()) return get<float>();
    else if(is_type<std::string>()) return !get<std::string>().empty();
    else if(is_type<Array>()) return !get<Array>().empty();
    else if(is_type<Dictionary>()) return true;
    else if(is_type<Struct>()) return true;
    else if(is_type<IFunction*>()) return true;
    return false;
}


// unary op
Value Value::operator-() const {
    if(is_type<int>()) return -get<int>();
    else if(is_type<float>()) return -get<float>();
    FATAL("Value error", "Cannot negate value of wrong type!");
}

Value Value::operator+() const {
    if(is_type<int>()) return +get<int>();
    else if(is_type<float>()) return +get<float>();
    FATAL("Value error", "Cannot negate value of wrong type!");
}

// string
std::string Value::to_string() const {
    if(is_type<int>()) return std::to_string(get<int>());
    else if(is_type<float>()) return std::to_string(get<float>());
    else if(is_type<std::string>()) return get<std::string>();
    else if(is_type<Array>()) return "[array]";
    else if(is_type<Dictionary>()) return "[dictionary]";
    else if(is_type<Struct>()) return "[struct]";
    else if(is_type<IFunction*>()) return "[function]";
    return "nil";

}
