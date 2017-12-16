'''This module can be used to easily generate interfaces to Rust from many other languages,
by generating a thin SWIG wrapper.

To use it, create a Program() and .add() various Wrappers (StructWrappers, FunctionWrappers)
to the program, then call .write_files() and it will generate three outputs:

- program.rs, containing a thin c-compatible wrapper for the rust API you've defined
- program.h, a header file for the wrapper
- program.i, a swig interface file for the wrapper

You can then compile these files using cargo + swig + a c compiler.

This is not a principled framework, it's quite hacky. However, it gets the job done.
'''

from collections import namedtuple

RUST_HEADER = '''/// GENERATED RUST, DO NOT EDIT
extern crate {module};

use std::os::raw::c_char;

/// Static checking

/// This function throws an error at compile time if it isn't safe to return
/// its argument outside of rust.
/// <T: 'static + Send> is read "the type T contains no references and is safe to move between threads".
/// That is a good enough guarantee for us.
fn borrow_check<T: 'static + Send>(val: T) -> T {{ val }}

// Swig compatible error handling.
// This code is tightly bound to error_handling.inc.i.

// We have to pass errors to c somehow :/
thread_local! {{
    static mut ERROR: Option<String>;
}}
// only usable from rust
fn set_error(code: SwigError, err: String) {{
    ERROR.with(move |e| unsafe {{
        *e = Some(err);
    }})
}}
// called from c
#[no_mangle]
pub unsafe extern "C" fn get_last_err() -> *mut u8 {{
    ERROR.with(|e| unsafe {{
        if let Some((code, err)) = *e {{
            *result = CString::new(err)
                .map(|r| r.into_raw())
                .unwrap_or_else(|_| CString::new("unknown error").into_raw().unwrap())
        }}
    }})
    result_code
}}
// called from c
#[no_mangle]
pub unsafe extern "C" fn free_err(err: *mut u8) {{
    CString::from_raw(err)
}}
'''

C_HEADER = '''/// GENERATED C, DO NOT EDIT
%include <stdint.h>
'''

SWIG_HEADER = '''%module {module}
/// GENERATED SWIG, DO NOT EDIT
%feature("autodoc", "1");
%{{
#include "{module}.h"
%}}
// swig library file that improves output for code using stdint
%include "stdint.i"
// used for throwing exceptions
%include "exceptions.i"
'''


class Type(object):
    '''The type of a variable / return value.'''

    def __init__(self, rust, swig, default='!!no default constructor!!'):
        '''Rust: how this type will be represented in the rust shim code.
        Swig: how this type will be represented in swig / c.'''
        self.rust = rust
        self.swig = swig
        self.default = default

    def to_swig(self):
        '''Formatting for embedding in a swig .i file.'''
        return self.swig

    def to_c(self):
        '''Formatting for embedding in c .h file.'''
        return self.to_swig()

    def to_rust(self):
        '''Formatting for embedding in c .h file.'''
        return self.rust

    def wrap_c_value(self, value):
        return ('', value, '')

    def unwrap_rust_value(self, value):
       return value

u8 = Type('u8', 'uint8_t', '0')
i8 = Type('i8', 'int8_t', '0')
u16 = Type('u16', 'uint16_t', '0')
i16 = Type('i16', 'int16_t', '0')
u32 = Type('u32', 'uint32_t', '0')
i32 = Type('i32', 'int32_t', '0')
u64 = Type('u64', 'uint64_t', '0')
i64 = Type('i64', 'int64_t', '0')
void = Type('()', 'void', '()')

class StructType(Type):
    '''Rust structs are always treated as pointers by SWIG.
    However, a rust API can take values by value, by reference, or by pointer.
    When annotating your api, you can use Struct.type to pass by value,
    Struct.type.ref() to pass by (mutable) reference, etc.'''

    RUST_BY_VALUE = 0
    RUST_MUT_REF = 1
    RUST_RAW_PTR = 2

    def __init__(self, module, name, kind=0):
        super(StructType, self).__init__('*mut '+module+'::'+name, name+'*', default='ptr::null_mut()')
        self.module = module
        self.name = name
        self.kind = kind

    def mut_ref(self):
        return StructType(self.module, self.name, kind=StructType.RUST_MUT_REF)

    def raw(self):
        return StructType(self.module, self.name, kind=StructType.RUST_RAW_PTR)
    
    def wrap_c_value(self, name):
        if self.kind == StructType.RUST_BY_VALUE:
            name = '(*{}).clone()'.format(name)
            return ('', name, '')
        elif self.kind == StructType.RUST_MUT_REF:
            # fn thing(arg: *mut Banana) {
            #    let _arg = *arg;
            #    eng::thing(&mut _arg);
            #    mem::forget(_arg);
            # }
            # This prevents the engine from borrowing the argument in any way,
            # which would be extremely unsafe (other languages can destroy
            # the argument whenever they want).
            pre_check = '\tlet _{0} = *{0};\n'.format(name)
            value =  '&mut _{}'.format(name)
            post_check = '\tmem::forget(_{});\n'.format(name)
            return (pre_check, value, post_check)
        else:
            raise 'Unknown pointer type: {}'.format(name)
    
    def unwrap_rust_value(self, name):
        if self.kind == StructType.RUST_RAW_PTR:
            return name

        if self.kind == StructType.RUST_BY_VALUE:
            result = name
        elif self.kind == StructType.RUST_MUT_REF:
            # if a rust function returns a reference, we just clone it :/
            # It's The Only Way To Be Sure
            result = '{}.clone()'.format(name)

        return 'Box::into_raw(Box::new(borrow_check({})))'.format(result)

class Var(object):
    '''This is kinda a weird class.
    It represents an entry in an argument list / struct body.'''
    def __init__(self, type, name):
        self.type = type
        self.name = name
    
    def to_swig(self):
        return '{0.type.swig} {0.name}'.format(self)

    def to_c(self):
        return self.to_swig()

    def to_rust(self):
        return '{0.name}: {0.type.rust}'.format(self)

    def wrap_c_value(self):
        return self.type.wrap_c_value(self.name)

def make_safe_call(type, rust_function, args):
    prefix = []
    args_ = []
    postfix = []

    for i, arg in enumerate(args):
        pre, arg_, post = arg.wrap_c_value()
        prefix.append(pre)
        args_.append(arg_)
        postfix.append(post)

    return ''.join(prefix) +\
            '\tlet result = {}({});\n'.format(rust_function, ', '.join(args_)) +\
            ''.join(postfix[::-1]) +\
            '\tborrow_check(result)\n'

class Function(object):
    def __init__(self, type, name, args, body=''):
        self.type = type
        self.name = name
        self.args = args
        self.body = body
    
    def to_swig(self):
        result = '{0.type.swig} {0.name}('.format(self)
        result += ', '.join(a.to_swig() for a in self.args)
        result += ');\n'
        return result

    def to_c(self):
        return self.to_swig()

    def to_rust(self):
        result = '#[no_mangle]\npub extern "C" fn {0.name}('.format(self)
        result += ', '.join(a.to_rust() for a in self.args)
        result += ') -> {0.type.rust} {{\n{1}}}\n'.format(self, self.body)
        return result

class StructWrapper(object):
    def __init__(self, module, name, docs=''):
        self.module = module
        self.name = name
        self.members = []
        self.member_docs = []
        self.methods = []
        self.method_names = []
        self.method_docs = []
        self.getters = []
        self.type = StructType(module, name)
        self.constructor_ = None
        self.constructor_docs = ''
        self.destructor = Function(void, 'delete_'+self.name, [Var(self.type, 'self')],
            '\tBox::from_raw(self)\n'
        )
        self.docs = docs
    
    def constructor(self, rust_method, args, docs=''):
        assert self.constructor_ is None
        self.constructor_docs = docs

        method = '{}::{}::{}'.format(self.module, self.name, rust_method)

        self.constructor_ = Function(
            self.type,
            'new_' + self.name,
            args,
            make_safe_call(self.type.mut_ref(), method, args)
        )

        return self

    def member(self, type, name, docs=''):
        self.members.append(Var(type,name))
        self.member_docs.append(docs)

        pre, arg, post = self.type.mut_ref().wrap_c_value('self')
        arg = '(' + arg + ')'

        getter = Function(type, self.name + "_get_" + name, [Var(self.type, 'self')],
            pre +
            '\tlet result = ' + type.unwrap_rust_value(arg + '.' + name) + ';\n' +
            post +
            '\tresult\n'
        )

        vpre, varg, vpost = type.wrap_c_value(name)
        
        setter = Function(void, self.name + "_set_" + name,
            [Var(self.type, 'self'), Var(type,name)],
            pre + vpre +
            '\t{}.{} = {};\n'.format(arg, name, varg) +
            post + vpost
        )
        self.getters.append(getter)
        self.getters.append(setter)

        return self

    def method(self, type, name, args, docs=''):
        # we use the "Universal function call syntax"
        # Type::method(&mut self, arg1, arg2)
        # which is equivalent to:
        # self.method(arg1, arg2)
        method = '{}::{}::{}'.format(self.module, self.name, name)
        actual_args = [Var(self.type.mut_ref(), 'self')] + args

        self.methods.append(Function(type, self.name + '_' + name, actual_args,
            make_safe_call(type, method, actual_args)
        ))
        self.method_names.append(name)
        self.method_docs.append(docs)
        return self

    def to_c(self):
        assert self.constructor_ is not None
        definition = 'typedef struct {0.name} {0.name};\n'.format(self)
        definition += self.constructor_.to_c()
        definition += self.destructor.to_c()
        definition += ''.join(getter.to_c() for getter in self.getters)
        definition += ''.join(method.to_c() for method in self.methods)
        return definition

    def to_swig(self):
        '''Generate a SWIG interface for this struct.'''
        assert self.constructor_ is not None
        definition = '%feature("docstring", "{}");\n'.format(self.docs)
        definition += 'typedef struct {0.name} {{}} {0.name};\n'.format(self)
        # We use SWIG's %extend command to attach "methods" to this struct:
        # %extend Bananas {
        #     int peel(int);
        # }
        # results in a `peel` method on the Bananas object, which
        # calls into a method:
        # int Bananas_peel(Bananas *self, int)
        # which we generate :)
        extra = '%extend {0.name} {{\n'.format(self)
        extra += ''

        for method, method_name, method_docs in zip(self.methods, self.method_names, self.method_docs):
            extra += '\t%feature("docstring", "{}");\n'.format(method_docs)
            extra += '\t' + Function(method.type, method_name, method.args[1:]).to_swig()
        for member, member_docs in zip(self.members, self.member_docs):
            extra += '\n\t%feature("docstring", "{}");\n'.format(member_docs)
            # add getters
            extra += '\t' + member.to_swig() + ';'

        extra += '\n};\n'

        return '{}\n{}'.format(definition, extra)

    def to_rust(self):
        '''Generate a rust implementation for this struct.'''
        assert self.constructor_ is not None
        # assume that struct is already defined
        definition = self.constructor_.to_rust()
        definition += self.destructor.to_rust()
        definition += ''.join(getter.to_rust() for getter in self.getters)
        definition += ''.join(method.to_rust() for method in self.methods)

        return definition

class FunctionWrapper(Function):
    def __init__(self, module, type, name, args):
        body = 'check({}::{}('.format(module, name)
        body += ', '.join(a.name for a in args)
        body += '))'
        
        super(FunctionWrapper, self).__init__(type, name, args, body)

class Program(object):
    def __init__(self, name):
        self.name = name
        self.elements = []
    
    def add(self, elem):
        return self

    def to_rust(self):
        return RUST_HEADER.format(module=self.name) + ''.join(elem.to_rust() for elem in self.elements)

    def to_c(self):
        return C_HEADER.format(module=self.name) +\
            ''.join(elem.to_c() for elem in self.elements)

    def to_swig(self):
        result = SWIG_HEADER.format(module=self.name)
        result += ''.join(elem.to_swig() for elem in self.elements)
        return result

    def write_files(self):
        with open(self.name + '.rs', 'w') as f:
            f.write(self.to_rust())
        with open(self.name + '.h', 'w') as f:
            f.write(self.to_c())
        with open(self.name + '.i', 'w') as f:
            f.write(self.to_swig())
    
    def struct(self, *args, **kwargs):
        result = StructWrapper(self.name, *args, **kwargs)
        self.elements.append(result)
        return result
    
    def function(self, *args, **kwargs):
        result = FunctionWrapper(self.name, *args, **kwargs)
        self.elements.append(result)
        return result

p = Program('speleothem')

Apple = p.struct('Apple', docs='High in cyanide.')\
         .constructor('new', [])\
         .member(u32, 'stem')\
         .method(i64, 'bite', [Var(u8, 'greedy'), Var(i16, 'solipsistic')])\

Banana = p.struct('Banana', docs='High in potassium.')\
        .constructor('new', [])\
        .member(u8, 'hello')\
        .member(u32, 'goodbye')\
        .method(i64, 'peel', [Var(i16, 'amount')], docs='be careful!!')\
        .method(i64, 'consume', [Var(u8, 'greedy'), Var(i16, 'solipsistic')])\
        .method(i64, 'peel_harder', [Var(i16, 'amount'), Var(u64, 'veracity')])\
        .method(Apple.type, 'transform', [])

Banana.method(void, 'copy_from', [Var(Banana.type.mut_ref(), 'other')])

print(p.to_rust())
print('---')
print(p.to_c())
print('---')
print(p.to_swig())

p.write_files()