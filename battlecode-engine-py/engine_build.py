# in a separate file "package/foo_build.py"
import cffi
import re

with open('../battlecode-engine-c/include/battlecode.h', 'r') as f:
    contents = f.read()

# sanitize contents for cffi, which expects input without #includes and such
# a hack, but it's less code to maintain
stripped = re.sub(r"#ifdef __cplusplus\n[^\n]*\n#endif", '', contents, flags=re.MULTILINE)
stripped = '\n'.join(line for line in stripped.splitlines() if '#' not in line)

import os.path

ffibuilder = cffi.FFI()
ffibuilder.cdef(stripped)
ffibuilder.set_source(
    "battlecode._engine",
    contents,
    library_dirs=['../target/debug/deps'],
    libraries=['System','resolv','c','m'],
    extra_compile_args=['-arch','x86_64'],
    extra_link_args=['-arch','x86_64', '../target/debug/deps/libbattlecode.a']
    )

if __name__ == "__main__":
    ffibuilder.compile(verbose=True)