from .helpers import *

class Function(object):
    def __init__(self, type, name, args, body='', docs=''):
        self.type = type
        self.name = name
        self.args = args
        self.body = body
        self.docs = docs

    def to_swig(self):
        result = s(f'''\
            {self.type.to_swig()} {self.name}({', '.join(a.to_swig() for a in self.args)});
        ''')
        return result

    def to_c(self):
        return f'''{doxygen(self.docs)}{self.type.to_c()} {self.name}({', '.join(a.to_c() for a in self.args)});\n'''

    def to_rust(self):
        result = s(f'''\
            #[no_mangle]
            pub extern "C" fn {self.name}({', '.join(a.to_rust() for a in self.args)}) -> {self.type.rust} {{
                let _default: {self.type.rust} = {self.type.default};
            '''
        )
        result += s(self.body, indent=4)
        result += '\n}\n'
        return result

    @staticmethod
    def pyentry(type, args, pyname, docs):
        pyargs = ', '.join(a.to_python() for a in args)
        start = f'def {pyname}({pyargs}):\n'
        hargs = args if len(args) > 0 and args[0].name != 'self' else args[1:]
        mypy_hint = ', '.join(a.type.to_python() for a in hargs)
        mypy_hint = f'# type: ({mypy_hint}) -> {type.to_python()}'
        doc_hint = ''
        for a in args:
            doc_hint += f':type {a.name}: {a.type.to_python()}\n'
        doc_hint += f':rtype: {type.to_python()}\n'
        asserts = ''
        for arg in args:
            if arg.name == 'self':
                continue
            asserts += f'assert type({arg.name}) is {arg.type.to_python()}, "incorrect type of arg {arg.name}: should be {arg.type.to_python()}, is {{}}".format(type({arg.name}))\n' 

        docs = s(f"{mypy_hint}\n'''{docs}\n{doc_hint}'''\n{asserts}\n", indent=4)
        return start + docs

    def to_swig(self):
        result = s(f'''\
            %newobject {self.name};
            {self.type.to_swig()} {self.name}({', '.join(a.to_swig() for a in self.args)});
        ''')
        return result

    def to_python(self):
        # note: we assume that error + null checking, etc. will occur on the rust side.
        # (it'll probably be much faster there in any case.)
        pyargs = ', '.join(a.type.wrap_python_value(a.name) for a in self.args)

        body = f'result = _lib.{self.name}({pyargs})\n'
        body += '_check_errors()\n'
        body += self.type.python_postfix()
        body += 'return result\n'
        return Function.pyentry(self.type, self.args, self.name, self.docs) + s(body, indent=4)

class Method(Function):
    '''A function contained within some type.'''
    def __init__(self, type, container, method_name, args, body='', docs='', pyname=None, static=False, getter=False):
        self.container = container
        self.method_name = method_name
        self.static = static
        super().__init__(type, f'{self.container}_{self.method_name}', args, body, docs)
        if pyname is None:
            self.pyname = self.method_name
        else:
            self.pyname = pyname
        self.getter = getter

    def to_swig(self):
        result = s(f'''\
            %newobject {self.method_name};
            {self.type.to_swig()} {self.method_name}({', '.join(a.to_swig() for a in self.args[1:])});
        ''')
        return result
    
    def to_python(self):
        if self.static:
            args = self.args
        else:
            args = [Var(self.args[0].type, 'self')] + self.args[1:]
        pyargs = ', '.join(a.type.wrap_python_value(a.name) for a in args)

        body = f'result = _lib.{self.name}({pyargs})\n'
        body += '_check_errors()\n'
        body += self.type.python_postfix()
        body += 'return result\n'
        if self.static:
            pre = '@staticmethod\n'
        elif self.getter:
            pre = '@property\n'
        else:
            pre = ''
        return pre + Function.pyentry(self.type, args, self.pyname, self.docs) + s(body, indent=4)

class FunctionWrapper(Function):
    def __init__(self, program, type, name, args):
        self.program = program
        body = make_safe_call(type, f'{program.module}::{name}', args)
        super(FunctionWrapper, self).__init__(type, sanitize_rust_name(name), args, body)
