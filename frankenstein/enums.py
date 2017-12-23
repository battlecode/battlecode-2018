from helpers import *
from type import Type
from struct import StructWrapper

class CEnum(object):
    '''A c-style enum.'''
    def __init__(self, module, name):
        self.name = name
        self.c_name = f'{module}_{name}'
        self.variants = []
    
    def variant(self, name, value):
        self.variants.append((name, value))
        return self

    def to_rust(self):
        start = f'#[repr(C)]\npub enum {self.c_name} {{\n'
        internal = '\n'.join(f'{name} = {val},' for (name, val) in self.variants)
        end = '\n}\n'

        return start + s(internal, indent=4) + end

    def to_c(self):
        start = f'typedef enum {self.c_name} {{\n'
        internal = '\n'.join(f'{name} = {val},' for (name, val) in self.variants)
        end = f'\n}} {self.c_name};\n'

        return start + s(internal, indent=4) + end

    def to_swig(self):
        start = f'%javaconst(1);\ntypedef enum {self.c_name} {{\n'
        internal = '\n'.join(f'{name} = {val},' for (name, val) in self.variants)
        end = f'\n}} {self.c_name};\n'

        return start + s(internal, indent=4) + end

    def to_python(self):
        start = f'class {self.name}(enum.IntEnum):\n'
        internal = '\n'.join(f'{name} = {val}' for (name, val) in self.variants)
        return start + s(internal, indent=4) + '\n'

class CEnumWrapperType(Type):
    def __init__(self, module, rust_name):
        self.san_name = sanitize_rust_name(rust_name)
        self.c_name = f'{module}_{self.san_name}'
        self.orig_name = f'{module}::{rust_name}'
        super().__init__(self.c_name, self.c_name, self.san_name, default=None)
    
    def wrap_c_value(self, name):
        return ('', f'{name}.into()', '')

    def unwrap_rust_value(self, name):
        return f'{name}.into()'

class CEnumWrapper(CEnum):
    '''A wrapper for a rust c-style enum, that is, an enum with integer values.'''

    def __init__(self, module, rust_name, docs=''):
        self.type = CEnumWrapperType(module, rust_name)
        super().__init__(module, self.type.san_name)
        self.docs = docs
    
    def variant(self, name, value):
        super().variant(name, value)
        # this is.. unfortunate, but necessary due to builder pattern
        if self.type.default is None:
            self.type.default = f'{self.c_name}::{name}'
        return self
    
    def to_rust(self):
        decl = super().to_rust()

        for t1, t2 in [(self.type.c_name, self.type.orig_name), (self.type.orig_name, self.type.c_name)]:
            start = s(f'''\
            impl Into<{t2}> for {t1} {{
                fn into(self) -> {t2} {{
                    match self {{
            ''')

            body = ''
            for variant,_ in self.variants:
                body += f'{t1}::{variant} => {t2}::{variant},\n'
            body += f'_ => {t2}::{self.variants[0][0]},\n'


            end = s(f'''\
                    }}
                }}
            }}
            ''')

            decl += start + s(body, indent=12) + end

        return decl

class EnumWrapper(StructWrapper):
    def __init__(self, module, name, docs=''):
        super().__init__(module, name, docs)
        self.cenum = CEnum(module, name)

    def variant(self, name, attributes):
        pass
