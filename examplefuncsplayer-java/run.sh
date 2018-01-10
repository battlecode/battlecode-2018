#!/bin/sh
echo javac $(find . -name '*.java') -classpath ../battlecode/java
javac $(find . -name '*.java') -classpath ../battlecode/java
echo java -classpath .:../battlecode/java Player
java -classpath .:../battlecode/java Player