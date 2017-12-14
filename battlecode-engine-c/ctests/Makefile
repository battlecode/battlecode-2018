# Makefile to build C code.
CC = gcc

default: test_all
	@echo
	@echo

test_all: c_test_run

c_test: c_test.c cargo
	@echo
	@echo
	@echo "-- running C tests --"
	$(CC) c_test.c -o c_test -I../include/ -I. -L../../target/debug -lbattlecode

.PHONY: c_test_run
c_test_run: c_test
	./c_test


.PHONY: cargo
cargo:
	cargo build