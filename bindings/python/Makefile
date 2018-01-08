include ../../colors.mk

build: 
	$(call build_command,cargo build)
	$(call build_command,python setup.py build_ext --inplace)
	$(call build_command,python3 setup.py build_ext --inplace)

test: build
	$(call test_command,python -m nose)

clean:
	-rm -rf battlecode/*.so build 

.PHONY: build test clean