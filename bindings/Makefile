include ../helpers.mk

build: src/bindings.rs
	$(call build_command,cargo build)
	@$(MAKE) -wC c
	@$(MAKE) -wC python
	@$(MAKE) -wC java

src/bindings.rs: *.py frankenswig/*.py
	$(call build_command,python3.6 generate.py)

generate: src/bindings.rs

test: build
	@$(MAKE) -wC c test
	@$(MAKE) -wC python test
	@$(MAKE) -wC java test

clean:
	-rm src/bindings.rs
	@$(MAKE) -wC python clean
	@$(MAKE) -wC java clean

.PHONY: build test clean generate