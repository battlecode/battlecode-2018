include ../colors.mk

build:
	$(call build_command,python3 generate.py)
	$(call build_command,cargo build)
	@$(MAKE) -wC c
	@$(MAKE) -wC python

test: build
	@$(MAKE) -wC c test
	@$(MAKE) -wC python test

.PHONY: build test