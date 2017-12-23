# Makefile to build C code.
CC = gcc

default: test_all

test_all: c_test_run

c_test: test/c_test.c cargo
	$(CC) test/c_test.c -std=gnu99 -o test/c_test -g -Iinclude/ -L../../target/debug/deps -lbattlecode

.PHONY: c_test_run
c_test_run: c_test
	LD_LIBRARY_PATH=../../target/debug/deps:$(LD_LIBRARY_PATH) test/c_test

.PHONY: clean
clean:
	rm test/*o test/c_test

.PHONY: cargo
cargo:
	cargo build