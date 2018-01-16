include ../../helpers.mk

# we can't trust python's checking-whether-to-build because it doesn't look at the age of library dependences
# so we have to clean each time :/
build:.build-marker
.build-marker: ../c/include/bc.h $(CARGO_TARGET_DIR)/debug/deps/libbattlecode.a
	@# we have to clean first because python doesn't check lib dependencies for changes
	@make clean
	$(call build_command,CFLAGS="-O0" python3 setup.py build_ext --inplace)
	$(call build_command,CFLAGS="-O0" pypy3 setup.py build_ext --inplace)
	$(call build_command,touch .build-marker)

release:.release-marker
.release-marker: ../c/include/bc.h $(CARGO_TARGET_DIR)/release/deps/libbattlecode.a
	@# we have to clean first because python doesn't check lib dependencies for changes
	@make clean
	$(call build_command,RELEASE=1 python3 setup.py build_ext --inplace)
	$(call build_command,RELEASE=1 pypy3 setup.py build_ext --inplace)
	$(call build_command,touch .release-marker)

test: build
	$(call test_command,python3 -m nose)

clean:
	-rm -rf battlecode/*/*.so build
	-rm .build-marker .release-marker 2&>/dev/null

.PHONY: build test clean
