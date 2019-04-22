default: main
.PHONY: main.cpp

# Program
PARSER_SRC = $(wildcard src/*.cpp)
VM_SRC = $(wildcard vm/src/*.c)
HANAYO_SRC = $(wildcard hanayo/native/*.cpp)
OBJS = ${subst src/,build/,$(PARSER_SRC:.cpp=.o)} \
       ${subst vm/src/,build/vm/,$(VM_SRC:.c=.o)} \
       ${subst hanayo/native,build/hanayo/,$(HANAYO_SRC:.cpp=.o)}
DEPS = $(OBJS:.o=.d)
-include $(DEPS)

CXXFLAGS += -Wno-format-truncation

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
CCFLAGS += -Wall -Ivm/src -Iextern/xxHash
#LDDFLAGS += -s

# bytecode
ADDITIONAL=
ifdef INCLUDE_BYTECODE
CXXFLAGS += -Iextern/incbin -DINCLUDE_BYTECODE
ADDITIONAL += build/init.bin
endif

main: build/main.o $(OBJS) $(ADDITIONAL)
	$(CXX) $(LDDFLAGS) -o $@ build/main.o $(OBJS)
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
build/hanayo:
	mkdir -p build/hanayo
build/hanayo/%.o: hanayo/native/%.cpp build/hanayo
	$(CXX) -c -o $@ $< $(CXXFLAGS) -MMD -fno-rtti -fno-exceptions -nostdinc++
build/init.bin: hanayo/interpreted/*.hana ./main
	(cpp hanayo/interpreted/init.hana | sed "s/^#.*//g") >build/init.hana
	./main -d build/init.hana >$@

clean:
	rm -rf libhana.so $(OBJS) $(DEPS)
