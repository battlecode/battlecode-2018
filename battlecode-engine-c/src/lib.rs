//! The C bindings for battlecode.
//! 
//! Battlecode has no global state.
//! However, its data structures are not thread safe.
//! If you need to use multiple threads you should either lock a mutex on
//! a single bc_t, or create one bc_t (and child datastructures) for each thread.

// use nice c-style names
#![allow(non_camel_case_types, unused_variables)]

extern crate battlecode_engine;
#[macro_use]
extern crate failure;

use std::os::raw::{c_void as void, c_char as char};
use std::ptr;
use std::mem;
use std::panic;
use std::ffi::CString;

use battlecode_engine as eng;

/// "Global" state of battlecode.
/// Not thread safe.
/// Required for all method calls.
/// Used for error checking.
pub struct bc_t {
    /// Text describing an error, if one happened.
    /// This could be changed to an enum later if we want more specificity.
    error: Option<String>
}

#[no_mangle]
pub unsafe extern "C" fn bc_init() -> *mut bc_t {
    Box::into_raw(Box::new(bc_t {
        error: None
    }))
}

/// Check if there was an error.
#[no_mangle]
pub unsafe extern "C" fn bc_has_error(bc: *mut bc_t) -> u8 {
    if bc == ptr::null_mut() {
        return 0;
    }
    ((*bc).error != None) as u8
}

/// If there was an error, extract it, and return it as a string.
/// Otherwise, return NULL.
/// 
/// That string should be freed with bc_free_error (not normal free()).
#[no_mangle]
pub unsafe extern "C" fn bc_extract_error(bc: *mut bc_t) -> *mut char {
    if bc_has_error(bc) == 0 {
        return ptr::null_mut();
    }

    // extract error
    let mut result = None;
    mem::swap(&mut result, &mut (*bc).error);

    let result = match result {
        Some(result) => result,
        None => return ptr::null_mut()
    };
    // convert to cstring
    CString::new(result).map(|r| r.into_raw()).unwrap_or(ptr::null_mut())
}

/// Free an error.
#[no_mangle]
pub unsafe extern "C" fn bc_free_error(bc: *mut bc_t, error: *mut char) {
    if error != ptr::null_mut() {
        // convert to cstring
        CString::from_raw(error);
    }
}

/// With those functions out of the way, everything else can use this helper macro.
/// It adds error checking and panic handling to a function.
/// 
/// $bc: *mut bc_t
/// $ret: the function's return type
/// $default: the value to return if there is an error
/// $body: the body of the function. Should return Result<$ret, failure::Error>.
macro_rules! handle_errors {
    ($bc:ident $body:block) => {
        handle_errors!{$bc (()) [{}] $body}
    };
    ($bc:ident ($ret:ty) [$default:expr] $body:block) => {
        // Check for null.
        if $bc == ptr::null_mut() {
            return $default;
        }
        // It's not safe to unwind into c code, so we have to add a landing pad here.
        let result: std::thread::Result<Result<$ret, failure::Error>> = panic::catch_unwind(|| {
            // invoke body.
            $body
        });
        // check for errors.
        let cause = match result {
            // no error; early return.
            Ok(Ok(result)) => return result,
            // logic error
            Ok(Err(err)) => format!("{:?}", err),
            // caught panic
            Err(pan) => pan.downcast_ref::<&str>().unwrap_or(&"unknown panic").to_string()
        };
        (*$bc).error = Some(cause);
        $default
    };
}

/// A game world.
/// Opaque; you have to use accessor functions to learn about it.
pub type bc_game_world_t = void;

/// Allocate a game world.
#[no_mangle]
pub unsafe extern "C" fn bc_new_game_world(bc: *mut bc_t) -> *mut bc_game_world_t {
    handle_errors!{bc (*mut bc_game_world_t) [ptr::null_mut()] {
        let result = Box::into_raw(Box::new(eng::world::GameWorld::new()));
        Ok(result as *mut bc_game_world_t)
    }}
}

/// Free a game world.
#[no_mangle]
pub unsafe extern "C" fn bc_free_game_world(bc: *mut bc_t, game_world: *mut bc_game_world_t) {
    handle_errors!{bc {
        if game_world == ptr::null_mut() {
            return Err(format_err!("null game world"))
        }
        Box::from_raw(game_world);
        Ok(())
    }}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_handling() {
        unsafe {
            let bc = bc_init();
            assert_eq!(bc_has_error(bc), 0);
            bc_free_game_world(bc, ptr::null_mut());
            assert_eq!(bc_has_error(bc), 1);
            let err = bc_extract_error(bc);
            assert!(err != ptr::null_mut());
            bc_free_error(bc, err);
            assert_eq!(bc_has_error(bc), 0);
        }
    }
}
