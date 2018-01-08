from .helpers import *
from .type import Type, void, boolean, _stringliteral
from .function import Function, Method

class StructType(Type):
    '''Rust structs are always treated as pointers by SWIG.
    However, a rust API can take values by value, by reference, or by pointer.
    When annotating your api, you can use Struct.type to pass by value,
    Struct.type.ref() to pass by (mutable) reference, etc.
    Note that this is only for defining the types of structs, the actual struct codegen
    is in StructWrapper.'''

    RUST_BY_VALUE = 0
    RUST_REF = 1
    RUST_MUT_REF = 2

    def __init__(self, wrapper, kind=0):
        self.wrapper = wrapper
        super(StructType, self).__init__(
            '*mut '+wrapper.module+'::'+wrapper.name,
            wrapper.c_name+'*',
            sanitize_rust_name(wrapper.name),
            default='0 as *mut _'
        )
        self.kind = kind

    def ref(self):
        '''Mutable references coerce to non-mutable references, and the
        types in the C API are the same.'''
        return StructType(self.wrapper, kind=StructType.RUST_MUT_REF)

    def mut_ref(self):
        return StructType(self.wrapper, kind=StructType.RUST_MUT_REF)

    def wrap_c_value(self, name):
        pre_check = f'let _{name} = check_null!({name}, default);'
        if self.kind == StructType.RUST_BY_VALUE:
            value = f'_{name}.clone()'
        elif self.kind == StructType.RUST_MUT_REF:
            value = f'_{name}'
        else:
            raise Exception(f'Unknown pointer type: {self.kind}')
        return (pre_check, value, '')
    
    def unwrap_rust_value(self, name):
        if self.kind == StructType.RUST_BY_VALUE:
            result = name
        elif self.kind == StructType.RUST_MUT_REF:
            # if a rust function returns a reference, we just clone it :/
            # It's The Only Way To Be Sure
            result = f'{name}.clone()'

        return f'Box::into_raw(Box::new(borrow_check({result})))'

    def wrap_python_value(self, name):
        return f'{name}._ptr'

    def python_postfix(self):
        pyname = sanitize_rust_name(self.wrapper.name)
        return s(f'''\
            _result = {pyname}.__new__({pyname})
            if result != _ffi.NULL:
                _result._ptr = result
            result = _result
        ''')
    
    def orig_rust(self):
        return f'{"&" if self.kind == StructType.RUST_MUT_REF else ""}{self.wrapper.module}::{self.wrapper.name}'

class DeriveMixins(object):
    '''Helpers for easily bindings #[derive]'d methods.'''
    def serialize(self):
        '''Add "from_json" and "to_json" methods.'''

        args = [Var(self.program.strref.type, "s")]
        type = self.type.result()
        self.methods.append(Method(
            type, self.c_name, "from_json", args,
            make_safe_call(type, f'serde_json::from_str::<{self.module}::{self.name}>', args),
            docs=f'Deserialize a {self.type.to_python()} from a JSON string',
            static=True
        ))
 
        args = [Var(self.type.mut_ref(), 'this')]
        type = self.program.string.type.result()
        self.methods.append(Method(type, self.c_name, "to_json", args,
            make_safe_call(type, 'serde_json::to_string', args), docs=f'Serialize a {self.type.to_python()} to a JSON string'
        ))

        return self

    def debug(self):
        args = [Var(self.type.mut_ref(), 'this')]
        inner_args = [Var(_stringliteral.type, '"{:?}"')] + args
        type = self.program.string.type
        self.methods.append(Method(type, self.c_name, "debug", args,
            make_safe_call(type, 'format!', inner_args), docs=f'Create a human-readable representation of a {self.type.to_python()}',
        pyname="__repr__"))
    
    def clone(self):
        self.method(self.type, "clone", [], docs=f"Deep-copy a {self.type.to_python()}", self_ref=True)

    def eq(self):
        self.method(boolean.type, "eq", [Var(self.type.ref(), "other")], docs=f"Deep-copy a {self.type.to_python()}", pyname="__eq__", self_ref=True)

class StructWrapper(DeriveMixins):
    def __init__(self, program, name, docs='', module=None):
        self.program = program
        if module is None:
            self.module = program.module
        else:
            self.module = module
        self.name = name
        self.c_name = f'{self.program.module}_{sanitize_rust_name(self.name)}'
        self.members = []
        self.member_docs = []
        self.methods = []
        self.getters = []
        self.setters = []
        self.type = StructType(self)
        self.constructor_ = None
        self.docs = docs

        pre, arg, post = self.type.mut_ref().wrap_c_value('this')
        self.destructor = Function(void.type, f'delete_{self.c_name}',
            [Var(self.type, 'this')],
            pre + f'\nunsafe {{ Box::from_raw({arg}); }}' + post
        )

    def constructor(self, rust_method, args, docs='', result=False):
        assert self.constructor_ is None

        method = f'{self.module}::{self.name}::{rust_method}'
        ret = self.type.result() if result else self.type

        self.constructor_ = Function(
            ret,
            f'new_{self.c_name}',
            args,
            make_safe_call(ret, method, args),
            docs=docs
        )

        return self

    def member(self, type, name, docs=''):
        self.members.append(Var(type,name))
        self.member_docs.append(docs)

        pre, arg, post = self.type.mut_ref().wrap_c_value('this')
        arg = '(' + arg + ')'

        getter = Method(type, self.c_name, f"{name}_get", [Var(self.type, 'this')],
            pre +
            '\nlet result = ' + type.unwrap_rust_value(arg + '.' + name + '.clone()') + ';\n' +
            post +
            '\nresult',
            docs=docs,
            pyname=f'{name}'
        )

        vpre, varg, vpost = type.wrap_c_value(name)

        setter = Method(void.type, self.c_name, f"{name}_set",
            [Var(self.type, 'this'), Var(type, name)],
            pre + vpre +
            f'\n{arg}.{name} = {varg};\n' +
            post + vpost,
            docs=docs,
            pyname=f'{name}'
        )
        self.getters.append(getter)
        self.setters.append(setter)

        return self

    def method(self, type, name, args, docs='', static=False, pyname=None, self_ref=True, getter=False):
        # we use the "Universal function call syntax"
        # Type::method(&mut self, arg1, arg2)
        # which is equivalent to:
        # self.method(arg1, arg2)
        original = f'{self.module}::{self.name}::{name}'
        if static:
            actual_args = args
        else:
            if self_ref:
                actual_args = [Var(self.type.mut_ref(), 'this')] + args
            else:
                actual_args = [Var(self.type, 'this')] + args
        
        if pyname is None:
            pyname = name

        self.methods.append(Method(type, self.c_name, name, actual_args,
            make_safe_call(type, original, actual_args), docs=docs
        , pyname=pyname, static=static, getter=getter))
        return self

    def to_c(self):
        definition = 'typedef struct {0.c_name} {0.c_name};\n'.format(self)
        if self.constructor_:
            definition += self.constructor_.to_c()
        definition += self.destructor.to_c()
        definition += ''.join(getter.to_c() for getter in self.getters)
        definition += ''.join(setter.to_c() for setter in self.setters)
        definition += ''.join(method.to_c() for method in self.methods)
        return definition

    def to_swig(self):
        '''Generate a SWIG interface for this struct.'''
        definition = '%feature("docstring", "{}");\n'.format(self.docs)
        # luckily, swig treats all structs as pointers anyway
        definition += 'typedef struct {0.c_name} {{}} {0.c_name};\n'.format(self)
        # see:
        # http://www.swig.org/Doc3.0/Arguments.html#Arguments_nn4
        # note: this prints "Can't apply (sp_Apple *INPUT). No typemaps are defined."
        # but afaict that's a complete lie, it totally works
        definition += '%apply {0.c_name}* INPUT {{ {0.c_name}* a }};'.format(self)
        # We use SWIG's %extend command to attach "methods" to this struct:
        # %extend Bananas {
        #     int peel(int);
        # }
        # results in a `peel` method on the Bananas object, which
        # calls into a method:
        # int Bananas_peel(Bananas *self, int)
        # which we generate :)

        body = ''
        if self.constructor_:
            body += f'%feature("docstring", "{self.constructor_.docs}");\n'
            body += f'''{self.c_name}({", ".join(a.to_swig() for a in self.constructor_.args)});\n'''
            body += f'~{self.c_name}();\n'
        for method in self.methods:
            body += method.to_swig()
        for member, member_docs in zip(self.members, self.member_docs):
            body += f'%feature("docstring", "{member_docs}");\n{member.to_swig()};\n'

        body = s(body, indent=4)
        extra = f'%extend {self.c_name} {{\n{body}}}\n'

        return f'{definition}\n{extra}'

    def to_rust(self):
        '''Generate a rust implementation for this struct.'''
        # assume that struct is already defined
        definition = ''
        if self.constructor_:
            definition += self.constructor_.to_rust()
        definition += self.destructor.to_rust()
        definition += ''.join(getter.to_rust() for getter in self.getters)
        definition += ''.join(setter.to_rust() for setter in self.setters)
        definition += ''.join(method.to_rust() for method in self.methods)

        return definition

    def to_python(self):
        start = s(f'''\
        class {sanitize_rust_name(self.name)}(object):
            __slots__ = ['_ptr']
        ''')

        if self.constructor_:
            cargs = [Var(self.type, 'self')] + self.constructor_.args
            cinit = Function.pyentry(
                self.type,
                cargs,
                '__init__',
                self.constructor_.docs
                )
            cpyargs = ', '.join(a.type.wrap_python_value(a.name) for a in cargs[1:])
            cbody = f'ptr = _lib.{self.constructor_.name}({cpyargs})\n'
            cbody += 'if ptr != _ffi.NULL: self._ptr = ptr\n'
            cbody += '_check_errors()\n'
        else:
            cinit = Function.pyentry(
                self.type,
                [Var(self.type, 'self')],
                '__init__',
                'INVALID: this object cannot be constructed from Python code!'
            )
            cbody = 'raise TypeError("This object cannot be constructed from Python code!")'

        constructor = cinit + s(cbody, indent=4) + '\n'

        dinit = Function.pyentry(void.type, [Var(self.type, 'self')], '__del__', 'Clean up the object.')
        dbody = s(f'''\
            if hasattr(self, '_ptr'):
                # if there was an error in the constructor, we'll have no _ptr
                _lib.{self.destructor.name}(self._ptr)
                _check_errors()
        ''', indent=4)

        definition = dinit + dbody

        definition += '\n'.join('@property\n' + getter.to_python() for getter in self.getters) + '\n'
        definition += '\n'.join(f'@{setter.pyname}.setter\n' + setter.to_python()
                                for setter in self.setters) + '\n'
        definition += '\n'.join(method.to_python() for method in self.methods) + '\n'

        return start + s(constructor + definition, indent=4)
