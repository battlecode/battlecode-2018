class Type(object):
    '''The type of a variable / return value.'''

    def __init__(self, rust, swig, python, default='!!no default value for type!!'):
        '''Rust: how this type will be represented in the rust shim code.
        Swig: how this type will be represented in swig / c.'''
        self.rust = rust
        self.swig = swig
        self.python = python
        self.default = default

    def to_swig(self):
        '''Formatting for embedding in a swig .i file.'''
        return self.swig

    def to_c(self):
        '''Formatting for embedding in c .h file.'''
        return self.swig

    def to_rust(self):
        '''Formatting for embedding in c .h file.'''
        return self.rust

    def to_python(self):
        return self.python

    def wrap_c_value(self, value):
        # see make_safe_call
        # the first value is used to validate incoming arguments
        # the second is used to pass them to the function we're calling
        # the third is used to 
        return ('', value, '')

    def unwrap_rust_value(self, value):
       return value

    def unwrap_python_value(self, value):
        return value

    def python_postfix(self):
        return ''

class BuiltinWrapper(object):
    def __init__(self, *args):
        self.type = Type(*args)

char = BuiltinWrapper('char', 'c_char', 'int', '0')
u8 = BuiltinWrapper('u8', 'uint8_t', 'int', '0')
i8 = BuiltinWrapper('i8', 'int8_t', 'int', '0')
u16 = BuiltinWrapper('u16', 'uint16_t', 'int', '0')
i16 = BuiltinWrapper('i16', 'int16_t', 'int', '0')
u32 = BuiltinWrapper('u32', 'uint32_t', 'int', '0')
i32 = BuiltinWrapper('i32', 'int32_t', 'int', '0')
u64 = BuiltinWrapper('u64', 'uint64_t', 'int', '0')
i64 = BuiltinWrapper('i64', 'int64_t', 'int', '0')
void = BuiltinWrapper('()', 'void', 'int', '()')

# boolean's an odd case.
# it's size isn't actually defined by C, so we can't return it from rust.
# instead, we pass around uint8s, and convert them to bools when they get into other languages.
# todo: java
boolean = BuiltinWrapper('u8', 'uint8_t', 'bool', '0')
boolean.type.wrap_c_value = lambda name: ('', f'{name} as bool', '')
boolean.type.unwrap_rust_value = lambda name: f'{name} as u8'
boolean.type.python_postfix = lambda: 'result = bool(result)\n'
boolean.type.unwrap_python_value = lambda name: f'int({name})'

