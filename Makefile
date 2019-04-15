default: main
.PHONY: main.cpp

# Program
PARSER_SRC = $(wildcard src/*.cpp)
VM_SRC = $(wildcard vm/src/*.c)
OBJS = ${subst src/,build/,$(PARSER_SRC:.cpp=.o)} \
       ${subst vm/src/,build/vm/,$(VM_SRC:.c=.o)}
DEPS = $(OBJS:.o=.d)
-include $(DEPS)

# Version
ifdef RELEASE
CXXFLAGS += -O3 -DRELEASE
CCFLAGS += -O3 -DNOLOG
else
CXXFLAGS += -g -DDEBUG
CCFLAGS += -g
endif
ifdef PROFILE
CXXFLAGS += -O3 -DRELEASE -DNOLOG -g -pg
CCFLAGS += -O3 -DNOLOG -g -pg
LDDFLAGS += -O3 -g -pg
endif

# Logging
ifdef NOLOG
CXXFLAGS += -DNOLOG
CCFLAGS += -DNOLOG
endif
ifdef READLINE
CXXFLAGS += -DLREADLINE
LDDFLAGS += -lreadline
endif

# Default flags
CXXFLAGS += -std=c++11 -I. -Wall
CCFLAGS += -Wall -Ivm/src -Ivm/xxHash

main: build/main.o $(OBJS)
	$(CXX) $(LDDFLAGS) -o $@ $^
build/main.o: main.cpp build
	$(CXX) -c -o $@ $< $(CXXFLAGS)

libhana.so: $(OBJS)
	$(CXX) -shared -o $@ $^

build:
	mkdir -p build
build/%.o: src/%.cpp build
	$(CXX) -c -o $@ $< $(CXXFLAGS) -MMD
build/vm:
	mkdir -p build/vm
build/vm/%.o: vm/src/%.c build/vm
	$(CC) -c -o $@ $< $(CCFLAGS) -MMD

clean:
	rm -rf libhana.so $(OBJS) $(DEPS)
