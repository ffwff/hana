#include "ast.h"
#include "error.h"
#include "vm/src/vm.h"

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
    if(i <= 0xff) {
        array_push(vm->code, OP_PUSH8);
        array_push(vm->code, i);
    } else if (i <= 0xffff) {
        array_push(vm->code, OP_PUSH16);
        vm_code_push16(vm, i);
    } else if ((uint32_t)i <= 0xffffffff) {
        array_push(vm->code, OP_PUSH32);
        vm_code_push32(vm, i);
    } else {
        FATAL("Interpreter error", "64-bits not supported!");
    }
}
void AST::FloatLiteral::emit(struct vm *vm) {
}
void AST::Identifier::emit(struct vm *vm) {
    array_push(vm->code, OP_GET);
    vm_code_pushstr(vm, id.data());
}

//
#define FILLER64 0xbadc0ffee0ddf00d
static void fill_hole(struct vm *vm, size_t length, size_t n) {
    vm->code.data[length]   = (n >> 28) & 0xff;
    vm->code.data[length+1] = (n >> 24) & 0xff;
    vm->code.data[length+2] = (n >> 20) & 0xff;
    vm->code.data[length+3] = (n >> 16) & 0xff;
    vm->code.data[length+4] = (n >> 12) & 0xff;
    vm->code.data[length+5] = (n >> 8) & 0xff;
    vm->code.data[length+6] = (n >> 4) & 0xff;
    vm->code.data[length+7] = (n >> 0) & 0xff;
}

void AST::UnaryExpression::emit(struct vm *vm) {
}
void AST::MemberExpression::emit(struct vm *vm) {
}
void AST::CallExpression::emit(struct vm *vm) {
    for(auto arg = arguments.rbegin(); arg != arguments.rend(); ++arg)
        (*arg)->emit(vm);
    callee->emit(vm);
    array_push(vm->code, OP_CALL);
    array_push(vm->code, arguments.size());
}
void AST::BinaryExpression::emit(struct vm *vm) {
    if(op == SET) {
        right->emit(vm);
        if(left->type() == IDENTIFIER) {
            auto s = static_cast<Identifier*>(left.get())->id;
            array_push(vm->code, OP_SET);
            vm_code_pushstr(vm, s.data());
        }
    } else {
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

        else if(op == SET)  array_push(vm->code, OP_ADD);
        else if(op == ADDS) array_push(vm->code, OP_ADD);
        else if(op == SUBS) array_push(vm->code, OP_ADD);
        else if(op == MULS) array_push(vm->code, OP_ADD);
        else if(op == DIVS) array_push(vm->code, OP_ADD);
    }
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
    vm_code_push64(vm, FILLER64);
    // then statement
    statement->emit(vm);
    if(alt) {
        array_push(vm->code, OP_JMP);
        size_t length1 = vm->code.length;
        vm_code_push64(vm, FILLER64);
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
    vm_code_push64(vm, FILLER64);
    size_t length1 = vm->code.length;
    statement->emit(vm);
    fill_hole(vm, length, vm->code.length);
    condition->emit(vm);
    array_push(vm->code, OP_JCOND);
    vm_code_push64(vm, length1);
}
void AST::ForStatement::emit(struct vm *vm) {
    // start
    from->emit(vm);
    array_push(vm->code, OP_SET);
    vm_code_pushstr(vm, id.data());
    array_push(vm->code, OP_POP);
    // jmp to condition check (1)
    array_push(vm->code, OP_JMP);
    size_t length = vm->code.length;
    vm_code_push64(vm, FILLER64);
    size_t body_pos = vm->code.length;
    // body
    statement->emit(vm);
    // fill jmp (1)
    size_t n = vm->code.length;
    fill_hole(vm, length, n);
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
    to->emit(vm);
    array_push(vm->code, OP_EQ);

    array_push(vm->code, OP_JCOND);
    length = vm->code.length;
    vm_code_push64(vm, FILLER64);

    if(step) {
        // TODO
        FATAL("Interpreter error", "Not implemented");
    } else if(stepN == 1) {
        array_push(vm->code, OP_INC);
        vm_code_pushstr(vm, id.data());
    } else if(stepN == -1) {
        array_push(vm->code, OP_DEC);
        vm_code_pushstr(vm, id.data());
    }
    array_push(vm->code, OP_JMP);
    vm_code_push64(vm, body_pos);

    n = vm->code.length;
    fill_hole(vm, length, n);
}
void AST::FunctionStatement::emit(struct vm *vm) {
    array_push(vm->code, OP_DEF_FUNCTION);
    vm_code_pushstr(vm, id.data());
    size_t length = vm->code.length;
    vm_code_push64(vm, FILLER64);
    //body
    array_push(vm->code, OP_POP); // narg
    array_push(vm->code, OP_ENV_INHERIT);
    for(auto &arg : arguments) {
        array_push(vm->code, OP_SET_LOCAL);
        vm_code_pushstr(vm, arg.data());
        array_push(vm->code, OP_POP);
    }
    statement->emit(vm);
    // default return
    array_push(vm->code, OP_PUSH8);
    array_push(vm->code, 0);
    array_push(vm->code, OP_RET); // pops env for us
    //fill in
    fill_hole(vm, length, vm->code.length);
}
void AST::StructStatement::emit(struct vm *vm) {
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
