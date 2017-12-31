//! The C bindings for battlecode.
//! 
//! Battlecode has no global state.
//! However, its data structures are not thread safe.
//! If you need to use multiple threads you should either lock a mutex on
//! a single bc_t, or create one bc_t (and child datastructures) for each thread.

// use nice c-style names
#![allow(non_camel_case_types, unused_variables, non_upper_case_globals)]

extern crate battlecode_engine;
extern crate failure;

mod conversions;
#[macro_use]
mod macros;

use std::os::raw::{c_char as char};
use std::ptr;
use std::mem;
use std::panic;
use std::ffi::CString;
use std::clone::Clone;

use battlecode_engine as eng;

struct bc_t_ {
    /// Text describing an error, if one happened.
    /// This could be changed to an enum later if we want more specificity.
    error: Option<String>
}

/// "Global" state of battlecode.
/// Not thread safe.
/// Required for all method calls.
/// Used for error checking.
#[repr(C)]
pub struct bc_t(bc_t_);

// note: cheddar interprets wrapper structs as opaque;
// i.e. the above compiles to `typedef struct bc_t bc_t`.

/// Initialize a battlecode instance.
#[no_mangle]
pub unsafe extern "C" fn bc_init() -> *mut bc_t {
    Box::into_raw(Box::new(bc_t(bc_t_ {
        error: None
    })))
}

/// Shut down a battlecode instance.
/// You don't have to call this, it's fine to leak a bc_t (although if you leak a lot you may
/// have problems.)
/// Does *not* free extra resources (i.e. game worlds), you need to do that yourself first.
#[no_mangle]
pub unsafe extern "C" fn bc_shutdown(bc: *mut bc_t) {
    if bc != ptr::null_mut() {
        Box::from_raw(bc);
    }
}

/// Check if there was an error.
#[no_mangle]
pub unsafe extern "C" fn bc_has_error(bc: *mut bc_t) -> u8 {
    if bc == ptr::null_mut() {
        return 0;
    }
    ((*bc).0.error != None) as u8
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
    mem::swap(&mut result, &mut (*bc).0.error);

    let result = match result {
        Some(result) => result,
        None => return ptr::null_mut()
    };
    // convert to cstring
    CString::new(result).map(|r| r.into_raw()).unwrap_or(ptr::null_mut())
}

/// Free an error.
/// Guaranteed to not create a new error.
#[no_mangle]
pub unsafe extern "C" fn bc_free_error(bc: *mut bc_t, error: *mut char) {
    if error != ptr::null_mut() {
        // convert to cstring
        CString::from_raw(error);
    }
}


/// A game world.
/// Opaque; you have to use accessor functions to learn about it.
/// Don't try to access the rust struct values from C, rust structs
/// are organized differently from c structs.
#[repr(C)]
pub struct bc_game_world_t(eng::world::GameWorld);

/// Allocate a game world.
#[no_mangle]
pub unsafe extern "C" fn bc_new_game_world(bc: *mut bc_t) -> *mut bc_game_world_t {
    // This macro is from macros.rs
    handle_errors!{bc () -> *mut bc_game_world_t [0] {
        let result = Box::into_raw(Box::new(
            bc_game_world_t(eng::world::GameWorld::test_world())));
        Ok(result)
    }}
}

/// Free a game world.
#[no_mangle]
pub unsafe extern "C" fn bc_free_game_world(bc: *mut bc_t, game_world: *mut bc_game_world_t) {
    handle_errors!{bc (game_world) {
        Box::from_raw(game_world as *mut _);
        Ok(())
    }}
}

/// Clone a game world.
#[no_mangle]
pub unsafe extern "C" fn bc_clone_game_world(bc: *mut bc_t,
                                             game_world: *mut bc_game_world_t) -> *mut bc_game_world_t {
    // This macro is from macros.rs
    handle_errors!{bc (game_world) -> *mut bc_game_world_t [0] {
        let result = Box::into_raw(Box::new(
            bc_game_world_t((*game_world).0.clone())
        ));
        Ok(result)
    }}
}



/// Get the current round of a game world.
#[no_mangle]
pub unsafe extern "C" fn bc_get_round(bc: *mut bc_t, game_world: *mut bc_game_world_t) -> u32 {
    handle_errors!{bc (game_world) -> u32 [0] {
        Ok((*game_world).0.round())
    }}
}

// Note: we redefine location and direction types
// so that they can be cheaply manipulated in other languages,
// while not polluting our main codebase with lots of repr(c) annotations.
// See the `conversions` model for the code that converts between types;
// in general, you can just call `.into()` to convert to the type you want.

/// A location on the map.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct bc_map_location_t {
    x: i32,
    y: i32
}

// slighly inconsistent naming required to get reasonable names from the C side.
/// A direction.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum bc_direction {
    north = 0,
    northeast = 1,
    east = 2,
    southeast = 3,
    south = 4,
    southwest = 5,
    west = 6,
    northwest = 7,
    center = 8
}
pub type bc_direction_t = bc_direction;

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
