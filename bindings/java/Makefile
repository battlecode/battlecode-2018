include ../../helpers.mk

ifeq ($(CUR_OS),linux)
    JAVAINCLUDE=-I /usr/lib/jvm/java-8-openjdk-amd64/include -I /usr/lib/jvm/java-8-openjdk-amd64/include/linux
    LIBS=-lutil -ldl -Lrt -lpthread -lgcc_s -lc -lm
    TARGET=src/bc/libbattlecode-java-linux.so
endif
ifeq ($(CUR_OS),darwin)
    JAVAINCLUDE=-I /System/Library/Frameworks/JavaVM.framework/Versions/Current/Headers/
    LIBS=-lSystem -lresolv -lc -lm
    TARGET=src/bc/libbattlecode-java-darwin.so
endif

build: .build-marker
.build-marker: ../c/include/bc.i ../../target/debug/deps/libbattlecode.a src/bc/bc_wrap.o src/bc/bcJNI.class
	$(call build_command,gcc -shared -O0 -fPIC src/bc/bc_wrap.o ../../target/debug/deps/libbattlecode.a $(LIBS) -o $(TARGET))
	$(call build_command,touch .build-marker && rm .release-marker 2&>/dev/null || true)

test: build
	javac tests/*.java -classpath ./tests:./src
	java -classpath ./tests:./src Test

release: .release-marker
.release-marker: ../c/include/bc.i ../../target/release/deps/libbattlecode.a src/bc/bc_wrap.o src/bc/bcJNI.class
	$(call build_command,gcc -shared -O3 -fPIC src/bc/bc_wrap.o ../../target/release/deps/libbattlecode.a $(LIBS) -o $(TARGET))
	$(call build_command,touch .release-marker && rm .build-marker 2&>/dev/null || true)

src/bc/bc_wrap.o: src/bc/bc_wrap.c
	$(call build_command,gcc -c -O3 -fPIC src/bc/bc_wrap.c $(JAVAINCLUDE) -I../c/include/ -o src/bc/bc_wrap.o)

src/bc/bcJNI.class: src/bc/bcJNI.java
	$(call build_command,javac src/bc/*.java)

src/bc/bcJNI.java src/bc/bc_wrap.c: ../c/include/bc.i
	$(call build_command,cd src/bc && swig -java -package bc -outcurrentdir ../../../c/include/bc.i)

clean:
	-rm -rf src/bc/*.class
	-rm -rf src/bc/*.java
	-rm -rf src/bc/*.so
	-rm .build-marker .release-marker 2&>/dev/null
	
.PHONY: build test clean
