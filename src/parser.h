#pragma once
#include <fstream>
#include <sstream>
#include <cassert>
#include <stack>
#include <exception>
#include "error.h"

namespace Hana {

class Parser {
private:

    struct Position {
        std::streamoff pos;
        size_t lines;
    };
    std::stack<Position> fposs;

protected:

    std::stringstream f;
    bool ended = false;
    size_t lines = 0;

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
            throw LexerError("Expected string value");
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
            throw LexerError("Expected integer value");
        return t.intv;
    }
    int nextf() {
        const Token t = next();
        if(t.type != Token::FLOAT)
            throw LexerError("Expected float value");
        return t.floatv;
    }
    void nextnl() {
        const Token t = next();
        if(t.type != Token::NONE && t.type != Token::NEWLINE)
            throw LexerError("Expected newline");
    }

public:
    void loadf(std::string &file) {
        std::ifstream f(file);
        this->f << f.rdbuf();
    };
    void loads(std::string &s) {
        f.str(s);
        f.clear();
        ended = false;
    };
    void parse();

// Errors
class LexerError : public std::exception {
private:
    std::string s;
public:
    template<typename First, typename ...Rest>
    LexerError(First && first, Rest && ...rest) {
        std::stringstream ss;
        ss << std::forward<First>(first);
        using expander = int[];
        (void)expander{0, (void(ss << std::forward<Rest>(rest)), 0)...};
        this->s = ss.str();
    };
    const char* what() const throw() { return s.data(); };
};
class ParserError : public std::exception {
private:
    std::string s;
public:
    template<typename First, typename ...Rest>
    ParserError(First && first, Rest && ...rest) {
        std::stringstream ss;
        ss << std::forward<First>(first);
        using expander = int[];
        (void)expander{0, (void(ss << std::forward<Rest>(rest)), 0)...};
        this->s = ss.str();
    };
    const char* what() const throw() { return s.data(); };
};

};


}
