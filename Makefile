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
	-rm -rf docker-artifacts
	# run build first, to generate code and stuff

generate:
	@$(MAKE) -wC bindings generate

docker-sandbox:
	docker build -t battlebaby -f SandboxDockerfile .
	mkdir -p docker-artifacts/
	docker save battlebaby -o docker-artifacts/battlebaby.tar
	ID=$$(docker create battlebaby);\
	   docker cp $$ID:/usr/lib/python3.6/site-packages/UNKNOWN-0.0.0-py3.6-linux-x86_64.egg/battlecode docker-artifacts/battlecode;\
       docker rm -v $$ID

create-bundle:
	-rm -rf bundle
	mkdir -p bundle
	cp -R bindings/python/battlecode bundle
	cp -R bindings/java/src/bc bundle
	cp -R bindings/c/include/bc.h bundle
	cp -R target/debug/deps/libbattlecode.a bundle

nodocker: build create-bundle

docker-manager:
	docker build -t battledaddy -f ManagerDockerfile .

dockers: docker-py3 docker-java

.PHONY: build test dockers
