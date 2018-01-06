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
            %feature("docstring", "{self.docs}");
            {self.type.to_swig()} {self.name}({', '.join(a.to_swig() for a in self.args)});
        ''')
        return result

    def to_c(self):
        return f'''{self.type.to_c()} {self.name}({', '.join(a.to_c() for a in self.args)});\n'''

    def to_rust(self):
        result = s(f'''\
            #[no_mangle]
            pub extern "C" fn {self.name}({', '.join(a.to_rust() for a in self.args)}) -> {self.type.rust} {{
                const default: {self.type.rust} = {self.type.default};
            '''
        )
        result += s(self.body, indent=4)
        result += '\n}\n'
        return result

    @staticmethod
    def pyentry(args, pyname, docs):
        pyargs = ', '.join(a.to_python() for a in args)
        start = f'def {pyname}({pyargs}):\n'
        docs = s(f"'''{docs}'''\n", indent=4)
        return start + docs #+ s(checks, indent=4)

    def to_swig(self):
        result = s(f'''\
            %feature("docstring", "{self.docs}");
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
        return Function.pyentry(self.args, self.name, self.docs) + s(body, indent=4)

class Method(Function):
    '''A function contained within some type.'''
    def __init__(self, type, container, method_name, args, body='', docs='', pyname=None, static=False):
        self.container = container
        self.method_name = method_name
        self.static = static
        super().__init__(type, f'{self.container}_{self.method_name}', args, body, docs)
        if pyname is None:
            self.pyname = self.method_name
        else:
            self.pyname = pyname

    def to_swig(self):
        result = s(f'''\
            %feature("docstring", "{self.docs}");{'ERROR BROKEN STATIC METHODS' if self.static else ''}
            {self.type.to_swig()} {self.method_name}({', '.join(a.to_swig() for a in self.args)});
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
        pre = '@staticmethod\n' if self.static else ''
        return pre + Function.pyentry(args, self.pyname, self.docs) + s(body, indent=4)

class FunctionWrapper(Function):
    def __init__(self, module, type, name, args):
        body = make_safe_call(type, f'{module}::{name}', args)
        super(FunctionWrapper, self).__init__(type, name, args, body)
