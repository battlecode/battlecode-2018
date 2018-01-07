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
elif sys.platform.startswith('linux'):
    libraries = ['util','dl','rt','pthread','gcc_s','c','m']
else:
    raise Exception('What libraries should I link with on '+sys.platform+'?')

if 'RELEASE' in os.environ:
    library_dirs=['../../target/release/deps']
    extra_link_args=['../../target/release/deps/libbattlecode.a']
else:
    library_dirs=['../../target/debug/deps']
    extra_link_args=['../../target/debug/deps/libbattlecode.a']

ffibuilder = cffi.FFI()
ffibuilder.cdef(stripped)
ffibuilder.set_source(
    'battlecode._bc',
    contents,
    library_dirs=library_dirs,
    libraries=libraries,
    extra_link_args=extra_link_args
)

if __name__ == '__main__':
    ffibuilder.compile(verbose=True)