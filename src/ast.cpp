#include "ast.h"
#include "error.h"
#include <array>

using namespace Hana;

// evaluation
//-----------------------------
// constant
void AST::Constant::evaluate(Environment *env) {
    env->stack.emplace_back(value);
}

void AST::Identifier::evaluate(Environment *env) {
    // Constants
    LOG(id);
    if(id == "nil") env->stack.emplace_back();
    else {
        auto &val = env->get_var(id);
        env->stack.emplace_back(Value(static_cast<Value::Ref>(&val)));
    }
}

// expressions
void AST::UnaryExpression::evaluate(Environment *env) {
    body->evaluate(env);
    if(op == NEG) env->stack.emplace_back(-env->pop());
    else if(op == POS) env->stack.emplace_back(+env->pop());
    else if(op == NOT) env->stack.emplace_back(!env->pop());
}

void AST::MemberExpression::evaluate(Environment *env) {
    left->evaluate(env);
    auto l = env->pop();
    Value::Dictionary dict;
    if(l.is_type<Value::Dictionary>())
        dict = l.get<Value::Dictionary>();
    else if(l.is_type<Value::Ref>() && l.get<Value::Ref>()->is_type<Value::Dictionary>())
        dict = l.get<Value::Ref>()->get<Value::Dictionary>();
    else
        FATAL("Interpreter error", "Left hand side must be dictionary");
    LOG(right->type());
    right->evaluate(env);
    auto key = env->pop().get<std::string>();
    LOG(dict.data.size());
    try {
        env->stack.emplace_back(Value(static_cast<Value::Ref>(&dict.data.at(key))));
    } catch(const std::out_of_range&) {
        FATAL("Interpreter error", "Expected ", key, " key to exist");
    }
}

void AST::CallExpression::evaluate(Environment *env) {
    callee->evaluate(env);
    const auto value = env->pop();
    std::unique_ptr<Environment> scoped(env->inherit());

    // type cast
    if(value.is_type<Hana::Type::Value>()) {
        // pass argument to scoped for further processing
        for(size_t i = 0; i < arguments.size(); i++) {
            arguments[i]->evaluate(env);
            scoped->stack.emplace_back(env->pop());
        }
        value.get<Hana::Type::Value>()->fn(scoped.get());
        env->stack.emplace_back(scoped->pop());
        return;
    }

    // function call
    if(!value.is_type<Value::IFunction*>())
        FATAL("Interpreter error", "value is not callable");
    auto fn = value.get<Value::IFunction*>();
    if(fn->is_variable) {
        // pass argument to scoped for further processing
        for(size_t i = 0; i < arguments.size(); i++) {
            arguments[i]->evaluate(env);
            scoped->stack.emplace_back(env->pop());
        }
    } else {
        if(arguments.size() != fn->arguments.size())
            FATAL("Interpreter error", "argument sizes don't match for");
        // set function arguments
        for(size_t i = 0; i < arguments.size(); i++) {
            arguments[i]->evaluate(env);
            auto &arg = env->pop();
            const auto &id = fn->arguments[i];
            LOG(id);
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
            const auto &id = static_cast<Identifier*>(left.get())->id;
#define doop(ot, o) \
        if(op == OpType::ot) { env->set_var(id, env->get_var(id) o env->stack.front()); }
            if(op == OpType::SET) env->set_var(id, env->stack.front());
            else doop(ADDS, +)
            else doop(SUBS, -)
            else doop(MULS, *)
            else doop(DIVS, /)
#undef doop
        } else { // member expression
            const auto expr = static_cast<MemberExpression*>(left.get());
            expr->left->evaluate(env);
            const auto &l = env->pop();
            if(!l.is_type<Value::Ref>())
                FATAL("", "not type dictionary");
            auto &dict = l.get<Value::Ref>()->get<Value::Dictionary>();
            expr->right->evaluate(env);
            const auto &name = env->pop().get<std::string>();
            const auto &val = env->stack.front();
#define doop(ot, o) \
        if(op == OpType::ot) { dict.data[name] = dict.data[name] o val; }
            if(!dict.types.empty()) {
                if(dict.types.find(name) == dict.types.end())
                    FATAL("Interpreter error", "Cannot set '" + name + "'");
                else if(val.index() != dict.types[name])
                    FATAL("Interpreter error", "Invalid type for '" + name + "'");
            }
            if(op == OpType::SET) dict.data[name] = val;
            else doop(ADDS, +)
            else doop(SUBS, -)
            else doop(MULS, *)
            else doop(DIVS, /)
#undef doop
            LOG(dict.data.size());
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
        else if(op == OpType::MOD) env->stack.emplace_back(left %  right);
        else if(op == OpType::EQ)  env->stack.emplace_back(left == right);
        else if(op == OpType::NEQ) env->stack.emplace_back(left != right);
        else if(op == OpType::GT)  env->stack.emplace_back(left >  right);
        else if(op == OpType::LT)  env->stack.emplace_back(left <  right);
        else if(op == OpType::GEQ) env->stack.emplace_back(left >= right);
        else if(op == OpType::LEQ) env->stack.emplace_back(left <= right);
        else if(op == OpType::AND) env->stack.emplace_back(left && right);
        else if(op == OpType::OR)  env->stack.emplace_back(left || right);
        else FATAL("not implemented here", op);
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
    const auto fi = env->pop().get<int>();

    this->to->evaluate(env);
    auto ti = env->pop().get<int>();

    if(this->step == nullptr) {
        if(stepN == 1) ti++;
        else if(stepN == -1) ti--;
        for(int i = fi; i != ti; i += stepN) {
            scoped->set_local_var(id, i);
            statement->evaluate(scoped.get());
        }
    } else {
        step->evaluate(env);
        const auto si = env->pop().get<int>();
        for(int i = fi; i != ti; i += si) {
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
            s.types[k] = Value::variant_index<int>();
        else if(v == "float")
            s.types[k] = Value::variant_index<float>();
        else if(v == "string")
            s.types[k] = Value::variant_index<std::string>();
        else if(v == "array")
            s.types[k] = Value::variant_index<Value::Array>();
        else if(v == "function")
            s.types[k] = Value::variant_index<Value::IFunction*>();
        else if(v == "dict")
            s.types[k] = Value::variant_index<Value::Dictionary>();
        else {
            // TODO struct
//             s->types[k] = Value::Type::DICTIONARY;
//             s->structs[k] = env->get_var(v).get_struct();
        }
    }
    env->set_local_var(id, s);
}

void AST::ExpressionStatement::evaluate(Environment *env) {
    LOG("expression stmt");
    expression->evaluate(env);
    env->pop();
    LOG(env->stack.size());
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
