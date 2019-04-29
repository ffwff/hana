#include "ast.h"
#include "error.h"
#include "vm/src/vm.h"
#include "vm/src/xxhash.h"
#include <stdio.h>

using namespace Hana;

// macros
#define START_AST \
Hana::Compiler::SourceMap *_SRC_MAP = new Hana::Compiler::SourceMap( \
    vm->code.length, start_line, end_line \
); \
compiler->src_maps.emplace_back(_SRC_MAP);

#define END_AST \
_SRC_MAP->end_byte = vm->code.length;

// indent
static void pindent(int levels) {
    while(levels--)
        fputs(" ", stdout);
}

// DEBUG
// Literals
void AST::StrLiteral::print(int indent) {
    pindent(indent);
    printf("\"%s\"\n", str.data());
}
void AST::IntLiteral::print(int indent) {
    pindent(indent);
    printf("%ld\n", i);
}
void AST::FloatLiteral::print(int indent) {
    pindent(indent);
    printf("%f\n", f);
}
void AST::Identifier::print(int indent) {
    pindent(indent);
    printf("%s\n", id.data());
}
void AST::Array::print(int indent) {
}

// Expressions
void AST::UnaryExpression::print(int indent) {
    pindent(indent);
    fputs("unaryexpr\n", stdout);
    body->print(indent+1);
}
void AST::MemberExpression::print(int indent) {
    pindent(indent);
    fputs("memexpr\n", stdout);
    left->print(indent+1);
    right->print(indent+1);
}
void AST::CallExpression::print(int indent) {
    pindent(indent);
    fputs("callexpr\n", stdout);
    callee->print(indent+1);
    for(auto &arg : arguments)
        arg->print(indent+1);
}
void AST::BinaryExpression::print(int indent) {
    pindent(indent);
    fputs("binexpr\n", stdout);
    left->print(indent+1);
    pindent(indent+1);
    if(op == ADD) fputs("+", stdout);
    else if(op == SUB) fputs("-", stdout);
    else if(op == MUL) fputs("*", stdout);
    else if(op == DIV) fputs("/", stdout);
    else if(op == MOD) fputs("mod", stdout);
    else if(op == AND) fputs("and", stdout);
    else if(op == OR) fputs("or", stdout);
    else if(op == EQ) fputs("==", stdout);
    else if(op == NEQ) fputs("!=", stdout);
    else if(op == GT) fputs(">", stdout);
    else if(op == LT) fputs("<", stdout);
    else if(op == GEQ) fputs(">=", stdout);
    else if(op == LEQ) fputs("<=", stdout);
    else if(op == SET) fputs("=", stdout);
    else if(op == ADDS) fputs("+=", stdout);
    else if(op == SUBS) fputs("-=", stdout);
    else if(op == MULS) fputs("*=", stdout);
    else if(op == DIVS) fputs("/=", stdout);
    fputs("\n", stdout);
    right->print(indent+1);
}
void AST::ConditionalExpression::print(int indent) {
    pindent(indent);
    fputs("conditional\n", stdout);
    condition->print(indent+1);
    expression->print(indent+1);
    alt->print(indent+1);
}

// Statements
void AST::IfStatement::print(int indent) {
    pindent(indent);
    printf("if (%ld->%ld)\n", start_line, end_line);
    condition->print(indent+1);
    statement->print(indent+1);
    if(alt) alt->print(indent+1);
}
void AST::WhileStatement::print(int indent) {
    pindent(indent);
    printf("while (%ld->%ld)\n", start_line, end_line);
    condition->print(indent+1);
    statement->print(indent+1);
}
void AST::ForStatement::print(int indent) {
    pindent(indent);
    printf("for (%ld->%ld)\n", start_line, end_line);
    from->print(indent+1);
    to->print(indent+1);
    if(step) step->print(indent+1);
    else {
        pindent(indent+1);
        printf("%d\n", stepN);
    }
    statement->print(indent+1);
}
void AST::ContinueStatement::print(int indent) {
    pindent(indent);
    printf("continue (%ld->%ld)\n", start_line, end_line);
}
void AST::BreakStatement::print(int indent) {
    pindent(indent);
    printf("break (%ld->%ld)\n", start_line, end_line);
}
void AST::FunctionStatement::print(int indent) {
    pindent(indent);
    printf("function (%ld->%ld)\n", start_line, end_line);
    for(auto &arg : arguments) {
        pindent(indent);
        printf("%s\n", arg.data());
    }
    statement->print(indent+1);
}
void AST::StructStatement::print(int indent) {
    pindent(indent);
    printf("struct (%ld->%ld)\n", start_line, end_line);
    for(auto &s : statements)
        s->print(indent+1);
}
void AST::ExpressionStatement::print(int indent) {
    pindent(indent);
    printf("expr stmt (%ld->%ld)\n", start_line, end_line);
    expression->print(indent+1);
}
void AST::ReturnStatement::print(int indent) {
    pindent(indent);
    printf("return (%ld->%ld)\n", start_line, end_line);
    if(expression) expression->print(indent+1);
}
void AST::CaseStatement::print(int indent) {
    pindent(indent);
    printf("case (%ld->%ld)\n", start_line, end_line);
    if(etype) etype->print(indent+1);
    for(auto &s : statements)
        s->print(indent+1);
}
void AST::TryStatement::print(int indent) {
    pindent(indent);
    printf("try (%ld->%ld)\n", start_line, end_line);
    for(auto &s : statements)
        s->print(indent+1);
    for(auto &s : cases)
        s->print(indent+1);
}
void AST::RaiseStatement::print(int indent) {
    pindent(indent);
    printf("raise (%ld->%ld)\n", start_line, end_line);
    if(expression) expression->print(indent+1);
}

void AST::Block::print(int indent) {
    pindent(indent);
    fputs("block\n", stdout);
    for(auto &s : statements)
        s->print(indent+1);
}
void AST::BlockStatement::print(int indent) {
    pindent(indent);
    printf("block stmt (%ld->%ld)\n", start_line, end_line);
    for(auto &s : statements)
        s->print(indent+1);
}

// EMIT
static void emit_set_var(struct vm *vm, Hana::Compiler *compiler, std::string s, bool is_function_def=false) {
    if((s.size() > 1 && s[0] == '$') || compiler->scopes.size() == 0) { // global var
        if(s.size() > 1 && s[0] == '$') s.erase(0, 1);
        LOG("set global var ", s);
        array_push(vm->code, OP_SET_GLOBAL);
        vm_code_pushstr(vm, s.data());
    } else { // local var
        auto local = compiler->get_local(s);
        if(local.relascope != 0) {
            compiler->set_local(s);
            local = compiler->get_local(s);
            LOG("actually -1");
        }
        array_push(vm->code, is_function_def ? OP_SET_LOCAL_FUNCTION_DEF : OP_SET_LOCAL);
        LOG("set local var ", s, ":", local.relascope);
        vm_code_push16(vm, local.idx);
    }
}
void debug(){}
static void emit_get_var(struct vm *vm, Hana::Compiler *compiler, std::string s) {
    if((s.size() > 1 && s[0] == '$') || compiler->scopes.size() == 0) { // global var
        if(s.size() > 1 && s[0] == '$') s.erase(0, 1);
        array_push(vm->code, OP_GET_GLOBAL);
        vm_code_pushstr(vm, s.data());
        vm_code_push32(vm, XXH32(s.data(), s.size(), 0));
    } else { // local var
        auto local = compiler->get_local(s);
        LOG("get local var ", s, ":", local.relascope);
        if(local.relascope == (size_t)-1) {
            array_push(vm->code, OP_GET_GLOBAL);
            vm_code_pushstr(vm, s.data());
            vm_code_push32(vm, XXH32(s.data(), s.size(), 0));
        } else if(local.relascope == 0) {
            array_push(vm->code, OP_GET_LOCAL);
            vm_code_push16(vm, local.idx);
        } else {
            array_push(vm->code, OP_GET_LOCAL_UP);
            vm_code_push16(vm, local.idx);
            vm_code_push16(vm, local.relascope);
        }
    }
}

// emits
// Literals
void AST::StrLiteral::emit(struct vm *vm, Hana::Compiler *compiler) {
    array_push(vm->code, OP_PUSHSTR);
    vm_code_pushstr(vm, str.data());
}
void AST::IntLiteral::emit(struct vm *vm, Hana::Compiler *compiler) {
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
void AST::FloatLiteral::emit(struct vm *vm, Hana::Compiler *compiler) {
    array_push(vm->code, OP_PUSHF64);
    vm_code_pushf64(vm, f);
}
void AST::Identifier::emit(struct vm *vm, Hana::Compiler *compiler) {
    emit_get_var(vm, compiler, id);
}
void AST::Array::emit(struct vm *vm, Hana::Compiler *compiler) {
    for(auto &v : values)
        v->emit(vm, compiler);
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

// helpers
#define FILLER32 0xdeadbeef
static void fill_hole(struct vm *vm, uint32_t length, uint32_t n) {
    vm->code.data[length+0] = (n >> 12) & 0xff;
    vm->code.data[length+1] = (n >> 8) & 0xff;
    vm->code.data[length+2] = (n >> 4) & 0xff;
    vm->code.data[length+3] = (n >> 0) & 0xff;
}
static void fill_hole16(struct vm *vm, uint32_t length, uint16_t n) {
    vm->code.data[length+0] = (n >> 4) & 0xff;
    vm->code.data[length+1] = (n >> 0) & 0xff;
}

static void emit_tail_call(AST::CallExpression *call, struct vm *vm, Hana::Compiler *compiler) {
    for(auto arg = call->arguments.rbegin(); arg != call->arguments.rend(); arg++)
        { (*arg)->emit(vm, compiler); }
    call->callee->emit(vm, compiler);
    array_push(vm->code, OP_RETCALL);
    if( call->callee->type() == AST::MEMBER_EXPR &&
        !static_cast<AST::MemberExpression*>(call->callee.get())->is_namespace )
        vm_code_push16(vm, call->arguments.size()+1);
    else
        vm_code_push16(vm, call->arguments.size());
}

//
void AST::UnaryExpression::emit(struct vm *vm, Hana::Compiler *compiler) {
    body->emit(vm, compiler);
    if(op == NEG) array_push(vm->code, OP_NEGATE);
    else if(op == NOT) array_push(vm->code, OP_NOT);
}
void AST::MemberExpression::emit(struct vm *vm, Hana::Compiler *compiler) {
    if(is_namespace) {
        const auto &k = static_cast<Identifier*>(right.get())->id;
        left->emit(vm, compiler);
        array_push(vm->code, OP_MEMBER_GET);
        vm_code_pushstr(vm, k.data());
        vm_code_push32(vm, XXH32(k.data(), k.size(), 0));
        return;
    }

    left->emit(vm, compiler);
    if((right->type() == IDENTIFIER || right->type() == STR_LITERAL) && !is_expr) {
        const char *key = right->type() == IDENTIFIER
                        ? static_cast<Identifier*>(right.get())->id.data()
                        : static_cast<StrLiteral*>(right.get())->str.data();
        if(is_called) array_push(vm->code, OP_MEMBER_GET_NO_POP);
        else array_push(vm->code, OP_MEMBER_GET);
        vm_code_pushstr(vm, key);
        vm_code_push32(vm, XXH32(key, strlen(key), 0));
    } else {
        right->emit(vm, compiler);
        array_push(vm->code, OP_INDEX_GET);
    }
}
void AST::CallExpression::emit(struct vm *vm, Hana::Compiler *compiler) {
    for(auto arg = arguments.rbegin(); arg != arguments.rend(); arg++)
        (*arg)->emit(vm, compiler);
    callee->emit(vm, compiler);
    array_push(vm->code, OP_CALL);
    if( callee->type() == MEMBER_EXPR &&
        !static_cast<MemberExpression*>(callee.get())->is_namespace )
        vm_code_push16(vm, arguments.size()+1);
    else
        vm_code_push16(vm, arguments.size());
}
void AST::BinaryExpression::emit(struct vm *vm, Hana::Compiler *compiler) {
    if(op == SET || op == ADDS || op == SUBS || op == MULS || op == DIVS) { // assignment
        if(op == ADDS || op == SUBS || op == MULS || op == DIVS) {
            left->emit(vm, compiler);
        }
    #define EMIT_OP \
        if(op == ADDS) array_push(vm->code, OP_ADD); \
        else if(op == SUBS) array_push(vm->code, OP_SUB); \
        else if(op == MULS) array_push(vm->code, OP_MUL); \
        else if(op == DIVS) array_push(vm->code, OP_DIV);

        if(left->type() == IDENTIFIER) {
            right->emit(vm, compiler);
            EMIT_OP
            auto s = static_cast<Identifier*>(left.get())->id;
            emit_set_var(vm, compiler, s);
        } else if(left->type() == MEMBER_EXPR) { // member expr
            right->emit(vm, compiler);
            EMIT_OP
            auto mem = static_cast<MemberExpression*>(left.get());
            mem->left->emit(vm, compiler);
            if((mem->right->type() == IDENTIFIER || mem->right->type() == STR_LITERAL) && !mem->is_expr) {
                const char *key = mem->right->type() == IDENTIFIER
                    ? static_cast<Identifier*>(mem->right.get())->id.data()
                    : static_cast<StrLiteral*>(mem->right.get())->str.data();
                array_push(vm->code, OP_MEMBER_SET);
                vm_code_pushstr(vm, key);
            } else { // TODO
                mem->right->emit(vm, compiler);
                array_push(vm->code, OP_INDEX_SET);
            }
        } else if(left->type() == CALL_EXPR) {
            auto expr = static_cast<CallExpression*>(left.get());
            array_push(vm->code, OP_DEF_FUNCTION_PUSH);
            //assert(expr->callee->type() == IDENTIFIER);
            vm_code_push16(vm, (uint16_t)(expr->arguments.size()));
            LOG((uint16_t)(expr->arguments.size()));

            const auto id = static_cast<Identifier*>(expr->callee.get())->id;
            if(!(id.size()>1 && id[0] == '$'))
                compiler->set_local(id);

            uint32_t length = vm->code.length;
            vm_code_push32(vm, FILLER32);

            //body
            compiler->scope();
            array_push(vm->code, OP_ENV_NEW);
            uint32_t env_length = vm->code.length;
            vm_code_push16(vm, 0xFFFF);
            for(auto &arg : expr->arguments) {
                assert(arg->type() == IDENTIFIER);
                auto s = static_cast<Identifier*>(arg.get())->id;
                emit_set_var(vm, compiler, s);
                array_push(vm->code, OP_POP);
            }
            if(right->type() == CALL_EXPR) {
                emit_tail_call(static_cast<CallExpression*>(right.get()), vm, compiler);
            } else if(right->type() == COND_EXPR) {
                auto c = static_cast<ConditionalExpression*>(right.get());
                if( c->expression->type() == CALL_EXPR || c->alt->type() == CALL_EXPR ) {
                    c->condition->emit(vm, compiler);
                    array_push(vm->code, OP_JNCOND);
                    uint32_t length = vm->code.length;
                    vm_code_push32(vm, FILLER32);
                    // then statement
                #define OPTIMIZE_BRANCH(branch) \
                    if( c->branch->type() == CALL_EXPR ) { \
                        emit_tail_call(static_cast<CallExpression*>(c->branch.get()), vm, compiler); \
                    } else { \
                        c->branch->emit(vm, compiler);\
                    }
                    OPTIMIZE_BRANCH(expression)
                    // alt
                    array_push(vm->code, OP_JMP);
                    uint32_t length1 = vm->code.length;
                    vm_code_push32(vm, FILLER32);
                    uint32_t n = vm->code.length;
                    OPTIMIZE_BRANCH(alt)
                    uint32_t n1 = vm->code.length;
                    fill_hole(vm, length, n);
                    fill_hole(vm, length1, n1);
                } else // no optimization possible
                    right->emit(vm, compiler);
            } else {
                right->emit(vm, compiler);
            }
            auto scope_size = compiler->unscope();
            fill_hole16(vm, env_length, scope_size);
            array_push(vm->code, OP_RET);
            //fill in
            fill_hole(vm, length, vm->code.length);
            LOG("len: ", vm->code.length);
            emit_set_var(vm, compiler, id, true);
        }
    }
    else {
        left->emit(vm, compiler);
        right->emit(vm, compiler);
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
void AST::ConditionalExpression::emit(struct vm *vm, Hana::Compiler *compiler) {
    condition->emit(vm, compiler);
    array_push(vm->code, OP_JNCOND);
    uint32_t length = vm->code.length;
    vm_code_push32(vm, FILLER32);
    // then statement
    expression->emit(vm, compiler);
    // alt
    array_push(vm->code, OP_JMP);
    uint32_t length1 = vm->code.length;
    vm_code_push32(vm, FILLER32);
    uint32_t n = vm->code.length;
    alt->emit(vm, compiler);
    uint32_t n1 = vm->code.length;
    fill_hole(vm, length, n);
    fill_hole(vm, length1, n1);
}

// Statements
void AST::IfStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    // condition
    // jcond [else]
    // [statement]
    // jmp done
    // [else]
    // done:
    condition->emit(vm, compiler);
    array_push(vm->code, OP_JNCOND);
    uint32_t length = vm->code.length;
    vm_code_push32(vm, FILLER32);
    // then statement
    statement->emit(vm, compiler);
    if(alt) {
        array_push(vm->code, OP_JMP);
        uint32_t length1 = vm->code.length;
        vm_code_push32(vm, FILLER32);
        uint32_t n = vm->code.length;
        alt->emit(vm, compiler);
        uint32_t n1 = vm->code.length;
        fill_hole(vm, length, n);
        fill_hole(vm, length1, n1);
    } else {
        uint32_t n = vm->code.length;
        fill_hole(vm, length, n);
    }
    END_AST
}
void AST::WhileStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    // 1: jmp condition
    // [statement]
    // [condition]
    // jcond 1
    array_push(vm->code, OP_JMP);
    uint32_t length = vm->code.length;
    vm_code_push32(vm, FILLER32);

    compiler->loop_stmts.emplace_back();

    uint32_t length1 = vm->code.length;
    statement->emit(vm, compiler);
    fill_hole(vm, length, vm->code.length);

    uint32_t next_it_pos = vm->code.length;
    condition->emit(vm, compiler);
    array_push(vm->code, OP_JCOND);
    vm_code_push32(vm, length1);

    uint32_t end_pos = vm->code.length;

    auto top = compiler->loop_stmts.back();
    for(auto cont : top.fill_continue)
        fill_hole(vm, cont, next_it_pos);
    for(auto brk : top.fill_break)
        fill_hole(vm, brk, end_pos);

    compiler->loop_stmts.pop_back();
    END_AST
}
void AST::ForStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    // start
    from->emit(vm, compiler);
    emit_set_var(vm, compiler, id);
    array_push(vm->code, OP_POP);
    // body
    uint32_t body_pos = vm->code.length;
    compiler->loop_stmts.emplace_back();
    statement->emit(vm, compiler);
    // condition:
    // [body]
    // get [id]
    // [to]
    // neq
    // jcond [1]
    // step
    // jmp [body]
    // 1: done
    uint32_t next_it_pos = vm->code.length;
    emit_get_var(vm, compiler, id);
    to->emit(vm, compiler);
    if(stepN == 1) array_push(vm->code, OP_GEQ);
    else array_push(vm->code, OP_LEQ);

    array_push(vm->code, OP_JCOND);
    uint32_t length = vm->code.length;
    vm_code_push32(vm, FILLER32);

    if(step) {
        emit_get_var(vm, compiler, id);
        step->emit(vm, compiler);
        array_push(vm->code, OP_ADD);
        emit_set_var(vm, compiler, id);
        array_push(vm->code, OP_POP);
    } else {
        emit_get_var(vm, compiler, id);
        array_push(vm->code, OP_PUSH8);
        array_push(vm->code, 1);
        if(stepN == -1) array_push(vm->code, OP_SUB);
        else array_push(vm->code, OP_ADD);
        emit_set_var(vm, compiler, id);
        array_push(vm->code, OP_POP);
    }
    array_push(vm->code, OP_JMP);
    vm_code_push32(vm, body_pos);

    uint32_t end_pos = vm->code.length;
    fill_hole(vm, length, end_pos);

    auto top = compiler->loop_stmts.back();
    for(auto cont : top.fill_continue)
        fill_hole(vm, cont, next_it_pos);
    for(auto brk : top.fill_break)
        fill_hole(vm, brk, end_pos);

    compiler->loop_stmts.pop_back();
    END_AST
}
void AST::ContinueStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    assert(!compiler->loop_stmts.empty());
    array_push(vm->code, OP_JMP);
    uint32_t pos = vm->code.length;
    vm_code_push32(vm, FILLER32);
    compiler->loop_stmts.back().fill_continue.emplace_back(pos);
    END_AST
}
void AST::BreakStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    assert(!compiler->loop_stmts.empty());
    array_push(vm->code, OP_JMP);
    uint32_t pos = vm->code.length;
    vm_code_push32(vm, FILLER32);
    compiler->loop_stmts.back().fill_break.emplace_back(pos);
    END_AST
}
void AST::FunctionStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    array_push(vm->code, OP_DEF_FUNCTION_PUSH);
    vm_code_push16(vm, arguments.size());
    uint32_t function_end = vm->code.length;
    vm_code_push32(vm, FILLER32);
    compiler->set_local(id);
    // scope
    compiler->scope();
    array_push(vm->code, OP_ENV_NEW);
    uint32_t env_length = vm->code.length;
    vm_code_push16(vm, 0xFFFF);
    // body
    for(auto &arg : arguments) {
        emit_set_var(vm, compiler, arg);
        array_push(vm->code, OP_POP);
    }
    statement->emit(vm, compiler);
    auto scope_size = compiler->unscope();
    fill_hole16(vm, env_length, scope_size);
    // default return
    if( vm->code.data[vm->code.length] != OP_RET &&
        vm->code.data[vm->code.length] != OP_RETCALL) {
        array_push(vm->code, OP_PUSH_NIL);
        array_push(vm->code, OP_RET); // pops env for us
    }
    // fill holes
    fill_hole(vm, function_end, vm->code.length);

    // push set var
    if(!is_expr) {
        emit_set_var(vm, compiler, id, true);
        array_push(vm->code, OP_POP);
    }
    END_AST
}
void AST::StructStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    if(statements.empty()) {
        array_push(vm->code, OP_DICT_NEW);
        return;
    }
    array_push(vm->code, OP_PUSH_NIL);
    for(auto &s : statements) {
        if(s->type() == EXPR_STMT) {
            auto expr = static_cast<BinaryExpression*>(
                static_cast<ExpressionStatement*>(s.get())->expression.get());
            assert(expr->left->type() == IDENTIFIER);
            expr->right->emit(vm, compiler);
            array_push(vm->code, OP_PUSHSTR);
            vm_code_pushstr(vm, static_cast<Identifier*>(expr->left.get())->id.data());
        } else if(s->type() == FUNCTION_STMT) {
            auto fn = static_cast<FunctionStatement*>(s.get());
            fn->emit(vm, compiler);
            array_push(vm->code, OP_PUSHSTR);
            vm_code_pushstr(vm, fn->id.data());
        } else if(s->type() == STRUCT_STMT) {
            auto ss = static_cast<StructStatement*>(s.get());
            ss->emit(vm, compiler);
            array_push(vm->code, OP_PUSHSTR);
            vm_code_pushstr(vm, ss->id.data());
        }
    }
    array_push(vm->code, OP_DICT_LOAD);
    emit_set_var(vm, compiler, id);
    if(!is_expr) array_push(vm->code, OP_POP);
    END_AST
}
void AST::ExpressionStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    expression->emit(vm, compiler);
    array_push(vm->code, OP_POP);
    END_AST
}
void AST::ReturnStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    if(expression == nullptr)
        array_push(vm->code, OP_PUSH_NIL);
    else if(expression->type() == CALL_EXPR) {
        // recursive tail call, perform tco
        emit_tail_call(static_cast<CallExpression*>(expression.get()), vm, compiler);
        END_AST
        return;
    }
    else
        expression->emit(vm, compiler);
    array_push(vm->code, OP_RET);
    END_AST
}
void AST::TryStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    array_push(vm->code, OP_PUSH_NIL);
    std::vector<uint32_t> cases_to_fill;
    // case statements is generated as a function
    for(auto &case_stmt : cases) {
        // function will take in 1 arg if id is set
        array_push(vm->code, OP_DEF_FUNCTION_PUSH);
        vm_code_push16(vm, !case_stmt->id.empty());
        uint32_t body_start = vm->code.length;
        vm_code_push32(vm, FILLER32);
        // id
        if(!case_stmt->id.empty())
            emit_set_var(vm, compiler, case_stmt->id);
        array_push(vm->code, OP_POP);
        // body
        for(auto &s : case_stmt->statements)
            s->emit(vm, compiler);
        array_push(vm->code, OP_EXFRAME_RET);
        uint32_t body_end = vm->code.length;
        vm_code_push32(vm, FILLER32);
        cases_to_fill.push_back(body_end);
        // end
        fill_hole(vm, body_start, vm->code.length);
        // exception type
        case_stmt->etype->emit(vm, compiler);
    }
    array_push(vm->code, OP_TRY);
    for(auto &s : statements)
        s->emit(vm, compiler);
    for(auto hole : cases_to_fill)
        fill_hole(vm, hole, vm->code.length);
    END_AST
}
void AST::RaiseStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    START_AST
    if(expression == nullptr)
        array_push(vm->code, OP_PUSH_NIL);
    else
        expression->emit(vm, compiler);
    array_push(vm->code, OP_RAISE);
    END_AST
}

void AST::Block::emit(struct vm *vm, Hana::Compiler *compiler) {
    for(auto &s : statements)
        s->emit(vm, compiler);
}
void AST::BlockStatement::emit(struct vm *vm, Hana::Compiler *compiler) {
    for(auto &s : statements)
        s->emit(vm, compiler);
}
