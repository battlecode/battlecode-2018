# Makefile to build C code.
include ../../colors.mk

CC = gcc

default: c_test

test: c_test_run

c_test: test/c_test.c cargo
	$(call build_command,$(CC) test/c_test.c -std=gnu99 \
		-o test/c_test -g -Iinclude/ -L../../target/debug/deps -lbattlecode)

.PHONY: c_test_run
c_test_run: c_test
	$(call test_command,LD_LIBRARY_PATH=../../target/debug/deps:$(LD_LIBRARY_PATH) test/c_test)

.PHONY: clean
clean:
	rm test/*o test/c_test

.PHONY: cargo
cargo:
	cargo build