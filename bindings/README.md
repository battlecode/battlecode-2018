# Bindings

This folder contains the bindings generation system for Battlecode 2018.
To build the bindings, run `make`.

## How it works
The script `generate.py` contains a description of the Battlecode API using the `frankenswig` python library. `generate.py` spits out a bunch of files:
- src/bindings.rs: A file that represents a thin C wrapper around a rust API
- c/include/bc.h: A C header file for that wrapper
- c/include/bc.i: A SWIG interface file for that wrapper
- python/battlecode/bc.py: A python file that wraps c/include/bc.h using CFFI (NOT using swig). Note that the python interface does not go through SWIG in order to support using the fast PyPy interpreter.

The `Makefile` then calls into various sub-makefiles and build systems in order to compile all of the bindings systems.

Strictly speaking, the bindings system is written for Linux, and during the official Battlecode competition all player code will be run in one of the official Battlecode docker containers. However, the bindings system also generally works on Windows and Mac; you may just have to futz with the build scripts a little. If you're compiling on Windows you should probably use Cygwin or MinGW.

## Frankenswig
Frankenswig is a simple python 3 library with no dependencies (although to compile the generated code you'll need SWIG). It's essentially just a loose collection of objects with 4 methods: `to_rust`, `to_c`, `to_swig`, and `to_python`. These objects are wrapped with a literate api called `Program`.

### Example

For instance, if I have the following rust code, in the crate `bananas`:

```rust
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

pub struct Banana {
    pub age: u32,
    pub color: Color,
}
impl Banana {
    fn new(age: u32, species: u8) -> Banana { /* ... */ }
    fn is_edible(&self) -> bool { /* ... */ }
}
```

I can bind this code with a python script that looks like so:

```python
from frankenswig import *

p = Program(name='bn', crate='bananas', docs='Helpful tools for working with bananas.')

Color = p.struct('Color')
Color.member(u8.type, 'r')
Color.member(u8.type, 'g')
Color.member(u8.type, 'b')

Banana = p.struct('Banana')
Banana.constructor('new', [Var(u32.type, 'age'), Var(u8.type, 'species')])
Banana.member(u32.type, 'age')
Banana.member(color.type, 'color')
Banana.method(boolean.type, 'is_edible', [])

print(p.to_rust())
print(p.to_c())
print(p.to_swig())
print(p.to_python())
```

This will generate (something like) the following output:

#### Rust
```rust
/// GENERATED RUST, DO NOT EDIT
extern crate bananas;

use bananas as bn;

use std::os::raw::c_char;
use std::cell::RefCell;
use std::ffi::CString;
use std::panic;
use std::ptr;
use std::mem;

// Static error checking

/// This function throws an error at compile time if it isn't safe to return
/// its argument outside of rust.
/// <T: 'static + Send> is read "the type T contains no references and is safe to move between threads".
/// That is a good enough guarantee for us.
fn borrow_check<T: 'static + Send>(val: T) -> T { val }

// Runtime error checking

// see https://github.com/swig/swig/blob/master/Lib/swigerrors.swg
#[repr(i8)]
enum SwigError {
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
}
// We have to pass errors to c somehow :/
thread_local! {
    // this can be replaced with UnsafeCell / pointers and flags
    // if we're really hurting for performance
    static ERROR: RefCell<Option<(SwigError, String)>> = RefCell::new(None);
}
// only usable from rust
fn set_error(code: SwigError, err: String) {
    ERROR.with(move |e| {
        *e.borrow_mut() = Some((code, err));
    });
}

// called from c
#[no_mangle]
pub unsafe extern "C" fn bn_has_err() -> u8 {
    let mut result = 0;
    ERROR.with(|e| {
        if let &Some(..) = &*e.borrow() {
            result = 1;
        }
    });
    result
}

#[no_mangle]
pub unsafe extern "C" fn bn_get_last_err(result: *mut *mut c_char) -> i8 {
    let mut result_code = 0i8;
    ERROR.with(|e| {
        let mut data = None;
        mem::swap(&mut data, &mut *e.borrow_mut());
        if let Some((code, err)) = data {
            result_code = code as i8;
            *result = CString::new(err)
                .map(|r| r.into_raw())
                .unwrap_or_else(|_| CString::new("unknown error").unwrap().into_raw())
        }
    });
    result_code
}
// called from c
#[no_mangle]
pub unsafe extern "C" fn bn_free_err(err: *mut c_char) {
    if err != ptr::null_mut() {
        CString::from_raw(err);
    }
}
// you ever wonder if you're going too deep?
// because I haven't.
macro_rules! check_null {
    ($val:expr, $default:expr) => {
        if $val == ptr::null_mut() {
            set_error(SwigError::NullReference, "self is null".into());
            return $default;
        } else {
            unsafe {
                &mut *$val
            }
        }
    };
}
macro_rules! check_panic {
    ($maybe_panic:expr, $default:expr) => {
        match $maybe_panic {
            Err(err) => {
                let cause = err.downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| err.downcast_ref::<String>().map(|s| s.clone()))
                    .unwrap_or_else(|| "unknown panic, mysterious".to_string());
                set_error(SwigError::Runtime, format!("panic occurred, talk to the devs: {}", cause));
                $default
            },
            Ok(result) => {
                result
            }
        }
    };
}
macro_rules! check_result {
    ($result:expr, $default:expr) => {
        match $result {
            Err(err) => {
                set_error(SwigError::Runtime, format!("{}", err));
                $default
            },
            Ok(result) => {
                result
            }
        }
    };
}

#[no_mangle]
pub extern "C" fn delete_bn_Color(this: *mut bn::Color) -> () {
    const default: () = ();
    let _this = check_null!(this, default);
    unsafe { Box::from_raw(_this); }
}
#[no_mangle]
pub extern "C" fn bn_Color_r_get(this: *mut bn::Color) -> u8 {
    const default: u8 = 0;
    let _this = check_null!(this, default);
    let result = (_this).r;

    result
}
#[no_mangle]
pub extern "C" fn bn_Color_g_get(this: *mut bn::Color) -> u8 {
    const default: u8 = 0;
    let _this = check_null!(this, default);
    let result = (_this).g;

    result
}
#[no_mangle]
pub extern "C" fn bn_Color_b_get(this: *mut bn::Color) -> u8 {
    const default: u8 = 0;
    let _this = check_null!(this, default);
    let result = (_this).b;

    result
}
#[no_mangle]
pub extern "C" fn bn_Color_r_set(this: *mut bn::Color, r: u8) -> () {
    const default: () = ();
    let _this = check_null!(this, default);
    (_this).r = r;

}
#[no_mangle]
pub extern "C" fn bn_Color_g_set(this: *mut bn::Color, g: u8) -> () {
    const default: () = ();
    let _this = check_null!(this, default);
    (_this).g = g;

}
#[no_mangle]
pub extern "C" fn bn_Color_b_set(this: *mut bn::Color, b: u8) -> () {
    const default: () = ();
    let _this = check_null!(this, default);
    (_this).b = b;

}
#[no_mangle]
pub extern "C" fn new_bn_Banana(age: u32, species: u8) -> *mut bn::Banana {
    const default: *mut bn::Banana = 0 as *mut _;

    let maybe_panic = panic::catch_unwind(move || {
        let result = bn::Banana::new(age, species);
        Box::into_raw(Box::new(borrow_check(result)))
    });
    check_panic!(maybe_panic, default)
}
#[no_mangle]
pub extern "C" fn delete_bn_Banana(this: *mut bn::Banana) -> () {
    const default: () = ();
    let _this = check_null!(this, default);
    unsafe { Box::from_raw(_this); }
}
#[no_mangle]
pub extern "C" fn bn_Banana_age_get(this: *mut bn::Banana) -> u32 {
    const default: u32 = 0;
    let _this = check_null!(this, default);
    let result = (_this).age;

    result
}
#[no_mangle]
pub extern "C" fn bn_Banana_age_set(this: *mut bn::Banana, age: u32) -> () {
    const default: () = ();
    let _this = check_null!(this, default);
    (_this).age = age;

}
#[no_mangle]
pub extern "C" fn bn_Banana_is_edible(this: *mut bn::Banana) -> u8 {
    const default: u8 = 0;

    let maybe_panic = panic::catch_unwind(move || {
        let _this = check_null!(this, default);
        let result = bn::Banana::is_edible(_this);
        result as u8
    });
    check_panic!(maybe_panic, default)
}
```

#### C
```c
/// GENERATED C, DO NOT EDIT
#ifndef bn_h_
#define bn_h_
#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
uint8_t bn_has_err();
int8_t bn_get_last_err(char** result);
int8_t bn_free_err(char* err);
typedef struct bn_Color bn_Color;
void delete_bn_Color(bn_Color* this);
uint8_t bn_Color_r_get(bn_Color* this);
uint8_t bn_Color_g_get(bn_Color* this);
uint8_t bn_Color_b_get(bn_Color* this);
void bn_Color_r_set(bn_Color* this, uint8_t r);
void bn_Color_g_set(bn_Color* this, uint8_t g);
void bn_Color_b_set(bn_Color* this, uint8_t b);
typedef struct bn_Banana bn_Banana;
bn_Banana* new_bn_Banana(uint32_t age, uint8_t species);
void delete_bn_Banana(bn_Banana* this);
uint32_t bn_Banana_age_get(bn_Banana* this);
void bn_Banana_age_set(bn_Banana* this, uint32_t age);
uint8_t bn_Banana_is_edible(bn_Banana* this);
#ifdef __cplusplus
}
#endif
#endif // bn_h_
```

#### Swig
```c
%module bn
/// GENERATED SWIG, DO NOT EDIT
%feature("autodoc", "1");
%{
#include "bn.h"

#ifdef __GNUC__
    #define unlikely(expr)  __builtin_expect(!(expr),  0)
#else
    #define unlikely(expr) (expr)
#endif
%}

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
%exception {
    $action
    char *err;
    int8_t code;
    if (unlikely((code = bn_get_last_err(&err)))) {
        SWIG_exception(code, err);
        bn_free_err(err);
    }
}

// We generate code with the prefix "bn_".
// This will strip it out.
%rename("%(strip:[bn_])s") "";

%feature("docstring", "");
typedef struct bn_Color {} bn_Color;
%apply bn_Color* INPUT { bn_Color* a };
%extend bn_Color {
    %feature("docstring", "");
    uint8_t r;
    %feature("docstring", "");
    uint8_t g;
    %feature("docstring", "");
    uint8_t b;
}
%feature("docstring", "");
typedef struct bn_Banana {} bn_Banana;
%apply bn_Banana* INPUT { bn_Banana* a };
%extend bn_Banana {
    %feature("docstring", "");
    bn_Banana(uint32_t age, uint8_t species);
    ~bn_Banana();
    %feature("docstring", "");
    uint8_t is_edible(bn_Banana* this);
    %feature("docstring", "");
    uint32_t age;
}
```

#### Python
```python
"""Helpful tools for working with bananas."""

from ._bn import ffi as _ffi
from ._bn import lib as _lib
import threading
import enum

# might be cheaper to just allocate new strings, TODO benchmark.
def _check_errors():
    if _lib.bn_has_err():
        _lasterror = _ffi.new('char**')
        err = _lib.bn_get_last_err(_lasterror)
        errtext = _ffi.string(_lasterror[0])
        _lib.bn_free_err(_lasterror[0])
        raise Exception(errtext)

class Color(object):
    __slots__ = ['_ptr']
    def __del__(self):
        '''Clean up the object.'''
        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bn_Color(self._ptr)
            _check_errors()
    @property
    def r(self):
        ''''''
        result = _lib.bn_Color_r_get(self._ptr)
        _check_errors()
        return result

    @property
    def g(self):
        ''''''
        result = _lib.bn_Color_g_get(self._ptr)
        _check_errors()
        return result

    @property
    def b(self):
        ''''''
        result = _lib.bn_Color_b_get(self._ptr)
        _check_errors()
        return result

    @r.setter
    def r(self, r):
        ''''''
        result = _lib.bn_Color_r_set(self._ptr, r)
        _check_errors()
        return result

    @g.setter
    def g(self, g):
        ''''''
        result = _lib.bn_Color_g_set(self._ptr, g)
        _check_errors()
        return result

    @b.setter
    def b(self, b):
        ''''''
        result = _lib.bn_Color_b_set(self._ptr, b)
        _check_errors()
        return result



class Banana(object):
    __slots__ = ['_ptr']
    def __init__(self, age, species):
        ''''''
        self._ptr = _lib.new_bn_Banana(age, species)
        _check_errors()

    def __del__(self):
        '''Clean up the object.'''
        if hasattr(self, '_ptr'):
            # if there was an error in the constructor, we'll have no _ptr
            _lib.delete_bn_Banana(self._ptr)
            _check_errors()
    @property
    def age(self):
        ''''''
        result = _lib.bn_Banana_age_get(self._ptr)
        _check_errors()
        return result

    @age.setter
    def age(self, age):
        ''''''
        result = _lib.bn_Banana_age_set(self._ptr, age)
        _check_errors()
        return result

    def is_edible(self):
        ''''''
        result = _lib.bn_Banana_is_edible(self._ptr)
        _check_errors()
        result = bool(result)
        return result

```

Hopefully you're getting a sense of how the system works. The bindings generated in each language try to be relatively idiomatic; swig does most of the work for us for the non-python languages, and we do a little bit of extra work in Python to be compatible with PyPy. We're basically like an even hackier version of SWIG.

### Ownership discipline
You can see that the rust code treats objects passed through the language barrier as being owned by the language runtime that is calling into rust. In addition, objects passed out of rust are forbidden from holding references, and must be threadsafe. This is because we can't trust other language runtimes to respect rust's invariants around borrows and such. If you really need to bind an object graph, use `Arc<Mutex<T>>`.

It is, of course, still possible for there to be an ABI mismatch between the generated rust and the generated c header, if there is a bug in frankenswig. For this reason it's a good idea to write tests for languages that aren't rust.