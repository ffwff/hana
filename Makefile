default: libhana_setup libhana.so
libhana_setup:
	$(eval CXXFLAGS := $(CXXFLAGS) -fPIC)
.PHONY: main.cpp

# Program
SRC = $(wildcard src/*.cpp)
OBJS = ${subst src/,build/,$(SRC:.cpp=.o)}
CXXFLAGS += -Wall -std=c++17
DEPS = $(OBJS:.o=.d)
-include $(DEPS)
LDDFLAGS =

# Version
ifdef RELEASE
CXXFLAGS += -O3 -DRELEASE
else
CXXFLAGS += -g -Wfatal-errors -DDEBUG
endif

main: build/main.o $(OBJS)
	$(CXX) -o $@ $^
build/main.o: main.cpp build
	$(CXX) -c -o $@ $< $(CXXFLAGS)

libhana.so: $(OBJS)
	$(CXX) -shared -o $@ $^

build:
	mkdir -p build

build/%.o: src/%.cpp build
	$(CXX) -c -o $@ $< $(CXXFLAGS) -MMD

clean:
	rm -rf libhana.so $(OBJS) $(DEPS)
