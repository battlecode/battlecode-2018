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

docker-prebuild:
	docker build -t sev -f SandboxEverythingDockerfile .
	mkdir -p docker-artifacts/python
	mkdir -p docker-artifacts/java
	set -e;\
	ID=$$(docker create sev);\
	docker cp $$ID:/battlecode/target/release/deps/libbattlecode.a docker-artifacts/libbattlecode.a;\
	docker cp $$ID:/battlecode/target/release/libbattlecode.so docker-artifacts/libbattlecode.so;\
	docker cp $$ID:/battlecode/bindings/python/battlecode docker-artifacts/python/battlecode;\
	docker cp $$ID:/battlecode/bindings/java/src docker-artifacts/java/src;\
	docker rm -v $$ID;\

docker-py3:
	docker build -t battlebaby-py3 -f Python3Dockerfile . --squash
	docker save battlebaby-py3 -o docker-artifacts/battlebaby-py3.tar

docker-java:
	cd docker-artifacts/java/src && jar cvf battlecode.jar bc/*.class
	docker build -t battlebaby-java -f JavaDockerfile . --squash
	docker save battlebaby-java -o docker-artifacts/battlebaby-java.tar

dockers: docker-py3 docker-java

.PHONY: build test dockers
