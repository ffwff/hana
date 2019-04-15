#include "ast.h"
#include "error.h"
#include "vm/src/vm.h"
#include "vm/src/xxhash.h"

using namespace Hana;

static void pindent(int levels) {
    while(levels--)
        std::cout << " ";
}

// DEBUG
// Literals
void AST::StrLiteral::print(int indent) {
    pindent(indent);
    std::cout << "\"" << str << "\"\n";
}
void AST::IntLiteral::print(int indent) {
    pindent(indent);
    std::cout << i << '\n';
}
void AST::FloatLiteral::print(int indent) {
    pindent(indent);
    std::cout << f << '\n';
}
void AST::Identifier::print(int indent) {
    pindent(indent);
    std::cout << id << '\n';
}
void AST::Array::print(int indent) {
}

//
void AST::UnaryExpression::print(int indent) {
    pindent(indent);
    std::cout << "unaryexpr\n";
    body->print(indent+1);
}
void AST::MemberExpression::print(int indent) {
    pindent(indent);
    std::cout << "memexpr\n";
    left->print(indent+1);
    right->print(indent+1);
}
void AST::CallExpression::print(int indent) {
    pindent(indent);
    std::cout << "callexpr\n";
    callee->print(indent+1);
    for(auto &arg : arguments)
        arg->print(indent+1);
}
void AST::BinaryExpression::print(int indent) {
    pindent(indent);
    std::cout << "binexpr\n";
    left->print(indent+1);
    pindent(indent+1);
    if(op == ADD) std::cout << "+";
    else if(op == SUB) std::cout << "-";
    else if(op == MUL) std::cout << "*";
    else if(op == DIV) std::cout << "/";
    else if(op == MOD) std::cout << "mod";
    else if(op == AND) std::cout << "and";
    else if(op == OR) std::cout << "or";
    else if(op == EQ) std::cout << "==";
    else if(op == NEQ) std::cout << "!=";
    else if(op == GT) std::cout << ">";
    else if(op == LT) std::cout << "<";
    else if(op == GEQ) std::cout << ">=";
    else if(op == LEQ) std::cout << "<=";
    else if(op == SET) std::cout << "=";
    else if(op == ADDS) std::cout << "+=";
    else if(op == SUBS) std::cout << "-=";
    else if(op == MULS) std::cout << "*=";
    else if(op == DIVS) std::cout << "/=";
    std::cout << "\n";
    right->print(indent+1);
}
void AST::ConditionalExpression::print(int indent) {
    pindent(indent);
    std::cout << "conditional\n";
    condition->print(indent+1);
    expression->print(indent+1);
    alt->print(indent+1);
}

void AST::IfStatement::print(int indent) {
    pindent(indent);
    std::cout << "if\n";
    condition->print(indent+1);
    statement->print(indent+1);
    if(alt) alt->print(indent+1);
}
void AST::WhileStatement::print(int indent) {
    pindent(indent);
    std::cout << "while\n";
    condition->print(indent+1);
    statement->print(indent+1);
}
void AST::ForStatement::print(int indent) {
    pindent(indent);
    std::cout << "for\n";
    from->print(indent+1);
    to->print(indent+1);
    if(step) step->print(indent+1);
    else {
        pindent(indent+1);
        std::cout << stepN << "\n";
    }
    statement->print(indent+1);
}
void AST::FunctionStatement::print(int indent) {
    pindent(indent);
    std::cout << "function " <<id<< "\n";
    for(auto &arg : arguments) {
        pindent(indent);
        std::cout << arg << "\n";
    }
    statement->print(indent+1);
}
void AST::StructStatement::print(int indent) {
    pindent(indent);
    std::cout << "struct\n";
    for(auto &s : statements)
        s->print(indent+1);
}
void AST::ExpressionStatement::print(int indent) {
    pindent(indent);
    std::cout << "expr stmt\n";
    expression->print(indent+1);
}
void AST::ReturnStatement::print(int indent) {
    pindent(indent);
    std::cout << "return\n";
    if(expression) expression->print(indent+1);
}

void AST::Block::print(int indent) {
    pindent(indent);
    std::cout << "block\n";
    for(auto &s : statements)
        s->print(indent);
}
void AST::BlockStatement::print(int indent) {
    pindent(indent);
    std::cout << "block\n";
    for(auto &s : statements)
        s->print(indent+1);
}

// EMIT

// DEBUG
// Literals
void AST::StrLiteral::emit(struct vm *vm) {
    array_push(vm->code, OP_PUSHSTR);
    vm_code_pushstr(vm, str.data());
}
void AST::IntLiteral::emit(struct vm *vm) {
    const uint64_t ui = (uint64_t)i;
    if(ui <= 0xff) {
        array_push(vm->code, OP_PUSH8);
        array_push(vm->code, ui);
    } else if (ui <= 0xfff) {
        array_push(vm->code, OP_PUSH16);
        vm_code_push16(vm, ui);
    } else if (ui <= 0xfffff) {
        array_push(vm->code, OP_PUSH32);
        vm_code_push32(vm, ui);
    } else { // 64-bit int
        array_push(vm->code, OP_PUSH64);
        vm_code_push64(vm, ui);
    }
}
void AST::FloatLiteral::emit(struct vm *vm) {
    array_push(vm->code, OP_PUSHF64);
    vm_code_pushf64(vm, f);
}
void AST::Identifier::emit(struct vm *vm) {
    array_push(vm->code, OP_GET);
    vm_code_pushstr(vm, id.data());
    vm_code_push32(vm, XXH32(id.data(), id.size(), 0));
}
void AST::Array::emit(struct vm *vm) {
    for(auto &v : values)
        v->emit(vm);
    const uint64_t ui = (uint64_t)values.size();
    if(ui <= 0xff) {
        array_push(vm->code, OP_PUSH8);
        array_push(vm->code, ui);
    } else if (ui <= 0xfff) {
        array_push(vm->code, OP_PUSH16);
        vm_code_push16(vm, ui);
    } else if (ui <= 0xfffff) {
        array_push(vm->code, OP_PUSH32);
        vm_code_push32(vm, ui);
    } else { // 64-bit int
        array_push(vm->code, OP_PUSH64);
        vm_code_push64(vm, ui);
    }
    array_push(vm->code, OP_ARRAY_LOAD);
}

//
#define FILLER32 0xdeadbeef
static void fill_hole(struct vm *vm, size_t length, size_t n) {
    vm->code.data[length+0] = (n >> 12) & 0xff;
    vm->code.data[length+1] = (n >> 8) & 0xff;
    vm->code.data[length+2] = (n >> 4) & 0xff;
    vm->code.data[length+3] = (n >> 0) & 0xff;
}

void AST::UnaryExpression::emit(struct vm *vm) {
    body->emit(vm);
    if(op == NEG) array_push(vm->code, OP_NEGATE);
    else if(op == NOT) array_push(vm->code, OP_NOT);
}
void AST::MemberExpression::emit(struct vm *vm) {
    left->emit(vm);
    if((right->type() == IDENTIFIER || right->type() == STR_LITERAL) && !is_expr) {
        const char *key = right->type() == IDENTIFIER
                        ? static_cast<Identifier*>(right.get())->id.data()
                        : static_cast<StrLiteral*>(right.get())->str.data();
        if(is_called) array_push(vm->code, OP_MEMBER_GET_NO_POP);
        else array_push(vm->code, OP_MEMBER_GET);
        vm_code_pushstr(vm, key);
        vm_code_push32(vm, XXH32(key, strlen(key), 0));
    } else {
        right->emit(vm);
        array_push(vm->code, OP_INDEX_GET);
    }
}
void AST::CallExpression::emit(struct vm *vm) {
    for(auto arg = arguments.rbegin(); arg != arguments.rend(); arg++)
        (*arg)->emit(vm);
    callee->emit(vm);
    array_push(vm->code, OP_CALL);
    if(callee->type() == MEMBER_EXPR)
        array_push(vm->code, arguments.size()+1);
    else
        array_push(vm->code, arguments.size());
}
void AST::BinaryExpression::emit(struct vm *vm) {
    if(op == SET || op == ADDS || op == SUBS || op == MULS || op == DIVS) {
        if(op == ADDS || op == SUBS || op == MULS || op == DIVS) {
            left->emit(vm);
        }
    #define EMIT_OP \
        if(op == ADDS) array_push(vm->code, OP_ADD); \
        else if(op == SUBS) array_push(vm->code, OP_SUB); \
        else if(op == MULS) array_push(vm->code, OP_MUL); \
        else if(op == DIVS) array_push(vm->code, OP_DIV);

        if(left->type() == IDENTIFIER) {
            right->emit(vm);
            EMIT_OP
            auto s = static_cast<Identifier*>(left.get())->id;
            array_push(vm->code, OP_SET);
            vm_code_pushstr(vm, s.data());
        } else if(left->type() == MEMBER_EXPR) { // member expr
            right->emit(vm);
            EMIT_OP
            auto mem = static_cast<MemberExpression*>(left.get());
            mem->left->emit(vm);
            if((mem->right->type() == IDENTIFIER || mem->right->type() == STR_LITERAL) && !mem->is_expr) {
                const char *key = mem->right->type() == IDENTIFIER
                    ? static_cast<Identifier*>(mem->right.get())->id.data()
                    : static_cast<StrLiteral*>(mem->right.get())->str.data();
                array_push(vm->code, OP_MEMBER_SET);
                vm_code_pushstr(vm, key);
            } else { // TODO
                mem->right->emit(vm);
                array_push(vm->code, OP_INDEX_SET);
            }
        } else if(left->type() == CALL_EXPR) {
            auto expr = static_cast<CallExpression*>(left.get());
            array_push(vm->code, OP_DEF_FUNCTION_PUSH);
            assert(expr->callee->type() == IDENTIFIER);
            const auto id = static_cast<Identifier*>(expr->callee.get())->id.data();
            vm_code_pushstr(vm, id);
            size_t length = vm->code.length;
            vm_code_push32(vm, FILLER32);
            array_push(vm->code, (uint8_t)(expr->arguments.size()));
            //body
            size_t body_start = vm->code.length;
            for(auto &arg : expr->arguments) {
                assert(arg->type() == IDENTIFIER);
                array_push(vm->code, OP_SET_LOCAL);
                vm_code_pushstr(vm, static_cast<Identifier*>(arg.get())->id.data());
                array_push(vm->code, OP_POP);
            }
            // primitive tail call optimizations
            if(right->type() == COND_EXPR) {
                auto c = static_cast<ConditionalExpression*>(right.get());
                if( c->expression->type() == CALL_EXPR || c->alt->type() == CALL_EXPR ) {
                    c->condition->emit(vm);
                    array_push(vm->code, OP_JNCOND);
                    size_t length = vm->code.length;
                    vm_code_push32(vm, FILLER32);
                    // then statement
                #define OPTIMIZE_BRANCH(branch) \
                    if( c->branch->type() == CALL_EXPR && \
                        static_cast<CallExpression*>(c->branch.get())->callee->type() == IDENTIFIER && \
                        static_cast<Identifier*>(static_cast<CallExpression*>(c->branch.get())->callee.get())->id == id) { \
                        auto call = static_cast<CallExpression*>(c->branch.get()); \
                        for(auto arg = call->arguments.rbegin(); arg != call->arguments.rend(); arg++) \
                        { (*arg)->emit(vm); } \
                        array_push(vm->code, OP_JMP); \
                        vm_code_push32(vm, body_start); \
                    } else { \
                        c->branch->emit(vm);\
                    }
                    OPTIMIZE_BRANCH(expression)
                    // alt
                    array_push(vm->code, OP_JMP);
                    size_t length1 = vm->code.length;
                    vm_code_push32(vm, FILLER32);
                    size_t n = vm->code.length;
                    OPTIMIZE_BRANCH(alt)
                    size_t n1 = vm->code.length;
                    fill_hole(vm, length, n);
                    fill_hole(vm, length1, n1);
                } else // no optimization possible
                    right->emit(vm);
            } else {
                right->emit(vm);
            }
            array_push(vm->code, OP_RET); // pops env for us
            //fill in
            fill_hole(vm, length, vm->code.length);
            array_push(vm->code, OP_SET);
            vm_code_pushstr(vm, id);
        }
    }
    else {
        left->emit(vm);
        right->emit(vm);
        if(op == ADD)      array_push(vm->code, OP_ADD);
        else if(op == SUB) array_push(vm->code, OP_SUB);
        else if(op == MUL) array_push(vm->code, OP_MUL);
        else if(op == DIV) array_push(vm->code, OP_DIV);
        else if(op == MOD) array_push(vm->code, OP_MOD);
        else if(op == AND) array_push(vm->code, OP_AND);
        else if(op == OR)  array_push(vm->code, OP_OR );
        else if(op == EQ)  array_push(vm->code, OP_EQ );
        else if(op == NEQ) array_push(vm->code, OP_NEQ);
        else if(op == GT)  array_push(vm->code, OP_GT );
        else if(op == LT)  array_push(vm->code, OP_LT );
        else if(op == GEQ) array_push(vm->code, OP_GEQ);
        else if(op == LEQ) array_push(vm->code, OP_LEQ);
    }
}
void AST::ConditionalExpression::emit(struct vm *vm) {
    condition->emit(vm);
    array_push(vm->code, OP_JNCOND);
    size_t length = vm->code.length;
    vm_code_push32(vm, FILLER32);
    // then statement
    expression->emit(vm);
    // alt
    array_push(vm->code, OP_JMP);
    size_t length1 = vm->code.length;
    vm_code_push32(vm, FILLER32);
    size_t n = vm->code.length;
    alt->emit(vm);
    size_t n1 = vm->code.length;
    fill_hole(vm, length, n);
    fill_hole(vm, length1, n1);
}

void AST::IfStatement::emit(struct vm *vm) {
    // condition
    // jcond [else]
    // [statement]
    // jmp done
    // [else]
    // done:
    condition->emit(vm);
    array_push(vm->code, OP_JNCOND);
    size_t length = vm->code.length;
    vm_code_push32(vm, FILLER32);
    // then statement
    statement->emit(vm);
    if(alt) {
        array_push(vm->code, OP_JMP);
        size_t length1 = vm->code.length;
        vm_code_push32(vm, FILLER32);
        size_t n = vm->code.length;
        alt->emit(vm);
        size_t n1 = vm->code.length;
        fill_hole(vm, length, n);
        fill_hole(vm, length1, n1);
    } else {
        size_t n = vm->code.length;
        fill_hole(vm, length, n);
    }
}
void AST::WhileStatement::emit(struct vm *vm) {
    // 1: jmp condition
    // [statement]
    // [condition]
    // jcond 1
    array_push(vm->code, OP_JMP);
    size_t length = vm->code.length;
    vm_code_push32(vm, FILLER32);
    size_t length1 = vm->code.length;
    statement->emit(vm);
    fill_hole(vm, length, vm->code.length);
    condition->emit(vm);
    array_push(vm->code, OP_JCOND);
    vm_code_push32(vm, length1);
}
void AST::ForStatement::emit(struct vm *vm) {
    // start
    from->emit(vm);
    array_push(vm->code, OP_SET_LOCAL);
    vm_code_pushstr(vm, id.data());
    array_push(vm->code, OP_POP);
    // body
    size_t body_pos = vm->code.length;
    statement->emit(vm);
    // condition:
    // [body]
    // get [id]
    // [to]
    // neq
    // jcond [1]
    // step
    // jmp [body]
    // 1: done
    array_push(vm->code, OP_GET);
    vm_code_pushstr(vm, id.data());
    vm_code_push32(vm, XXH32(id.data(), id.size(), 0));
    to->emit(vm);
    if(stepN == 1) array_push(vm->code, OP_GEQ);
    else array_push(vm->code, OP_LEQ);

    array_push(vm->code, OP_JCOND);
    size_t length = vm->code.length;
    vm_code_push32(vm, FILLER32);

    if(step) {
        step->emit(vm);
        array_push(vm->code, OP_ADDS);
        vm_code_pushstr(vm, id.data());
        array_push(vm->code, OP_POP);
    } else if(stepN == 1) {
        array_push(vm->code, OP_INC);
        vm_code_pushstr(vm, id.data());
    } else if(stepN == -1) {
        array_push(vm->code, OP_DEC);
        vm_code_pushstr(vm, id.data());
    }
    array_push(vm->code, OP_JMP);
    vm_code_push32(vm, body_pos);

    fill_hole(vm, length, vm->code.length);
}
void AST::FunctionStatement::emit(struct vm *vm) {
    if(record_fn) array_push(vm->code, OP_DEF_FUNCTION_PUSH);
    else array_push(vm->code, OP_DEF_FUNCTION);
    vm_code_pushstr(vm, id.data());
    size_t length = vm->code.length;
    vm_code_push32(vm, FILLER32);
    array_push(vm->code, (uint8_t)arguments.size());
    //body
    for(auto &arg : arguments) {
        array_push(vm->code, OP_SET_LOCAL);
        vm_code_pushstr(vm, arg.data());
        array_push(vm->code, OP_POP);
    }
    statement->emit(vm);
    // default return
    array_push(vm->code, OP_PUSH_NIL);
    array_push(vm->code, OP_RET); // pops env for us
    //fill in
    fill_hole(vm, length, vm->code.length);
}
void AST::StructStatement::emit(struct vm *vm) {
    array_push(vm->code, OP_PUSH_NIL);
    for(auto &s : statements) {
        if(s->type() == EXPR_STMT) {
            auto expr = static_cast<BinaryExpression*>(
                static_cast<ExpressionStatement*>(s.get())->expression.get());
            assert(expr->left->type() == IDENTIFIER);
            expr->right->emit(vm);
            array_push(vm->code, OP_PUSHSTR);
            vm_code_pushstr(vm, static_cast<Identifier*>(expr->left.get())->id.data());
        } else if(s->type() == FUNCTION_STMT) {
            auto fn = static_cast<FunctionStatement*>(s.get());
            fn->emit(vm);
            array_push(vm->code, OP_PUSHSTR);
            vm_code_pushstr(vm, fn->id.data());
        }
    }
    array_push(vm->code, OP_DICT_LOAD);
    array_push(vm->code, OP_SET);
    vm_code_pushstr(vm, id.data());
    array_push(vm->code, OP_POP);
}
void AST::ExpressionStatement::emit(struct vm *vm) {
    expression->emit(vm);
    array_push(vm->code, OP_POP);
}
void AST::ReturnStatement::emit(struct vm *vm) {
    expression->emit(vm);
    array_push(vm->code, OP_RET);
}

void AST::Block::emit(struct vm *vm) {
    for(auto &s : statements)
        s->emit(vm);
}
void AST::BlockStatement::emit(struct vm *vm) {
    for(auto &s : statements)
        s->emit(vm);
}
