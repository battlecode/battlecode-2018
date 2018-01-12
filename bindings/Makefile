include ../helpers.mk

build:
	$(call build_command,python3.6 generate.py)
	$(call build_command,cargo build)
	@$(MAKE) -wC c
	@$(MAKE) -wC python
	@$(MAKE) -wC java

release:
	$(call build_command,python3.6 generate.py)
	$(call build_command,cargo build --release)
	@$(MAKE) -wC c release
	@$(MAKE) -wC python release
	@$(MAKE) -wC java release

test: build
	@$(MAKE) -wC c test
	@$(MAKE) -wC python test
	@$(MAKE) -wC java test

# extra command, not useful:
generate:
	$(call build_command,python3.6 generate.py)

clean:
	-rm src/bindings.rs
	@$(MAKE) -wC python clean
	@$(MAKE) -wC java clean

.PHONY: build test clean generate