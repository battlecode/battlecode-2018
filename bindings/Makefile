include ../helpers.mk

build: c/include/bc.i
	$(call build_command,cargo build)
	@$(MAKE) -wC c
	@$(MAKE) -wC python
	@$(MAKE) -wC java

release: c/include/bc.i
	$(call build_command,cargo build --release)
	@$(MAKE) -wC c release
	@$(MAKE) -wC python release
	@$(MAKE) -wC java release

# signifies that generate.py has been run recently
c/include/bc.i: generate.py frankenswig/*.py
	$(call build_command,python3.6 generate.py)

test: build
	@$(MAKE) -wC c test
	@$(MAKE) -wC python test
	@$(MAKE) -wC java test

# extra command, not useful:
generate:
	$(call build_command,python3.6 generate.py)

# we're responsible for cleaning up files that we generate
clean:
	-rm src/bindings.rs
	@$(MAKE) -wC python clean
	@$(MAKE) -wC java clean
	-rm c/include/*
	-rm python/battlecode/__init__.py

.PHONY: build test clean generate
