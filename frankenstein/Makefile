include ../colors.mk

build:
	$(call build_command,python3 generate.py)
	@$(MAKE) -wC c
	@$(MAKE) -wC python

test:
	@$(MAKE) -wC c test
	@$(MAKE) -wC python test

.PHONY: build test