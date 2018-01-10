include helpers.mk

ifeq ($(CUR_OS),darwin)
	LIB_TARGET = battlecode/c/lib/libbattlecode-darwin.a
endif
ifeq ($(CUR_OS),linux)
	LIB_TARGET = battlecode/c/lib/libbattlecode-linux.a
endif

build: battlecode
	@$(MAKE) -wC bindings
	cp -R bindings/python/battlecode battlecode/python/battlecode
	cp -R bindings/java/src/bc battlecode/java/bc
	cp -R bindings/c/include battlecode/c/include
	cp -R target/debug/deps/libbattlecode.a $(LIB_TARGET)

release: battlecode
	@$(MAKE) -wC bindings release

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
	# run build first, to generate code and stuff

generate:
	@$(MAKE) -wC bindings generate

docker-sandbox:
	docker build -t battlebaby -f SandboxDockerfile .
	mkdir -p docker-artifacts/
	docker save battlebaby -o docker-artifacts/battlebaby.tar
	ID=$$(docker create battlebaby);\
	   docker cp $$ID:/battlecode/battlecode docker-artifacts/linux-battlecode;\
       docker rm -v $$ID

nodocker: build create-bundle

docker-manager:
	docker build -t battledaddy -f ManagerDockerfile .

dockers: docker-py3 docker-java

bc18-scaffold:
	git clone https://github.com/battlecode/bc18-scaffold

package:
	# TODO: combine different operating system build artifacts
	# TODO: edit manager 
	-rm -rf battlecode-manager/working_dir 
	cp -R battlecode bc18-scaffold/battlecode
	cp -R battlecode-manager bc18-scaffold/battlecode-manager
	cp -R examplefuncsplayer-python bc18-scaffold/examplefuncsplayer-python
	cp -R examplefuncsplayer-c bc18-scaffold/examplefuncsplayer-c
	cp -R examplefuncsplayer-java bc18-scaffold/examplefuncsplayer-java
	cp run_nodocker.sh bc18-scaffold/

.PHONY: build test dockers battlecode
