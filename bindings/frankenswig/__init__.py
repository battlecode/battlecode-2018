'''This module can be used to easily generate interfaces to Rust from many other languages,
by generating a thin SWIG wrapper, as well as a CFFI wrapper for python.

TODO: %newobject, makefiles, enums, results, javadocs

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

from .helpers import *
from .type import *
from .function import FunctionWrapper
from .struct import StructWrapper
from .enums import EnumWrapper, CEnumWrapper

RUST_HEADER = '''/// GENERATED RUST, DO NOT EDIT
extern crate {crate};

use {crate} as {module};

use std::os::raw::c_char;
use std::cell::RefCell;
use std::ffi::{{CStr, CString}};
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
    // this can be replaced with UnsafeCell / pointers and flags
    // if we're really hurting for performance
    static ERROR: RefCell<Option<(SwigError, String)>> = RefCell::new(None);
}}
// only usable from rust
fn set_error(code: SwigError, err: String) {{
    ERROR.with(move |e| {{
        *e.borrow_mut() = Some((code, err));
    }});
}}

// called from c
#[no_mangle]
pub unsafe extern "C" fn {module}_has_err() -> u8 {{
    let mut result = 0;
    ERROR.with(|e| {{
        if let &Some(..) = &*e.borrow() {{
            result = 1;
        }}
    }});
    result
}}

#[no_mangle]
pub unsafe extern "C" fn {module}_get_last_err(result: *mut *mut c_char) -> i8 {{
    let mut result_code = 0i8;
    ERROR.with(|e| {{
        let mut data = None;
        mem::swap(&mut data, &mut *e.borrow_mut());
        if let Some((code, err)) = data {{
            result_code = code as i8;
            *result = CString::new(err)
                .map(|r| r.into_raw())
                .unwrap_or_else(|_| CString::new("unknown error").unwrap().into_raw())
        }}
    }});
    result_code
}}
// called from c
#[no_mangle]
pub unsafe extern "C" fn {module}_free_string(err: *mut c_char) {{
    if err != ptr::null_mut() {{
        CString::from_raw(err);
    }}
}}
// you ever wonder if you're going too deep?
// because I haven't.
macro_rules! check_null {{
    ($val:expr, $default:expr) => {{
        if $val == ptr::null_mut() {{
            set_error(SwigError::NullReference, "self is null".into());
            return $default;
        }} else {{
            unsafe {{
                &mut *$val
            }}
        }}
    }};
}}
macro_rules! check_panic {{
    ($maybe_panic:expr, $default:expr) => {{
        match $maybe_panic {{
            Err(err) => {{
                let cause = err.downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| err.downcast_ref::<String>().map(|s| s.clone()))
                    .unwrap_or_else(|| "unknown panic, mysterious".to_string());
                set_error(SwigError::Runtime, format!("panic occurred, talk to the devs: {{}}", cause));
                $default
            }},
            Ok(result) => {{
                result
            }}
        }}
    }};
}}
macro_rules! check_result {{
    ($result:expr, $default:expr) => {{
        match $result {{
            Err(err) => {{
                set_error(SwigError::Runtime, format!("{{}}", err));
                $default
            }},
            Ok(result) => {{
                result
            }}
        }}
    }};
}}
'''
RUST_FOOTER = ''

C_HEADER = '''/// GENERATED C, DO NOT EDIT
#ifndef {module}_h_
#define {module}_h_
#ifdef __cplusplus
extern "C" {{
#endif

#include <stdint.h>
uint8_t {module}_has_err();
int8_t {module}_get_last_err(char** result);
int8_t {module}_free_string(char* err);
'''

C_FOOTER = '''#ifdef __cplusplus
}}
#endif
#endif // {module}_h_
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
// good enums
%include "enums.swg"

// This code is inserted around every method call.
%exception {{
    $action
    char *err;
    int8_t code;
    if (unlikely((code = {module}_get_last_err(&err)))) {{
        SWIG_exception(code, err);
        {module}_free_string(err);
    }}
}}

// We generate code with the prefix "{module}_".
// This will strip it out.
%rename("%(strip:[{module}_])s") "";

'''
SWIG_FOOTER = ''

PYTHON_HEADER = '''"""{docs}"""

from ._{module} import ffi as _ffi
from ._{module} import lib as _lib
import threading
import enum

# might be cheaper to just allocate new strings, TODO benchmark.
def _check_errors():
    if _lib.{module}_has_err():
        _lasterror = _ffi.new('char**')
        err = _lib.{module}_get_last_err(_lasterror)
        errtext = _ffi.string(_lasterror[0])
        _lib.{module}_free_string(_lasterror[0])
        raise Exception(errtext)

'''
PYTHON_FOOTER = ''

class TypedefWrapper(object):
    def __init__(self, module, rust_name, c_type):
        self.type = Type(f'{module}::{rust_name}', c_type.swig, c_type.python, c_type.default)
    
    to_rust = to_c = to_swig = to_python = lambda self: ''

class Program(object):
    def __init__(self, module, crate, docs=''):
        self.module = module
        self.crate = crate
        self.docs = docs
        self.elements = []

        # maintaining the "thing.type" idiom
        self.string = namedtuple('String', ['type'])(StringType(self.module))
        self.strref = namedtuple('StrRef', ['type'])(StrRefType(self.module))

    def add(self, elem):
        return self

    def format(self, header):
        return header.format(crate=self.crate, module=self.module, docs=self.docs)

    def to_rust(self):
        return self.format(RUST_HEADER)\
            + ''.join(elem.to_rust() for elem in self.elements)\
            + self.format(RUST_FOOTER)

    def to_c(self):
        return self.format(C_HEADER)\
            + ''.join(elem.to_c() for elem in self.elements)\
            + self.format(C_FOOTER)

    def to_swig(self):
        return self.format(SWIG_HEADER)\
            + ''.join(elem.to_swig() for elem in self.elements)\
            + self.format(SWIG_FOOTER)
    
    def to_python(self):
        return self.format(PYTHON_HEADER)\
            + '\n'.join(elem.to_python() for elem in self.elements)\
            + self.format(PYTHON_FOOTER)

    def struct(self, *args, **kwargs):
        result = StructWrapper(self.module, *args, **kwargs)
        self.elements.append(result)
        return result

    def function(self, *args, **kwargs):
        result = FunctionWrapper(self.module, *args, **kwargs)
        self.elements.append(result)
        return result

    def typedef(self, rust_name, c_type):
        result = TypedefWrapper(self.module, rust_name, c_type)
        self.elements.append(result)
        return result

    def enum(self, *args, **kwargs):
        result = EnumWrapper(self.module, *args, **kwargs)
        self.elements.append(result)
        return result

    def c_enum(self, *args, **kwargs):
        result = CEnumWrapper(self.module, *args, **kwargs)
        self.elements.append(result)
        return result

