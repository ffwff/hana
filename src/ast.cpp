#include "ast.h"
#include "error.h"
#include <array>

using namespace Hana;

// evaluation
//-----------------------------
// constant
void AST::Constant::evaluate(Environment *env) {
    env->stack.push_back(value);
}

void AST::Identifier::evaluate(Environment *env) {
    // Constants
    if(id == "nil") env->stack.emplace_back();
    else
        env->stack.push_back(env->get_var(id));
}

// expressions
void AST::UnaryExpression::evaluate(Environment *env) {
    body->evaluate(env);
    if(op == NEG) env->stack.emplace_back(-env->pop());
    else if(op == POS) env->stack.emplace_back(+env->pop());
    else if(op == NOT) env->stack.emplace_back(!env->pop());
}

void AST::MemberExpression::evaluate(Environment *env) {
    auto dict = env->get_var(id[0]).get<Value::Dictionary>();
    for(size_t i = 1; i < id.size()-1; i++) {
        try {
            dict = dict.data.at(id[i]).get<Value::Dictionary>();
        } catch(const std::out_of_range&) {
            FATAL("Interpreter error", "Expected ", id[i], " key to exist");
        }
    }
    try {
        env->stack.push_back(dict.data.at(id[id.size()-1]));
    } catch(const std::out_of_range&) {
        FATAL("Interpreter error", "Expected ", id[id.size()-1], " key to exist");
    }
}

void AST::CallExpression::evaluate(Environment *env) {
    auto fn = env->get_var(name).get<Value::IFunction*>();
    std::unique_ptr<Environment> scoped(env->inherit());
    if(fn->is_variable) {
        // pass argument to scoped for further processing
        for(size_t i = 0; i < arguments.size(); i++) {
            arguments[i]->evaluate(env);
            scoped->stack.emplace_back(env->pop());
        }
    } else {
        if(arguments.size() != fn->arguments.size())
            FATAL("Interpreter error", "argument sizes don't match for ", name);
        // set function arguments
        for(size_t i = 0; i < arguments.size(); i++) {
            arguments[i]->evaluate(env);
            auto &arg = env->pop();
            const auto &id = fn->arguments[i];
            scoped->set_local_var(id, arg);
        }
    }
    // call function
    fn->execute(scoped.get());
    // return value
    if(scoped->stack.empty())
        env->stack.emplace_back();
    else
        env->stack.emplace_back(scoped->pop());
}

void AST::BinaryExpression::evaluate(Environment *env) {
    if( op == OpType::SET  ||
        op == OpType::ADDS ||
        op == OpType::SUBS ||
        op == OpType::MULS ||
        op == OpType::DIVS
    ) {
        right->evaluate(env);
        if(left->type() == IDENTIFIER) {
            const auto &id = dynamic_cast<Identifier*>(left.get())->id;
#define doop(ot, o) \
        if(op == OpType::ot) { env->set_var(id, env->get_var(id) o env->stack.front()); }
            if(op == OpType::SET) env->set_var(id, env->stack.front());
            else doop(ADDS, +)
            else doop(SUBS, -)
            else doop(MULS, *)
            else doop(DIVS, /)
#undef doop
        } else { // member expression
            const auto &id = dynamic_cast<MemberExpression*>(left.get())->id;
            auto dict = env->get_var(id[0]).get<Value::Dictionary>();
            for(size_t i = 1; i < id.size()-1; i++) {
                dict = dict.data.at(id[i]).get<Value::Dictionary>();
            }
            const auto &name = id[id.size()-1];
            const auto &val = env->stack.front();
#define doop(ot, o) \
        if(op == OpType::ot) { dict.data[name] = dict.data[name] o val; }
            if(!dict.type.empty()) {
                if(dict.type.find(name) == dict.type.end())
                    FATAL("Interpreter error", "Cannot set '" + name + "'");
                else if(val.index() != dict.type[name])
                    FATAL("Interpreter error", "Invalid type for '" + name + "'");
            }
            if(op == OpType::SET) dict.data[name] = val;
            else doop(ADDS, +)
            else doop(SUBS, -)
            else doop(MULS, *)
            else doop(DIVS, /)
#undef doop
        }
    } else {
        left->evaluate(env);
        auto left = env->pop();
        right->evaluate(env);
        auto &right = env->pop();
        if     (op == OpType::ADD) env->stack.emplace_back(left +  right);
        else if(op == OpType::SUB) env->stack.emplace_back(left -  right);
        else if(op == OpType::MUL) env->stack.emplace_back(left *  right);
        else if(op == OpType::DIV) env->stack.emplace_back(left /  right);
        else if(op == OpType::EQ)  env->stack.emplace_back(left == right);
        else if(op == OpType::NEQ) env->stack.emplace_back(left != right);
        else if(op == OpType::GT)  env->stack.emplace_back(left >  right);
        else if(op == OpType::LT)  env->stack.emplace_back(left <  right);
        else if(op == OpType::GEQ) env->stack.emplace_back(left >= right);
        else if(op == OpType::LEQ) env->stack.emplace_back(left <= right);
        else if(op == OpType::AND) env->stack.emplace_back(left && right);
        else if(op == OpType::OR)  env->stack.emplace_back(left || right);
    }
}

// Statements
void AST::IfStatement::evaluate(Environment *env) {
    condition->evaluate(env);
    std::unique_ptr<Environment> scoped(env->inherit());
    auto val = env->pop();
    if(val) {
        statement->evaluate(scoped.get());
    } else if(alt.get() != nullptr) {
        alt->evaluate(scoped.get());
    }
}

void AST::WhileStatement::evaluate(Environment *env) {
    std::unique_ptr<Environment> scoped(env->inherit());
    condition->evaluate(env);
    while(env->pop()) {
        statement->evaluate(scoped.get());
        condition->evaluate(env);
    }
}

void AST::ForStatement::evaluate(Environment *env) {
    std::unique_ptr<Environment> scoped(env->inherit());
    this->from->evaluate(env);
    this->to->evaluate(env);
    const auto ti = env->pop().get<int>();
    const auto fi = env->pop().get<int>();
    if(this->step == nullptr) {
        for(int i = fi; i < ti; i += stepN) {
            scoped->set_local_var(id, i);
            statement->evaluate(scoped.get());
        }
    } else {
        step->evaluate(env);
        const auto si = env->pop().get<int>();
        for(int i = fi; i < ti; i += si) {
            scoped->set_local_var(id, i);
            statement->evaluate(scoped.get());
        }
    }
}

void AST::FunctionStatement::evaluate(Environment *env) {
    auto fn = new Value::Function(arguments, statement.get());
    env->set_var(id, fn);
}

void AST::StructStatement::evaluate(Environment *env) {
    Value::Struct s;
    for(auto &d : dict) {
        auto k = d.first, v = d.second;
        if(v == "integer")
            s.type[k] = Value::variant_index<int>();
        else if(v == "float")
            s.type[k] = Value::variant_index<float>();
        else if(v == "string")
            s.type[k] = Value::variant_index<std::string>();
        else if(v == "array")
            s.type[k] = Value::variant_index<Value::Array>();
        else if(v == "function")
            s.type[k] = Value::variant_index<Value::IFunction*>();
        else if(v == "dict")
            s.type[k] = Value::variant_index<Value::Dictionary>();
        else {
            // TODO struct
//             s->type[k] = Value::Type::DICTIONARY;
//             s->structs[k] = env->get_var(v).get_struct();
        }
    }
    env->set_local_var(id, s);
}

void AST::ExpressionStatement::evaluate(Environment *env) {
//     LOG("expression stmt");
    expression->evaluate(env);
    env->pop();
}

void AST::ReturnStatement::evaluate(Environment *env) {
    throw this;
}

// Block
void AST::Block::evaluate(Environment *env) {
    for(auto &ast : statements) {
        ast->evaluate(env);
    }
}

void AST::BlockStatement::evaluate(Environment *env) {
    for(auto &ast : statements) {
        ast->evaluate(env);
    }
}
