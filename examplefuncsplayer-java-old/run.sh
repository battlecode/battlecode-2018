#!/bin/sh
# build the java files.
# there will eventually be a separate build step, but for now the build counts against your time.
javac *.java -classpath /battlecode-java:.
java -classpath /battlecode-java:. Player