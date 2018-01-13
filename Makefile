include helpers.mk

ifeq ($(CUR_OS),darwin)
	LIB_TARGET = battlecode/c/lib/libbattlecode-darwin.a
endif
ifeq ($(CUR_OS),linux)
	LIB_TARGET = battlecode/c/lib/libbattlecode-linux.a
endif

build: battlecode
	@$(MAKE) -wC bindings
	cp -R target/debug/deps/libbattlecode.a $(LIB_TARGET)
	@$(MAKE) copy

release: battlecode
	@$(MAKE) -wC bindings release
	cp -R target/release/deps/libbattlecode.a $(LIB_TARGET)
	@$(MAKE) copy

copy:
	cp -R bindings/python/battlecode battlecode/python/battlecode
	cp -R bindings/java/src/bc battlecode/java/bc
	cp -R bindings/c/include battlecode/c/include

copy-linux:
	cp docker-artifacts/linux-battlecode/python/battlecode/linux/* battlecode/python/battlecode/linux/
	cp docker-artifacts/linux-battlecode/java/bc/*linux* battlecode/java/bc/
	cp docker-artifacts/linux-battlecode/c/lib/*linux* battlecode/c/lib/

copy-win32:
	cp win32-battlecode/python/battlecode/win32/* battlecode/python/battlecode/win32/

battlecode:
	rm -rf battlecode
	mkdir -p battlecode/python/
	mkdir -p battlecode/c/lib
	mkdir -p battlecode/java/

test:
	@$(MAKE) -wC bindings test
	$(call test_command,cargo test)

clean:
	@$(MAKE) -wC bindings clean
	-rm -rf docker-artifacts
	-rm -rf docker-manager/working_dir
	rm -rf battlecode
	# run build first, to generate code and stuff

generate:
	@$(MAKE) -wC bindings generate

linux-libs:
	docker build -t linuxbuild -f LinuxBuildDockerfile .
	mkdir -p docker-artifacts/
	ID=$$(docker create linuxbuild);\
	   docker cp $$ID:/battlecode docker-artifacts/linux-battlecode;\
       docker rm -v $$ID

docker-sandbox:
	docker build -t battlebaby -f SandboxDockerfile . --squash
	mkdir -p docker-artifacts/
	docker save battlebaby -o docker-artifacts/battlebaby.tar
	ID=$$(docker create battlebaby);\
	   docker cp $$ID:/battlecode docker-artifacts/linux-battlecode-musl;\
       docker rm -v $$ID

nodocker: build create-bundle

docker-manager:
	#???
	-rm -rf docker-artifacts/linux-battlecode-musl/battlecode
	docker build -t battledaddy -f ManagerDockerfile .

dockers: docker-py3 docker-java

bc18-scaffold:
	git clone https://github.com/battlecode/bc18-scaffold

package:
	-rm -rf battlecode-manager/working_dir 
	-rm -rf battlecode-manager/__pycache__ 
	-rm -rf examplefuncsplayer-python/__pycache__ 
	-rm -rf examplefuncsplayer-java/*class
	-rm -rf bc18-scaffold/battlecode
	-rm -rf bc18-scaffold/battlecode-manager
	-rm -rf bc18-scaffold/battlecode-maps
	-rm -rf bc18-scaffold/examplefuncsplayer-python
	-rm -rf bc18-scaffold/examplefuncsplayer-c
	-rm -rf bc18-scaffold/examplefuncsplayer-java
	cp -R battlecode bc18-scaffold/battlecode
	cp -R battlecode-manager bc18-scaffold/battlecode-manager
	cp -R battlecode-maps bc18-scaffold/battlecode-maps
	cp -R examplefuncsplayer-python bc18-scaffold/examplefuncsplayer-python
	cp -R examplefuncsplayer-c bc18-scaffold/examplefuncsplayer-c
	cp -R examplefuncsplayer-java bc18-scaffold/examplefuncsplayer-java
	cp run_nodocker.sh bc18-scaffold/
	cp run_nodocker.bat bc18-scaffold/
	cp battlecode.sh bc18-scaffold/
	cp battlecode.bat bc18-scaffold/
	cp run.sh bc18-scaffold/
	cp run.bat bc18-scaffold/

.PHONY: build test dockers battlecode
