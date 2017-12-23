define build_command
	@echo
	$(call magenta,$$ $(1))
	@$(1)
	$(call green,succeeded)
	@echo
endef

define test_command
	@echo
	$(call magenta,$$ $(1))
	@$(1)
	$(call green,passed)
	@echo
endef

define magenta
	@echo "\x1b[35m$(1)\x1b[0m"
endef

define green
	@echo "\x1b[32m$(1)\x1b[0m"
endef
