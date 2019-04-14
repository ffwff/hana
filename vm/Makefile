# Program
SRC = $(wildcard src/*.c)
OBJS = ${subst src/,build/,$(SRC:.c=.o)}
CCFLAGS += -g -Wall -Isrc -IxxHash
DEPS = $(OBJS:.o=.d)
-include $(DEPS)
LDDFLAGS =

main: build/main.o $(OBJS)
	$(CC) -o $@ $^
tests: build/tests.o $(OBJS)
	$(CC) -o $@ $^

build/main.o: main.c
	$(CC) -c -o $@ $< $(CCFLAGS) -MMD
build/tests.o: tests.c
	$(CC) -c -o $@ $< $(CCFLAGS) -MMD
build/%.o: src/%.c
	$(CC) -c -o $@ $< $(CCFLAGS) -MMD

clean:
	rm -rf $(OBJS) $(DEPS)
