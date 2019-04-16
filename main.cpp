#include <iostream>
#include <memory>
#include <cmath>
#include "src/scriptparser.h"
#include "hanayo/hanayo.h"
#ifdef LREADLINE
#include <readline/readline.h>
#include <readline/history.h>
#endif

int main(int argc, char **argv) {
    // virtual machine
    struct vm m; vm_init(&m);
    hanayo::_init(&m);

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
