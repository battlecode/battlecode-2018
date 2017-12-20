from helpers import *
from struct import StructWrapper

class CEnum(object):
    '''A c-style enum.'''
    def __init__(self, module, name):
        self.name = name
        self.c_name = f'{module}_{name}'
        self.variants = []
    
    def variant(self, name, value):
        self.variants.append((name, value))

    def to_rust(self):
        start = f'#[repr(C)]\nenum {self.c_name} {{\n'
        internal = '\n'.join(f'{name} = {val},' for (name, val) in self.variants)
        end = '\n}\n'

        return start + s(internal, indent=4) + end

    def to_c(self):
        start = f'typedef enum {self.c_name} {{\n'
        internal = '\n'.join(f'{name} = {val},' for (name, val) in self.variants)
        end = f'\n}} {self.name};\n'

        return start + s(internal, indent=4) + end

    def to_swig(self):
        start = f'%javaconst(1);\ntypedef enum {self.c_name} {{\n'
        internal = '\n'.join(f'{name} = {val},' for (name, val) in self.variants)
        end = f'\n}} {self.name};\n'

        return start + s(internal, indent=4) + end

    def to_python(self):
        start = f'class {self.name}:\n'
        internal = '\n'.join(f'{name} = {val}' for (name, val) in self.variants)
        return start + s(internal, indent=4)


class EnumWrapper(StructWrapper):
    def __init__(self, module, name, docs=''):
        super().__init__(module, name, docs)
        self.cenum = CEnum(module, name)

    def variant(self, name, attributes):
        pass
