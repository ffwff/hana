#pragma once

namespace Hana {

class Value;
class Environment;
struct Type {
    typedef Type* Value;
    std::function<void(Environment*)> fn;

    Type(std::function<void(Environment*)> fn) : fn(fn) {};
};

}
