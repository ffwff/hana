#pragma once
#include <iostream>
#include <cassert>

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
#ifdef NOLOG
#define LOG(...)
#else
#define LOG(...) _LOG(__AT, __FUNCTION__," :  ", __VA_ARGS__)
#endif
#else
#define LOG(...)
#endif
