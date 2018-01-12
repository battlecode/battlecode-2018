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
	@-tput setaf 5
	@echo "$(1)"
	@-tput sgr0
endef

define green
	@-tput setaf 2
	@echo "$(1)"
	@-tput sgr0
endef

ifeq ($(OS),Windows_NT)
	CUR_OS := win32
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        CUR_OS := linux
    endif
    ifeq ($(UNAME_S),Darwin)
		CUR_OS := darwin
    endif
endif