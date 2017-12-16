'''This module can be used to easily generate interfaces to Rust from many other languages,
by generating a thin SWIG wrapper.

To use it, create a Program() and then call .struct() and .function().
The rust struct:
    struct Banana {
        age: u8
    }
    impl Banana {
        fn days_until_rotten(&mut self, fly_count: u8) -> u8 {
            (20 - self.age) / fly_count
        }
    }
Can be bound like so:
    p = Program("my_crate")
    p.struct("Banana")
        .member(u8, age)
        .method(u8, 'days_until_rotten', [Var(u8, 'fly_count')])

Once you've created your definitions, call .write_files() and this script will generate three outputs:

- program.rs, containing a thin c-compatible wrapper for the rust API you've defined
- program.h, a header file for the wrapper
- program.i, a swig interface file for the wrapper

program.rs is a standalone file that imports your existing crate, 
and you shouldn't need to modify your crate's code.

This is not a principled framework, it's quite hacky. However, it gets the job done.
If you need to edit this file, I strongly recommend frequently examining the output you're getting.

# Ownership
The way we deal with ownership boundaries across languages is simple: the binding language owns all of your
data structures. Period, done. This has some consequences:

- Non-Send and non-'static types cannot be returned from rust. Rust code relies heavily on the
  borrow checker for correctness, there's no really any good way to do this that won't result in your Rust
  code stomping all over some poor interpreter's unsuspecting heap. This is enforced at compile time.
- However, it's perfectly fine to have methods that take rust borrowed references *in*, even mutable borrowed
  references.
- Also, if you have methods / functions that return references to Clone types, like so:
    #derive(Clone)
    struct Dog {
        //...
    }
    impl Kennel {
        fn acquire_dog(&self) -> &Dog;
    }
  You can bind that as:
    Kennel.method(Dog.type.cloned(), "acquire_dog", [])
  And the returned dog will be cloned.

In order to enforce thread safety, by default, a language-specific lock is used on every object returned.
This is the GIL in python, java synchronized blocks, etc.
It may be reasonable to disable these locks for Sync types, I haven't checked.
'''

from collections import namedtuple
import textwrap
import io

def s(string, indent=0):
    '''Helper method for dealing with long strings.'''
    return textwrap.indent(textwrap.dedent(string), ' '*indent)

RUST_HEADER = '''/// GENERATED RUST, DO NOT EDIT
extern crate {module};

use std::os::raw::c_char;
use std::panic;
use std::ptr;
use std::mem;

// Static error checking

/// This function throws an error at compile time if it isn't safe to return
/// its argument outside of rust.
/// <T: 'static + Send> is read "the type T contains no references and is safe to move between threads".
/// That is a good enough guarantee for us.
fn borrow_check<T: 'static + Send>(val: T) -> T {{ val }}

// Runtime error checking

// see https://github.com/swig/swig/blob/master/Lib/swigerrors.swg
#[repr(i8)]
enum SwigError {{
    NoError         = 0,
    Unknown        = -1,
    IO             = -2,
    Runtime        = -3,
    Index          = -4,
    Type           = -5,
    DivisionByZero = -6,
    Overflow       = -7,
    Syntax         = -8,
    Value          = -9,
    System         = -10,
    Attribute      = -11,
    Memory         = -12,
    NullReference  = -13
}}
// We have to pass errors to c somehow :/
thread_local! {{
    static mut ERROR: Option<(code, String)>;
}}
// only usable from rust
fn set_error(code: SwigError, err: String) {{
    ERROR.with(move |e| unsafe {{
        *e = Some((code, err));
    }})
}}
// called from c
#[no_mangle]
pub unsafe extern "C" fn {module}_get_last_err(result: *mut *mut c_char) -> i8 {{
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
pub unsafe extern "C" fn {module}_free_err(err: *mut c_char) {{
    if err != ptr::null_mut() {{
        CString::from_raw(err)
    }}
}}
// you ever wonder if you're going too deep?
// because I haven't.
macro_rules! check_null {{
    ($val:expr, $default:expr) => {{
        if $val == ptr::null() {{
            set_error(SwigError::NullReference, "self is null".into());
            return $default;
        }} else {{
            *$val
        }}
    }};
}}
macro_rules! check_panic {{
    ($maybe_panic:expr, $default:expr) => {{
        if let Err(err) = $maybe_panic {{
            let cause = err.downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|_| err.downcast_ref::<String>())
                .unwrap_or_else(|_| "unknown panic, mysterious".to_string());
            set_error(SwigError::Runtime, format!("panic occurred, talk to the devs: {{}}", cause));
            return $default;
        }} else if let Ok(result) = maybe_panic {{
            result
        }}
    }};
}}



'''

C_HEADER = '''/// GENERATED C, DO NOT EDIT
#include <stdint.h>
int8_t {module}_get_last_err(char** result);
int8_t {module}_free_err(char* err);
'''

SWIG_HEADER = '''%module {module}
/// GENERATED SWIG, DO NOT EDIT
%feature("autodoc", "1");
%{{
#include "{module}.h"

#ifdef __GNUC__
    #define unlikely(expr)  __builtin_expect(!(expr),  0)
#else
    #define unlikely(expr) (expr)
#endif
%}}

// swig library file that improves output for code using stdint
%include "stdint.i"
// used for throwing exceptions
%include "exception.i"
// used to tell swig to not generate pointer types for arguments
// passed by pointer
%include "typemaps.i"

// This code is inserted around every method call.
%exception {{
    $action
    char *err;
    int8_t code;
    if (unlikely((code = {module}_get_last_err(&err)))) {{
        SWIG_exception(code, err);
        {module}_free_err(err);
    }}
}}

// We generate code with the prefix "{module}_".
// This will strip it out.
%rename("%(strip:[{module}_])s") "";
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
        return self.swig

    def to_rust(self):
        '''Formatting for embedding in c .h file.'''
        return self.rust

    def wrap_c_value(self, value):
        # see make_safe_call
        # the first value is used to validate incoming arguments
        # the second is used to pass them to the function we're calling
        # the third is used to 
        return ('', value, '')

    def unwrap_rust_value(self, value):
       return value

char = Type('char', 'c_char', '0')
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
    Struct.type.ref() to pass by (mutable) reference, etc.
    Note that this is only for defining the types of structs, the actual struct codegen
    is in StructWrapper.'''

    RUST_BY_VALUE = 0
    RUST_MUT_REF = 1
    RUST_RAW_PTR = 2

    def __init__(self, module, name, kind=0):
        super(StructType, self).__init__(
            '*mut '+module+'::'+name,
            module + '_' + name+'*',
            default='0 as *mut '+module+'::'+name
        )
        self.module = module
        self.name = name
        self.kind = kind

    def mut_ref(self):
        return StructType(self.module, self.name, kind=StructType.RUST_MUT_REF)

    def raw(self):
        return StructType(self.module, self.name, kind=StructType.RUST_RAW_PTR)

    def wrap_c_value(self, name):
        if self.kind == StructType.RUST_BY_VALUE:
            name = f'(*{name}).clone()'
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
            pre_check = f'let _{name} = check_null!({name});'
            value = f'&mut _{name}'
            post_check = f'mem::forget(_{name});'
            return (pre_check, value, post_check)
        else:
            raise Exception(f'Unknown pointer type: {self.kind}')
    
    def unwrap_rust_value(self, name):
        if self.kind == StructType.RUST_RAW_PTR:
            return name

        if self.kind == StructType.RUST_BY_VALUE:
            result = name
        elif self.kind == StructType.RUST_MUT_REF:
            # if a rust function returns a reference, we just clone it :/
            # It's The Only Way To Be Sure
            result = f'{name}.clone()'

        return f'Box::into_raw(Box::new(borrow_check({result})))'

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

    def wrap_c_value(self):
        return self.type.wrap_c_value(self.name)

def make_safe_call(type, rust_function, args):
    prefix = []
    args_ = []
    postfix = []

    for i, arg in enumerate(args):
        pre, arg_, post = arg.wrap_c_value()
        if pre != '':
            prefix.append(pre)
        args_.append(arg_)
        if post != '':
            postfix.append(post)
    
    entry =  '\n'.join(prefix)
    entry += f'\nlet maybe_panic = panic::catch_unwind(move || {{'
    call = f'''\nlet result = {rust_function}({', '.join(args_)});'''
    call += ('\n' if len(postfix) > 0 else '')
    call += '\n'.join(postfix[::-1])
    call += '\n' + type.unwrap_rust_value('result')
    call = s(call, indent=4)
    exit = '\n}});'
    exit += '\ncheck_panic!(maybe_panic)'

    return entry + call + exit

def javadoc(docs):
    return ('/**\n' + '\n *'.join(docs.split('\n')) + '\n */')

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
                const default_if_err: {self.type.rust} = {self.type.default};
            '''
        )
        result += s(self.body, indent=4)
        result += '\n}\n'
        return result

class StructWrapper(object):
    def __init__(self, module, name, docs=''):
        self.module = module
        self.name = name
        self.c_name = f'{module}_{name}'
        self.members = []
        self.member_docs = []
        self.methods = []
        self.method_names = []
        self.getters = []
        self.type = StructType(module, name)
        self.constructor_ = None
        self.constructor_docs = ''
        self.destructor = Function(void, f'{module}_delete_{name}', [Var(self.type, 'self')],
            'Box::from_raw(self)'
        )
        self.docs = docs

    def constructor(self, rust_method, args, docs=''):
        assert self.constructor_ is None
        self.constructor_docs = docs

        method = f'{self.module}::{self.name}::{rust_method}'

        self.constructor_ = Function(
            self.type,
            self.module + '_new_' + self.name,
            args,
            make_safe_call(self.type, method, args),
            docs=docs
        )

        return self

    def member(self, type, name, docs=''):
        self.members.append(Var(type,name))
        self.member_docs.append(docs)

        pre, arg, post = self.type.mut_ref().wrap_c_value('self')
        arg = '(' + arg + ')'

        getter = Function(type, f"{self.c_name}_get_{name}", [Var(self.type, 'self')],
            pre +
            '\nlet result = ' + type.unwrap_rust_value(arg + '.' + name) + ';\n' +
            post +
            '\nresult',
            docs=docs
        )

        vpre, varg, vpost = type.wrap_c_value(name)

        setter = Function(void, f"{self.c_name}_set_{name}",
            [Var(self.type, 'self'), Var(type,name)],
            pre + vpre +
            f'\n{arg}.{name} = {varg};\n' +
            post + vpost,
            docs=docs
        )
        self.getters.append(getter)
        self.getters.append(setter)

        return self

    def method(self, type, name, args, docs=''):
        # we use the "Universal function call syntax"
        # Type::method(&mut self, arg1, arg2)
        # which is equivalent to:
        # self.method(arg1, arg2)
        method = f'{self.module}::{self.name}::{name}'
        actual_args = [Var(self.type.mut_ref(), 'self')] + args

        self.methods.append(Function(type, f"{self.c_name}_{name}", actual_args,
            make_safe_call(type, method, actual_args), docs=docs
        ))
        self.method_names.append(name)
        return self

    def to_c(self):
        assert self.constructor_ is not None
        definition = 'typedef struct {0.c_name} {0.c_name};\n'.format(self)
        definition += self.constructor_.to_c()
        definition += self.destructor.to_c()
        definition += ''.join(getter.to_c() for getter in self.getters)
        definition += ''.join(method.to_c() for method in self.methods)
        return definition

    def to_swig(self):
        '''Generate a SWIG interface for this struct.'''
        assert self.constructor_ is not None
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
        for method, method_name in zip(self.methods, self.method_names):
            body += Function(method.type, method_name, method.args[1:], docs=method.docs).to_swig()
        for member, member_docs in zip(self.members, self.member_docs):
            body += f'%feature("docstring", "{member_docs}");\n{member.to_swig()};\n'

        body = s(body, indent=4)
        extra = f'%extend {self.c_name} {{\n{body}}}\n'

        return f'{definition}\n{extra}'

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
        body = make_safe_call(type, f'{module}::{name}', args)
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
        return C_HEADER.format(module=self.name) + ''.join(elem.to_c() for elem in self.elements)

    def to_swig(self):
        return SWIG_HEADER.format(module=self.name) + ''.join(elem.to_swig() for elem in self.elements)

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

p = Program('sp')

Apple = p.struct('Apple', docs='High in cyanide.')\
         .constructor('new', [Var(i32, 'phoneme')])\
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