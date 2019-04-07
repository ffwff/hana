#pragma once
#include <unordered_map>
#include <string>
#include <stack>
#include <cassert>
#include "value.h"

namespace Hana {

namespace AST {
    class AST;
}

class Environment {

private:
    std::unordered_map<std::string, Value> variables;
    Environment *parent = nullptr;
public:
    Environment() {};
    Environment(Environment *parent) : parent(parent) {};
    void set_parent(Environment *parent) { this->parent = parent; };

    Value &get_var(const std::string &key);
    void set_var(const std::string &key, const Value &var);
    void set_local_var(const std::string &key, const Value &var);
    void del_var(const std::string &key);
    void clear();

    Environment *inherit();

    std::vector<Value> stack;
    inline const Value pop() {
        assert(stack.size() > 0);
        auto val = stack.back();
        stack.pop_back();
        return val;
    }

    void execute(AST::AST *);
    void execute_function(const std::string &key, const std::vector<Value> &arguments);
};

}
