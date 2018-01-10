include ../../helpers.mk

build: 
	$(call build_command,python3 setup.py build_ext --inplace)

release:
	$(call build_command,RELEASE=1 python3 setup.py build_ext --inplace)

test: build
	$(call test_command,python3 -m nose)

clean:
	-rm -rf battlecode/*.so build 

.PHONY: build test clean