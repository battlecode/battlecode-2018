# Makefile to build C code.
CC = gcc

default: test_all

test_all: c_test_run

c_test: c_test.c cargo
	$(CC) c_test.c -std=gnu99 -o c_test -g -I../include/ -I. -L../../target/debug/deps -lbattlecode

.PHONY: c_test_run
c_test_run: c_test
	LD_LIBRARY_PATH=../../target/debug/deps:$(LD_LIBRARY_PATH) ./c_test


.PHONY: cargo
cargo:
	cargo build