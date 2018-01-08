include ../../colors.mk

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
	JAVAINCLUDE=/usr/java/include
	LIBS=-lutil -ldl -Lrt -lpthread -lgcc_s -lc -lm
endif
ifeq ($(UNAME_S),Darwin)
	JAVAINCLUDE=/System/Library/Frameworks/JavaVM.framework/Versions/Current/Headers/
	LIBS=-lSystem -lresolv -lc -lm
endif

build: src/bc/libbattlecode.so src/bc/bc.class

src/bc/bc.class: src/bc/bc.java
	$(call build_command,javac src/bc/*.java)

src/bc/libbattlecode.so: src/bc/bc_wrap.o ../../target/debug/deps/libbattlecode.a
	$(call build_command,clang -shared src/bc/bc_wrap.o ../../target/debug/deps/libbattlecode.a $(LIBS) -o src/bc/libbattlecode.so)

../../target/debug/deps/libbattlecode.a:
	$(call build_command,cargo build)

src/bc/bc_wrap.o: src/bc/bc_wrap.c
	$(call build_command,clang -c src/bc/bc_wrap.c -I$(JAVAINCLUDE) -I../c/include/ -o src/bc/bc_wrap.o)

src/bc/bc_wrap.c src/bc/bc.java: ../c/include/bc.i
	$(call build_command,cd src/bc && swig -java -package bc -outcurrentdir ../../../c/include/bc.i)

test: build
	javac tests/*.java -classpath ./tests:./src
	java -classpath ./tests:./src -Djava.library.path=/Users/james/Library/Java/Extensions:/Library/Java/Extensions:/Network/Library/Java/Extensions:/System/Library/Java/Extensions:/usr/lib/java:.:/Users/james/Dev/battlecode-2018/bindings/java/src/bc Test

clean:
	-rm -rf src/bc/*
	
.PHONY: build test clean