#pragma once
#include <fstream>
#include <sstream>
#include <cassert>
#include <stack>
#include "error.h"

namespace {

class Parser {
private:

    struct Position {
        std::streamoff pos;
        int lines;
    };
    std::stack<Position> fposs;

protected:

    std::stringstream f;
    int lines = 0;

    // File position
    void fsave() {
        fposs.push({
            .pos = f.tellg(),
            .lines = lines
        });
    }
    void floadn() {
        const auto p = fposs.top();
        f.clear();
        f.seekg(p.pos);
        lines = p.lines;
    }
    void fload() {
        const auto p = fposs.top();
        f.clear();
        f.seekg(p.pos);
        lines = p.lines;
        fposs.pop();
    }
    void fpop() {
        fposs.pop();
    }
    size_t fpsize() const {
        return fposs.size();
    }

    // Tokeniser
    class Token {
    public:
        std::string strv;
        union {
            int intv;
            float floatv;
        };
        enum Type {
            NONE, NEWLINE, STRING, STRLITERAL, INTEGER, FLOAT, OPERATOR
        } type;
        Token() : type(NONE) {};
        Token(bool nl) : type(NEWLINE) {};
        Token(std::string str) : strv(str), type(STRING) {};
        Token(std::string str, const enum Type type) : strv(str), type(type) {};
        Token(int intv) : intv(intv), type(INTEGER) {};
        Token(float floatv) : floatv(floatv), type(FLOAT) {};
    };

    virtual Token next()=0;
    std::string nexts() {
        const Token t = next();
        if(t.type != Token::STRING)
            FATAL("Lexer error", "Expected string value");
        return t.strv;
    }
    void nexteq(const std::string &s) {
        const Token t = next();
        assert(t.type == Token::STRING);
        LOG(t.strv);
        assert(t.strv == s);
    }
    int nexti() {
        const Token t = next();
        if(t.type != Token::INTEGER)
            FATAL("Lexer error", "Expected string value");
        return t.intv;
    }
    int nextf() {
        const Token t = next();
        if(t.type != Token::FLOAT)
            FATAL("Lexer error", "Expected string value");
        return t.floatv;
    }
    void nextnl() {
        const Token t = next();
        assert(t.type == Token::NONE || t.type == Token::NEWLINE);
    }

public:
    void loadf(const char *file) {
        std::ifstream f(file);
        this->f << f.rdbuf();

    };
    void loads(const char *s) {
        f = std::stringstream(s);
    };
    void parse();
};


}
