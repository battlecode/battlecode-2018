# Makefile for using SWIG and Java for C code

WRAPFILE      = src\bc\bc_wrap.c

# Location of the Visual C++ tools (32 bit assumed)

TARGET        = example.dll
CC            = cl.exe
LINK          = link.exe
#MACHINE       = IX86

# Windows libraries that are apparently needed
LIBS        = kernel32.lib advapi32.lib user32.lib gdi32.lib comdlg32.lib \
			  winspool.lib dbghelp.lib ws2_32.lib userenv.lib shell32.lib msvcrt.lib oldnames.lib 

# Linker options
LOPT      = -debug:full -debugtype:cv /NODEFAULTLIB /RELEASE /NOLOGO \
              -entry:_DllMainCRTStartup@12 -dll
# /MACHINE:$(MACHINE)

# C compiler flags

CFLAGS        = /Z7 /Od /c /nologo

build::
	$(CC) $(CFLAGS) $(JAVA_INCLUDE) $(SRCS) $(WRAPFILE)
	$(LINK) $(LOPT) -out:src\bc\libbattlecode-java-win32.dll $(LIBS) src\bc\bc_wrap.obj ..\..\target\debug\battlecode.lib

release::
	$(CC) $(CFLAGS) $(JAVA_INCLUDE) $(SRCS) $(WRAPFILE)
	$(LINK) $(LOPT) -out:src\bc\libbattlecode-java-win32.dll $(LIBS) src\bc\bc_wrap.obj ..\..\target\release\battlecode.lib