from .helpers import *

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

    def wrap_python_value(self, value):
        return value

    def python_postfix(self):
        return ''

    def result(self):
        return ResultType(self)

    def orig_rust(self):
        return self.rust

class BuiltinType(Type):
    def __init__(self, rust, swig, python, default="!!no default!!", is_ref=False):
        super().__init__(rust, swig, python, default=default)
        self.is_ref = is_ref
    
    def ref(self):
        return BuiltinType(self.rust, self.swig, self.python, default=self.default, is_ref=True)

    def wrap_c_value(self, value):
        if self.is_ref:
            return '&' + value
        else:
            return ('', value, '')

    def unwrap_rust_value(self, value):
        if self.is_ref:
            return '*' + value
        else:
            return value

class BuiltinWrapper(object):
    def __init__(self, *args):
        self.type = BuiltinType(*args)

char = BuiltinWrapper('char', 'c_char', 'int', '0')
u8 = BuiltinWrapper('u8', 'uint8_t', 'int', '0')
i8 = BuiltinWrapper('i8', 'int8_t', 'int', '0')
u16 = BuiltinWrapper('u16', 'uint16_t', 'int', '0')
i16 = BuiltinWrapper('i16', 'int16_t', 'int', '0')
u32 = BuiltinWrapper('u32', 'uint32_t', 'int', '0')
i32 = BuiltinWrapper('i32', 'int32_t', 'int', '0')
u64 = BuiltinWrapper('u64', 'uint64_t', 'int', '0')
i64 = BuiltinWrapper('i64', 'int64_t', 'int', '0')
void = BuiltinWrapper('()', 'void', 'None', '()')
usize = BuiltinWrapper('usize', 'uintptr_t', 'int', '0')
isize = BuiltinWrapper('isize', 'intptr_t', 'int', '0')

# boolean's an odd case.
# it's size isn't actually defined by C, so we can't return it from rust.
# instead, we pass around uint8s, and convert them to bools when they get into other languages.
# todo: java
boolean = BuiltinWrapper('u8', 'uint8_t', 'bool', '0')
boolean.type.wrap_c_value = lambda name: ('', f'{name} as bool', '')
boolean.type.unwrap_rust_value = lambda name: f'{name} as u8'
boolean.type.python_postfix = lambda: 'result = bool(result)\n'
boolean.type.wrap_python_value = lambda name: f'int({name})'

# hack used in "debug" impl
_stringliteral = BuiltinWrapper('&str', 'INVALID', 'INVALID', '""')

class ResultType(Type):
    '''A Result<T, failure::Error>.'''

    def __init__(self, wrapped):
        super().__init__(wrapped.rust, wrapped.swig, wrapped.python, wrapped.default)
        self.wrapped = wrapped
    
    def wrap_c_value(self, value):
        raise Exception("Results can only be returned")
    
    def wrap_python_value(self, value):
        raise Exception("Results can only be returned")
    
    def unwrap_rust_value(self, value):
        v = self.wrapped.unwrap_rust_value('v')
        return f'check_result!({value}.map(|v| {v}), default)'

    def python_postfix(self):
        return self.wrapped.python_postfix()

# TODO: make sure this works with utf-8 stuff in python 2, java, etc.
class StringType(Type):
    '''A rust String.'''
    def __init__(self, module):
        self.module = module
        super().__init__('*const c_char', 'char*', 'str', '0 as *const _')
    
    def wrap_c_value(self, name):
        pre = ''
        value = f'(unsafe{{CStr::from_ptr({name})}}).to_string_lossy().to_string()'
        post = ''
        return (pre, value, post)

    def unwrap_rust_value(self, value):
        return f'check_result!(CString::new({value}).map(|s| s.into_raw()), default)'

    def wrap_python_value(self, value):
        return f'_ffi.new("char[]", {value})'

    def python_postfix(self):
        return s(f'''\
            _result = _ffi.string(result)
            _lib.{self.module}_free_string(result)
            result = _result
        ''')
    
    def orig_rust(self):
        return 'String'

class StrRefType(StringType):
    '''The &str type.
    Note that a copy will be made when passing over the ffi boundary even when using &str instead of
    String.'''
    def __init__(self, module):
        super().__init__(module)

    def wrap_c_value(self, name):
        value = f'&*(unsafe{{CStr::from_ptr({name})}}).to_string_lossy()'
        return ('', value, '')

    def orig_rust(self):
        return '&str'
