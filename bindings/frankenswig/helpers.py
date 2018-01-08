import textwrap

def s(string, indent=0):
    '''Helper method for dealing with multiline strings.'''
    return textwrap.indent(textwrap.dedent(string), ' '*indent)

def make_safe_call(type, rust_function, args):
    prefix = []
    args_ = []
    postfix = []

    for i, arg in enumerate(args):
        pre, arg_, post = arg.type.wrap_c_value(arg.name)
        if pre != '':
            prefix.append(pre)
        args_.append(arg_)
        if post != '':
            postfix.append(post)
    
    entry = f'\nlet maybe_panic = panic::catch_unwind(move || {{'
    call =  '\n' if len(prefix) > 0 else ''
    call += '\n'.join(prefix)
    call += f'''\nlet result = {rust_function}({', '.join(args_)});'''
    call += ('\n' if len(postfix) > 0 else '')
    call += '\n'.join(postfix[::-1])
    call += '\n' + type.unwrap_rust_value('result')
    call = s(call, indent=4)
    exit = '\n});'
    exit += '\ncheck_panic!(maybe_panic, default)'

    return entry + call + exit

def javadoc(docs):
    return '/**\n' + '\n *'.join(docs.split('\n')) + '\n */'

def sanitize_rust_name(name):
    if '<' in name:
        first = name.find('<')
        end = name.rfind('>')
        return sanitize_rust_name(name[:first-2]) + sanitize_rust_name(name[first+1:end])
    return name.split('::')[-1]

class Var(object):
    '''This is kinda a weird class.
    It represents an entry in an argument list / struct body.'''
    def __init__(self, type, name):
        self.type = type
        self.name = name
    
    def to_swig(self):
        return f'{self.type.to_swig()} {self.name}'

    def to_c(self):
        return f'{self.type.to_c()} {self.name}'

    def to_rust(self):
        return f'{self.name}: {self.type.to_rust()}'

    def to_python(self):
        return self.name