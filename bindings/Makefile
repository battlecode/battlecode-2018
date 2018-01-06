include ../colors.mk

build: src/bindings.rs
	$(call build_command,cargo build)
	@$(MAKE) -wC c
	@$(MAKE) -wC python

src/bindings.rs: *.py frankenswig/*.py
	$(call build_command,python3 generate.py)

test: build
	@$(MAKE) -wC c test
	@$(MAKE) -wC python test

clean:
	-rm src/bindings.rs

.PHONY: build test clean