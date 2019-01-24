#include "environment.h"
#include "error.h"
#include "ast.h"

using namespace Hana;

Environment *Environment::inherit() {
    return new Environment(this);
}

Value &Environment::get_var(const std::string &key) {
    // variable
    if(variables.find(key) != variables.end())
        return variables[key];
    if(parent != nullptr)
        return parent->get_var(key);
    FATAL("Environment error", "No such key ", key, " in environment");
}

void Environment::set_var(const std::string &key, const Value &var) {
    if(variables.find(key) != variables.end()) {
        // Overriding existing variable
        variables[key] = var;
        return;
    }
    if(parent != nullptr) {
        // Overriding existing var outside of scope
        parent->set_var(key, var);
        return;
    }
    variables[key] = var;
}

void Environment::set_local_var(const std::string &key, const Value &var) {
    variables[key] = var;
}

void Environment::del_var(const std::string &key) {
    variables.erase(key);
}

void Environment::clear() {
    variables.clear();
}

void Environment::execute(AST::AST *ast) {
    ast->evaluate(this);
}

void Environment::execute_function(const std::string &key, const std::vector<Value> &arguments) {
    std::unique_ptr<Environment> scoped(inherit());
    auto fn = get_var(key).get<Value::IFunction*>();
    for(size_t i = 0; i < arguments.size(); i++) {
        scoped->set_local_var(fn->arguments[i], arguments[i]);
    }
    fn->execute(scoped.get());
}
