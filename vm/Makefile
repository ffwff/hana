# Program
SRC = $(wildcard src/*.c)
OBJS = ${subst src/,build/,$(SRC:.c=.o)}
CCFLAGS += -g -Wall
DEPS = $(OBJS:.o=.d)
-include $(DEPS)
LDDFLAGS =

main: build/main.o $(OBJS)
	$(CC) -o $@ $^

build:
	mkdir -p build

build/%.o: src/%.c build
	$(CC) -c -o $@ $< $(CCFLAGS) -MMD

clean:
	rm -rf $(OBJS) $(DEPS)
