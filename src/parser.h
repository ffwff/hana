#pragma once
#include <fstream>
#include <cassert>
#include <stack>
#include <exception>
#include "error.h"

namespace Hana {

class Parser {
private:

    struct Position {
        size_t pos, lines;
#ifndef RELEASE
        std::string caller;
#endif
    };
    std::string s;

protected:

    size_t pos;
    std::stack<Position> fposs;
    bool ended = false;

    // Get/unget
    char getc() { return pos >= s.size() ? EOF : s[pos++]; }
    char peek() const { return pos >= s.size() ? EOF : s[pos]; }
    void unget() { pos--; }
    bool eof() { return pos >= s.size(); }
    size_t tellg() const { return pos; }
    void seekg(size_t s) { pos = s; }

    // File position
#ifndef RELEASE
    void fsave_(std::string caller) {
        fposs.push({
            .pos = pos,
            .lines = lines,
            .caller = caller
        });
    }
    void fpop_() {
        fposs.pop();
    }
#else
    void fsave() {
        fposs.push({
            .pos = pos,
            .lines = lines
        });
    }
    void fpop() {
        fposs.pop();
    }
#endif
    void floadn() {
        const auto p = fposs.top();
        pos = p.pos;
        lines = p.lines;
    }
    void fload() {
        const auto p = fposs.top();
        pos = p.pos;
        lines = p.lines;
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
            throw LexerError("Expected newline, got " + t.strv);
    }

public:
    void loadf(std::string &file) {
        std::ifstream f(file);
        f.seekg(0, std::ios::end);
        size_t size = f.tellg();
        s = std::string(size, '\0');
        f.seekg(0);
        f.read(&s[0], size);
    };
    void loads(std::string &s) {
        this->s = s;
        ended = false;
        pos = 0;
    };
    void parse();
    size_t lines = 0;

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
