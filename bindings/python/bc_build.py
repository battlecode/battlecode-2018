'''Handles building the _engine module, which embeds a statically linked version of the battlecode engine
into python.'''

import cffi
import re
import os
import os.path
import sys

with open('../c/include/bc.h', 'r') as f:
    contents = f.read()

# sanitize contents for cffi, which expects input without #includes and such
# a hack, but it's less code to maintain
stripped = re.sub(r'#ifdef __cplusplus\n[^\n]*\n#endif', '', contents, flags=re.MULTILINE)
stripped = '\n'.join(line for line in stripped.splitlines() if '#' not in line)

# libraries that rustc spits out when you compile a static library.
if sys.platform == 'darwin':
    libraries = ['System','resolv','c','m']
    source = 'battlecode.darwin._bc'
elif sys.platform.startswith('linux'):
    libraries = ['util','dl','rt','pthread','gcc_s','c','m']
    source = 'battlecode.linux._bc'
elif sys.platform == 'win32':
    libraries = ["kernel32", "advapi32", "dbghelp", "advapi32", "advapi32", "ws2_32", "userenv", "shell32", "msvcrt"]
    source = 'battlecode.win32._bc'
else:
    raise Exception("I don't understand"+sys.platform+'.')

if 'CARGO_TARGET_DIR' in os.environ:
    CTD = os.environ['CARGO_TARGET_DIR']
else:
    CTD = os.path.join('..','..','target')

if 'RELEASE' in os.environ:
    KIND = 'release'
else:
    KIND = 'debug'

if sys.platform == 'darwin' or sys.platform.startswith('linux'):
    ARTIFACT = 'libbattlecode.a'
else:
    ARTIFACT = 'battlecode.lib'

extra_link_args = [os.path.join(CTD, KIND, 'deps', ARTIFACT)]

ffibuilder = cffi.FFI()
ffibuilder.cdef(stripped)
ffibuilder.set_source(
    source,
    contents,
    libraries=libraries,
    extra_link_args=extra_link_args,
    depends=extra_link_args
)

if __name__ == '__main__':
    ffibuilder.compile(verbose=True)
