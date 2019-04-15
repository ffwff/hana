#include <iostream>
#include <memory>
#include <cmath>
#include "src/scriptparser.h"
#include "vm/src/vm.h"
#include "vm/src/dict.h"
#include "vm/src/array_obj.h"
#ifdef LREADLINE
#include <readline/readline.h>
#include <readline/history.h>
#endif

namespace hanayo {

static std::string to_string(struct value &val) {
    if(val.type == value::TYPE_STR)
        return val.as.str;
    else if(val.type == value::TYPE_INT)
        return std::to_string(val.as.integer);
    else if(val.type == value::TYPE_FLOAT)
        return std::to_string(val.as.floatp);
    else if(val.type == value::TYPE_NATIVE_FN || val.type == value::TYPE_FN)
        return "(function)";
    else if(val.type == value::TYPE_DICT)
        return "(record)";
    else if(val.type == value::TYPE_ARRAY) {
        std::string s = "[";
        if(val.as.array->data.length)
            s += to_string(val.as.array->data.data[0]);
        for(size_t i = 1; i < val.as.array->data.length; i++)
            s += ", " + to_string(val.as.array->data.data[i]);
        s += "]";
        return s;
    }
    return "(nil)";
}

#define fn(name) void name(struct vm *vm, int nargs)

fn(print) {
    int written = 0;
    while(nargs--) {
        struct value val = array_top(vm->stack);
        auto s = to_string(val);
        written += fputs(s.data(), stdout);
        value_free(&val);
        array_pop(vm->stack);
    }
    struct value val;
    value_int(&val, written);
    array_push(vm->stack, val);
}

// data types
namespace string {

    fn(constructor) {
        assert(nargs == 1);
        struct value val = array_top(vm->stack);
        auto s = to_string(val);
        value_free(&val);
        array_pop(vm->stack);
        value_str(&val, s.data());
        array_push(vm->stack, val);
        vm_print_stack(vm);
    }
    fn(bytesize) {
        assert(nargs == 1);
        struct value *val = &array_top(vm->stack);
        int64_t length = strlen(val->as.str);
        value_free(val);
        array_pop(vm->stack);
        struct value intv;
        value_int(&intv, length);
        array_push(vm->stack, intv);
    }
    fn(length) { // TODO unicode char length
        assert(nargs == 1);
        struct value *val = &array_top(vm->stack);
        int64_t length = strlen(val->as.str);
        value_free(val);
        array_pop(vm->stack);
        struct value intv;
        value_int(&intv, length);
        array_push(vm->stack, intv);
    }
    fn(delete_) {
        assert(nargs == 3);

        struct value val = {0};

        // string
        val = array_top(vm->stack);
        std::string s(val.as.str);
        value_free(&val);
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        array_pop(vm->stack);
        int64_t from_pos = val.as.integer;

        // n chars
        val = array_top(vm->stack);
        array_pop(vm->stack);
        int64_t nchars = val.as.integer;

        s.erase(from_pos, nchars);
        value_str(&val, s.data());
        array_push(vm->stack, val);
    }
    fn(copy) {
        assert(nargs == 3);

        struct value val = {0};

        // string
        val = array_top(vm->stack);
        std::string s(val.as.str);
        value_free(&val);
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        assert(val.type == value::TYPE_INT);
        array_pop(vm->stack);
        int64_t from_pos = val.as.integer;

        // n chars
        val = array_top(vm->stack);
        assert(val.type == value::TYPE_INT);
        array_pop(vm->stack);
        int64_t nchars = val.as.integer;

        char str[nchars+1];
        s.copy(str, nchars, from_pos);
        str[nchars] = 0;
        value_str(&val, str);
        array_push(vm->stack, val);
    }
    fn(at) {
        assert(nargs == 2);
        struct value val = {0};

        // string
        val = array_top(vm->stack);
        std::string s(val.as.str);
        value_free(&val);
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        assert(val.type == value::TYPE_INT);
        array_pop(vm->stack);
        int64_t index = val.as.integer;

        if(index < (int64_t)(s.size()) && index >= 0) {
            char str[2] = { s[index], 0 };
            value_str(&val, str);
            array_push(vm->stack, val);
        } else {
            array_push(vm->stack, {0});
        }
    }
    fn(index) {
        assert(nargs == 2);
        struct value val = {0};

        // string
        val = array_top(vm->stack);
        std::string src(val.as.str);
        value_free(&val);
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        assert(val.type == value::TYPE_STR);
        std::string needle(val.as.str);
        value_free(&val);
        array_pop(vm->stack);

        size_t index = src.find(needle);
        if(index == std::string::npos) {
            value_int(&val, -1);
        } else {
            value_int(&val, (int64_t)index);
        }
        array_push(vm->stack, val);
    }
    fn(insert) {
        assert(nargs == 3);

        struct value val = {0};

        // string
        val = array_top(vm->stack);
        std::string s(val.as.str);
        value_free(&val);
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        assert(val.type == value::TYPE_INT);
        array_pop(vm->stack);
        int64_t from_pos = val.as.integer;

        val = array_top(vm->stack);
        assert(val.type == value::TYPE_STR);
        s.insert(from_pos, val.as.str);
        value_free(&val);

        value_str(&array_top(vm->stack), s.data());
    }
}

namespace integer {
    fn(constructor) {
        struct value *val = &array_top(vm->stack);
        if(val->type == value::TYPE_INT)
            return;
        else if(val->type == value::TYPE_FLOAT)
            value_int(val, (int64_t)val->as.floatp);
        else if(val->type == value::TYPE_STR)
            value_int(val, std::stoi(val->as.str));
        else
            value_int(val, 0);
    }
}

namespace float_ {
    fn(constructor) {
        struct value *val = &array_top(vm->stack);
        if(val->type == value::TYPE_FLOAT)
            return;
        else if(val->type == value::TYPE_INT)
            value_float(val, (double)val->as.integer);
        else if(val->type == value::TYPE_STR)
            value_float(val, std::stof(val->as.str));
        else
            value_float(val, 0);
    }
    fn(round) {
        assert(nargs == 1);
        struct value *val = &array_top(vm->stack);
        value_int(val, (int64_t)::round(val->as.floatp));
    }
}

namespace record {
    fn(constructor) {
        struct value val; value_dict(&val);
        array_push(vm->stack, val);
    }
}

namespace array {
    fn(constructor) {
        struct value aval;
        if(nargs == 0) value_array(&aval);
        else {
            value_array_n(&aval, nargs);
            aval.as.array->data.length = nargs;
            for(size_t i = 0; i < (size_t)nargs; i++) {
                struct value val = array_top(vm->stack);
                value_copy(&aval.as.array->data.data[i], &val);
                array_pop(vm->stack);
                value_free(&val);
            }
        }
        array_push(vm->stack, aval);
    }

    fn(length) {
        assert(nargs == 1);
        struct value *val = &array_top(vm->stack);
        int64_t length = val->as.array->data.length;
        value_free(val);
        value_int(val, length);
    }
    fn(delete_) {
        assert(nargs == 3);

        struct value val = {0};

        // array
        val = array_top(vm->stack);
        struct value aval;
        value_copy(&aval, &val);
        value_free(&val);
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        array_pop(vm->stack);
        size_t from_pos = (size_t)val.as.integer;

        // n chars
        val = array_top(vm->stack);
        array_pop(vm->stack);
        size_t nchars = (size_t)val.as.integer;

        assert(from_pos >= 0 && from_pos+nchars < aval.as.array->data.length);
        for(size_t i = from_pos; i < (from_pos+nchars); i++) {
            value_free(&aval.as.array->data.data[i]);
        }
        size_t remaining = (aval.as.array->data.length-nchars)*sizeof(struct value);
        memmove(&aval.as.array->data.data[from_pos],
                &aval.as.array->data.data[from_pos+nchars], remaining);
        aval.as.array->data.length -= nchars;
        array_push(vm->stack, aval);
    }
    fn(copy) {
        assert(nargs == 3);

        struct value val = {0};

        // string
        val = array_top(vm->stack);
        struct value aval;
        value_copy(&aval, &val);
        value_free(&val);
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        assert(val.type == value::TYPE_INT);
        array_pop(vm->stack);
        int64_t from_pos = val.as.integer;

        // n chars
        val = array_top(vm->stack);
        assert(val.type == value::TYPE_INT);
        array_pop(vm->stack);
        int64_t nchars = val.as.integer;

        if(nchars == 0) {
            value_free(&aval);
            value_array(&aval);
            array_push(vm->stack, val);
        } else {
            struct value newval;
            value_array_n(&newval, nchars);
            for(size_t i = (size_t)from_pos, j = 0; i < (size_t)(from_pos+nchars); i++, j++) {
                value_copy(&newval.as.array->data.data[j], &aval.as.array->data.data[i]);
            }
            newval.as.array->data.length = nchars;
            value_free(&aval);
            array_push(vm->stack, newval);
        }
    }
    fn(at) {
        assert(nargs == 2);
        struct value val = {0};

        // string
        val = array_top(vm->stack);
        struct value aval;
        value_copy(&aval, &val);
        value_free(&val);
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        assert(val.type == value::TYPE_INT);
        array_pop(vm->stack);
        int64_t index = val.as.integer;

        if(index < (int64_t)(aval.as.array->data.length) && index >= 0) {
            struct value val;
            value_copy(&val, &aval.as.array->data.data[index]);
            value_free(&aval);
            array_push(vm->stack, val);
        } else {
            value_free(&aval);
            array_push(vm->stack, {0});
        }
    }
    fn(index) {
        assert(nargs == 2);
        struct value val = {0};

        // string
        val = array_top(vm->stack);
        struct value aval = val;
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        struct value needle;
        value_copy(&needle, &val);
        value_free(&val);
        array_pop(vm->stack);

        array_push(vm->stack, {0});
        for(size_t i = 0; i < aval.as.array->data.length; i++) {
            struct value result;
            value_eq(&result, &needle, &aval.as.array->data.data[i]);
            if(result.as.integer == 1) {
                value_free(&needle);
                value_free(&aval);
                value_int(&array_top(vm->stack), (int64_t)i);
                return;
            }
        }
        value_free(&needle);
        value_free(&aval);
        value_int(&array_top(vm->stack), 0);
    }
    fn(insert) {
        assert(nargs == 3);

        struct value val = {0};

        // string
        struct value aval;
        value_copy(&aval, &array_top(vm->stack));
        value_free(&array_top(vm->stack));
        array_pop(vm->stack);

        // from pos
        val = array_top(vm->stack);
        assert(val.type == value::TYPE_INT);
        array_pop(vm->stack);
        int64_t from_pos = val.as.integer;

        // new val
        val = array_top(vm->stack);

        if(from_pos < 0) {
            value_free(&aval);
            array_push(vm->stack, {0});
            return;
        } else if(from_pos >= (int64_t)aval.as.array->data.length) {
            for(int64_t i = aval.as.array->data.length; i < from_pos+1; i++)
                array_push(aval.as.array->data, {0});
        } else {
            array_push(aval.as.array->data, {0}); // reserve space
        }
        size_t n = (aval.as.array->data.length-from_pos)*sizeof(struct value);
        memmove(&aval.as.array->data.data[from_pos+1],
                &aval.as.array->data.data[from_pos],
                n
        );
        value_copy(&aval.as.array->data.data[from_pos], &val);
        value_free(&aval);

    }
    fn(push) {
        assert(nargs == 2);

        struct value aval;
        value_copy(&aval, &array_top(vm->stack));
        value_free(&array_top(vm->stack));
        array_pop(vm->stack);

        array_push(aval.as.array->data, array_top(vm->stack));
        value_free(&array_top(vm->stack));
        value_copy(&array_top(vm->stack), &aval);
        value_free(&aval);
    }
    fn(pop) {
        assert(nargs == 1);
        struct value *aval = &array_top(vm->stack);
        if(aval->as.array->data.length == 0) {
            value_free(&array_top(vm->stack));
            return;
        }
        value_free(&aval->as.array->data.data[aval->as.array->data.length-1]);
        aval->as.array->data.length--;
        value_free(&array_top(vm->stack));
    }
    fn(sort) {
        assert(nargs == 1);
        //array_pop(vm->stack);
        struct value *aval = &array_top(vm->stack);
        struct value val;
        value_array_n(&val, aval->as.array->data.length);
        for(size_t i = 0; i < aval->as.array->data.length; i++)
            value_copy(&val.as.array->data.data[i], &aval->as.array->data.data[i]);
        val.as.array->data.length = aval->as.array->data.length;
        qsort(val.as.array->data.data, val.as.array->data.length, sizeof(struct value), [](const void *a, const void *b) -> int {
            const struct value *va = (const struct value *)a,
              *vb = (const struct value *)b;
              struct value ret = {0};
              value_lt(&ret, va, vb);
              if(ret.as.integer == 1) return -1;
              value_eq(&ret, va, vb);
              if(ret.as.integer == 1) return 0;
              return 1;
        });
        value_free(aval);
        array_pop(vm->stack);
        array_push(vm->stack, val);
    }
    fn(sort_) {
        assert(nargs == 1);
        struct value *val = &array_top(vm->stack);
        qsort(val->as.array->data.data, val->as.array->data.length, sizeof(struct value), [](const void *a, const void *b) -> int {
            const struct value *va = (const struct value *)a,
                               *vb = (const struct value *)b;
            struct value ret = {0};
            value_lt(&ret, va, vb);
            if(ret.as.integer == 1) return -1;
            value_eq(&ret, va, vb);
            if(ret.as.integer == 1) return 0;
            return 1;
        });
    }
}

#undef fn

}

int main(int argc, char **argv) {
    // virtual machine
    struct vm m; vm_init(&m);
    // variables:
    struct value val;
    // # constants
    val = {0}; env_set(m.env, "nil", &val);
    value_int(&val, 0); env_set(m.env, "false", &val);
    value_int(&val, 1); env_set(m.env, "true", &val);
    // # functions
#define native_function(name) \
    value_native(&val, hanayo::name);  env_set(m.env, #name, &val);
#define native_function_key(name, key) \
    value_native(&val, hanayo::name);  env_set(m.env, key, &val);
    native_function(print)
    // # objects
#define native_obj_function(key, name) \
    do{ struct value v; value_native(&v, hanayo::name); hmap_set(&val.as.dict->data, key, &v); } while(0)

    value_dict(&val);
    native_obj_function("constructor", string::constructor);
    native_obj_function("bytesize",    string::bytesize);
    native_obj_function("length",      string::length);
    native_obj_function("delete",      string::delete_);
    native_obj_function("copy",        string::copy);
    native_obj_function("at",          string::at);
    native_obj_function("index",       string::index);
    native_obj_function("insert",      string::insert);
    env_set(m.env, "string", &val);
    value_free(&val);
    m.dstr = val.as.dict;

    value_dict(&val);
    native_obj_function("constructor", integer::constructor);
    env_set(m.env, "integer", &val);
    value_free(&val);
    m.dint = val.as.dict;

    value_dict(&val);
    native_obj_function("constructor", float_::constructor);
    native_obj_function("round",       float_::round);
    env_set(m.env, "float", &val);
    value_free(&val);
    m.dfloat = val.as.dict;

    value_dict(&val);
    native_obj_function("constructor", array::constructor);
    native_obj_function("length",      array::length);
    native_obj_function("delete",      array::delete_);
    native_obj_function("copy",        array::copy);
    native_obj_function("at",          array::at);
    native_obj_function("index",       array::index);
    native_obj_function("insert",      array::insert);
    native_obj_function("push",        array::push);
    native_obj_function("pop",         array::pop);
    native_obj_function("sort",        array::sort);
    native_obj_function("sort!",       array::sort_);
    env_set(m.env, "array", &val);
    value_free(&val);
    m.darray = val.as.dict;

    if(argc == 2) {
        Hana::ScriptParser p;
        p.loadf(argv[1]);
        auto ast = std::unique_ptr<Hana::AST::AST>(p.parse());
        #if defined(DEBUG)
        ast->print();
        #endif
        ast->emit(&m);
        array_push(m.code, OP_HALT);
        vm_execute(&m);
        goto cleanup;
    } else {
        #ifdef LREADLINE
        rl_bind_key('\t', rl_insert);
        #endif
        while(1) {
            #ifdef LREADLINE
            int nread = 0;
            std::string s;
            while(1) {
                char *buf = readline(nread == 0 ? ">> " : "");
                if(buf == nullptr) {
                    goto cleanup;
                } else if(buf[strlen(buf)-1] == '\\') {
                    add_history(buf);
                    buf[strlen(buf)-1] = 0;
                    s += buf;
                    s += '\n';
                    nread++;
                    free(buf);
                } else {
                    add_history(buf);
                    s += buf;
                    s += '\n';
                    free(buf);
                    break;
                }
            }
            #else
            std::cout << ">> ";
            std::string s, line;
            while(1) {
                std::getline(std::cin, line);
                if(std::cin.eof()) goto cleanup;
                if(line[line.size()-1] == '\\') {
                    line.resize(line.size()-1);
                    s += line+'\n';
                } else {
                    s += line+'\n';
                    break;
                }
            }
            #endif
            if(s.empty()) continue;
            Hana::ScriptParser p;
            p.loads(s);
            auto ast = std::unique_ptr<Hana::AST::AST>(p.parse());
            ast->emit(&m);
            array_push(m.code, OP_HALT);
            vm_execute(&m);
            m.ip += 1;
            std::cout << std::flush;
        }
    }

cleanup:
    vm_free(&m);
    return 0;
}
