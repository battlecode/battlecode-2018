include colors.mk

build:
	@$(MAKE) -wC bindings
	$(call build_command,cargo build)

test:
	@$(MAKE) -wC bindings test
	$(call test_command,cargo test)

package:
	mkdir -p package

clean:
	@$(MAKE) -wC bindings clean
	-rm -rf package

.PHONY: build test