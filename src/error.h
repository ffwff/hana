#pragma once
#include <iostream>
#include <cassert>

#if 0
template<typename First, typename ...Rest>
__attribute__((noreturn))
static void _FATAL(First && first, Rest && ...rest)
{
    std::cout << "[FATAL] " << std::forward<First>(first) << ": ";
    using expander = int[];
    (void)expander{0, (void(std::cout << std::forward<Rest>(rest)), 0)...};
    std::cout << "\n";
    assert(0);
}
#endif

template<typename First, typename ...Rest>
static void _LOG(First && first, Rest && ...rest)
{
    std::cerr << "[LOG] " << std::forward<First>(first) << ": ";
    using expander = int[];
    (void)expander{0, (void(std::cerr << std::forward<Rest>(rest)), 0)...};
    std::cerr << "\n";
}
#ifdef DEBUG
#define __STRINGIFY(x) #x
#define __TOSTRING(x) __STRINGIFY(x)
#define __AT __FILE__ ":" __TOSTRING(__LINE__)
//#define FATAL(why, ...) _FATAL(__AT " " why, __VA_ARGS__)
#ifdef NOLOG
#define LOG(...)
#else
#define LOG(...) _LOG(__AT, __VA_ARGS__)
#endif
#else
//#define FATAL(...) _FATAL(__VA_ARGS__)
#define LOG(...)
#endif
