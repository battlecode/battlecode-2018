from .helpers import *
from .type import Type
from .function import Function, Method
from .struct import DeriveMixins

class CEnum(object):
    '''A c-style enum.'''
    def __init__(self, module, gen_name):
        self.gen_name = gen_name
        self.c_name = f'{module}_{gen_name}'
        self.variants = []
    
    def variant(self, name, value):
        self.variants.append((name, value))
        return self

    def to_rust(self):
        start = f'#[repr(C)]\n#[derive(Copy, Clone)]\npub enum {self.c_name} {{\n'
        internal = '\n'.join(f'{name} = {val},' for (name, val) in self.variants)
        end = '\n}\n'

        return start + s(internal, indent=4) + end

    def to_c(self):
        start = f'{doxygen(self.docs)}typedef enum {self.c_name} {{\n'
        internal = '\n'.join(f'{name} = {val},' for (name, val) in self.variants)
        end = f'\n}} {self.c_name};\n'

        return start + s(internal, indent=4) + end

    def to_swig(self):
        start = f'#ifdef SWIGJAVA\n%javaconst(1);\n#endif\ntypedef enum {self.c_name} {{\n'
        internal = '\n'.join(f'{name} = {val},' for (name, val) in self.variants)
        end = f'\n}} {self.c_name};\n'

        return start + s(internal, indent=4) + end

    def to_python(self):
        start = f'class {self.gen_name}(enum.IntEnum):\n'
        internal = '\n'.join(f'{name} = {val}' for (name, val) in self.variants)
        return start + s(internal, indent=4) + '\n'

class CEnumWrapperType(Type):
    def __init__(self, wrapper, is_ref=False):
        self.wrapper = wrapper
        self.san_name = sanitize_rust_name(wrapper.name)
        self.c_name = f'{wrapper.program.module}_{self.san_name}'
        self.orig_name = f'{wrapper.program.module}::{wrapper.name}'
        self.is_ref = is_ref
        super().__init__(self.c_name, self.c_name, self.san_name, default=None)
    
    def ref(self):
        return CEnumWrapperType(self.wrapper, is_ref=True)

    @property
    def default(self):
        return f'{self.c_name}::{self.wrapper.variants[0][0]}'

    @default.setter
    def default(self, v):
        pass

    def mut_ref(self):
        return self.ref()
    
    def wrap_c_value(self, name):
        value = f'Into::<{self.wrapper.module}::{self.wrapper.name}>::into({name})'
        if self.is_ref:
            value = f'(&{value})'
        return ('', value, '')

    def unwrap_rust_value(self, name):
        if self.is_ref:
            value = f'Into::<{self.c_name}>::into(*{name})'
        else:
            value = f'Into::<{self.c_name}>::into({name})'
        return value

    def python_postfix(self):
        return f'result = {self.to_python()}(result)\n'

    def orig_rust(self):
        return f'{"&" if self.is_ref else ""}{self.wrapper.module}::{self.wrapper.name}'

class CEnumWrapper(CEnum, DeriveMixins):
    '''A wrapper for a rust c-style enum, that is, an enum with integer values.'''

    def __init__(self, program, name, docs=''):
        self.program = program
        self.name = name
        self.type = CEnumWrapperType(self)
        super().__init__(program.module, self.type.san_name)
        self.module = program.module
        self.docs = docs
        self.methods = []
    
    def variant(self, name, value, docs=''):
        # TODO: docs
        super().variant(name, value)
        return self

    def method(self, type, name, args, docs='', pyname=None, self_ref=False):
        original = f'{self.type.orig_name}::{name}'
        if self_ref:
            actual_args = [Var(self.type.mut_ref(), 'this')] + args
        else:
            actual_args = [Var(self.type, 'this')] + args

        if pyname is None:
            pyname = name

        self.methods.append(Method(type, self.c_name, name, actual_args,
            make_safe_call(type, original, actual_args), docs=docs
        , pyname=pyname))

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
        
        methods = '\n'.join(m.to_rust() for m in self.methods)

        return decl + methods

    def to_c(self):
        methods = '\n'.join(m.to_c() for m in self.methods)
        return super().to_c() + methods

    def to_swig(self):
        enum = super().to_swig()

        statics = ''
        for method in self.methods:
            statics += super(Method, method).to_swig() + '\n'

        return f'{doxygen(self.docs)}{enum}\n{statics}\n'


    def to_python(self):
        methods = '\n'.join(m.to_python() for m in self.methods)
        return super().to_python() + s(methods, indent=4)
