rem Building on windows requires the following pieces of software:
rem rust 1.22.0
rem visual studio 2017 CE
rem jdk 8
rem python 3.6 64-bit
rem    pip setuptools cffi nose

cd bindings
python generate.py
if %errorlevel% neq 0 exit /b %errorlevel%

rem cargo build
@if %errorlevel% neq 0 exit /b %errorlevel%

cd python
python setup.py build_ext --inplace
@if %errorlevel% neq 0 exit /b %errorlevel%

rem cd ../java/src/bc
rem swig -java -package bc -outcurrentdir ../../../c/include/bc.i
rem @if %errorlevel% neq 0 exit /b %errorlevel%
rem 
rem cd ../..
rem javac src/bc/*.java
rem @if %errorlevel% neq 0 exit /b %errorlevel%
rem 
rem nmake /f windows.mk
rem 
cd ../..

setlocal enableextensions
rmdir /s/q battlecode
mkdir battlecode\python
mkdir battlecode\java
mkdir battlecode\c\include
mkdir battlecode\c\lib
xcopy /s/e bindings\python\battlecode battlecode\python\battlecode\
xcopy /s/e bindings\java\src\bc battlecode\java\bc\
copy bindings\c\include\bc.h battlecode\c\include\bc.h
copy target\debug\deps\battlecode.lib battlecode\c\lib\libbattlecode-win32.lib
endlocal

echo "created folder battlecode"
