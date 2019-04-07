#pragma once
#include <string>
#include <memory>
// #include <SDL.h>
#include <vector>
#include <map>
#include <functional>
#include <variant>
#include "error.h"

namespace Hana {

namespace AST {
    class AST;
}
class Environment;

class Value {
public:
    struct Identifier {
        std::string s;
        Identifier(std::string s) : s(s) {};
    };

    // Functions
    struct IFunction {
        std::vector<std::string> arguments;
        bool is_variable = false;
        IFunction(std::vector<std::string> arguments) : arguments(arguments) {}
        IFunction(bool is_variable) : is_variable(true) {}
        virtual void execute(Environment *env)=0;
    };

    struct Function : public IFunction {
    private:
        AST::AST *body;
    public:
        Function(std::vector<std::string> arguments, AST::AST *body) : IFunction(arguments), body(body) {}
        Function(bool _, AST::AST *body) : IFunction(true), body(body) {}
        void execute(Environment *env) override;
    };

    struct NativeFunction : public IFunction {
        std::function<void(Environment*)> fn;
        NativeFunction(std::vector<std::string> arguments, std::function<void(Environment*)> fn) : IFunction(arguments), fn(fn) {};
        NativeFunction(std::function<void(Environment*)> fn) : IFunction(true), fn(fn) {};
        inline void execute(Environment *env) override { return fn(env); }
    };


    // Dictionary
    typedef std::map<std::string, std::size_t> TypeMap;

    struct Dictionary {
        std::map<std::string, Value> data;
        TypeMap types;
        Dictionary() {}
        Dictionary(std::map<std::string, Value> &data) : data(data) {}
        Dictionary(TypeMap types) : types(types) {}
        ~Dictionary() {LOG("DEL", data.size());}
    };
    struct Struct {
        TypeMap types;
//         std::map<std::string, Struct*> structs;
    };

    typedef std::vector<Value> Array;
    typedef Value* Ref;

    typedef std::variant<
        int, float, std::string, Identifier,
        Array, Dictionary, Struct,
        Ref, IFunction*> Variant;

private:
    Variant v;

public:

    template<typename T, std::size_t index = 0>
    static constexpr std::size_t variant_index() {
        if constexpr (index == std::variant_size_v<Variant>) {
            return index;
        } else if constexpr (std::is_same_v<std::variant_alternative_t<index, Variant>, T>) {
            return index;
        } else {
            return variant_index<T, index + 1>();
        }
    }
    const std::size_t index() const { return v.index(); };


    Value() {};
    template<class T> Value(T v) : v(v) {};

    template<class T> inline bool is_type() const { return std::holds_alternative<T>(v); };
    template<class T> inline T get() const { return std::get<T>(v); };
    template<class T> inline T &get() { return std::get<T>(v); };
    Value operator+(const Value &r) const;
    Value operator+() const;
    Value operator-(const Value &r) const;
    Value operator-() const;
    Value operator*(const Value &r) const;
    Value operator/(const Value &r) const;
    Value operator%(const Value &r) const;
    Value operator!() const { return Value(!((bool)this)); };
    operator bool() const;
    std::string to_string() const;

    inline bool operator==(const Value &r) const { return v == r.v; };
    inline bool operator!=(const Value &r) const { return v != r.v; };
    inline bool operator>(const Value &r)  const { return v >  r.v; };
    inline bool operator<(const Value &r)  const { return v <  r.v; };
    inline bool operator>=(const Value &r) const { return v >= r.v; };
    inline bool operator<=(const Value &r) const { return v <= r.v; };

};

#define uncomp(op, type) \
inline bool operator op(const type, const type) { return false; }

#define o(op) \
uncomp(op, Value::Identifier) \
uncomp(op, Value::Dictionary) \
uncomp(op, Value::Struct)
o(==)
o(!=)
o(>)
o(<)
o(>=)
o(<=)
#undef uncomp

}
