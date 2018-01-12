# Makefile for using SWIG and Java for C code

WRAPFILE      = src\bc\bc_wrap.c

# Location of the Visual C++ tools (32 bit assumed)

TARGET        = example.dll
CC            = cl.exe
LINK          = link.exe
#MACHINE       = IX86

# Windows libraries that are apparently needed
LIBS        = kernel32.lib advapi32.lib user32.lib gdi32.lib comdlg32.lib winspool.lib dbghelp.lib ws2_32.lib userenv.lib shell32.lib msvcrt.lib oldnames.lib libcpmt.lib

# Linker options
#LOPT      = -debug:full -debugtype:cv /NODEFAULTLIB /RELEASE /NOLOGO -entry:_DllMainCRTStartup@12 -dll
LOPT      = -debug:full -debugtype:cv /RELEASE /NOLOGO -entry:_DllMainCRTStartup@12 -dll
# /MACHINE:$(MACHINE)

# C compiler flags

CFLAGS        = /Z7 /Od /c /nologo

JAVA_INCLUDE = /I "$(JAVA_HOME)\include" /I "$(JAVA_HOME)\include\win32"
MY_INCLUDE = /I "..\c\include"
MY_LIBS = ..\..\target\debug\battlecode.lib


build::
	echo $(LIB)
	$(CC) $(CFLAGS) $(MY_INCLUDE) $(JAVA_INCLUDE) $(SRCS) $(WRAPFILE)
	$(LINK) $(LOPT) -out:src\bc\libbattlecode-java-win32.dll $(LIBS) $(MY_LIBS) bc_wrap.obj 
