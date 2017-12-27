include ../colors.mk

build: src/bindings.rs
	$(call build_command,cargo build)
	@$(MAKE) -wC c
	@$(MAKE) -wC python

src/bindings.rs: *.py
	$(call build_command,python3 generate.py)

test: build
	@$(MAKE) -wC c test
	@$(MAKE) -wC python test

.PHONY: build test