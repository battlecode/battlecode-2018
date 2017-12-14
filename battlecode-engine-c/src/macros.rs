//! Mad macro magic.

/// A helper function to make defining FFI functions easier, if a bit messy-looking.
/// It adds error checking and panic handling to a function.
/// 
/// syntax:
/// 
/// handle_errors { bc (p1, p2, p3) -> type [default] {
///     // do stuff
///     return bananas;
/// }}
/// 
/// bc is a *mut bc_t
/// (p1,p2,p3) are pointers to check for being non-null
/// type is the return type of the body
/// default is the value to return if an error is encountered
macro_rules! handle_errors {
    ($bc:ident $body:block) => {
        handle_errors!{$bc () -> ()[()] $body}
    };
    ($bc:ident ($($pointer:expr),*) $body:block) => {
        handle_errors!{$bc ($($pointer),*) -> ()[()] $body}
    };
    ($bc:ident ($($pointer:expr),*) -> $ret:ty [$default:expr] $body:block) => {
        // Check for null.
        if $bc == ptr::null_mut() {
            return $default as $ret;
        }
        $(
            // Check for null.
            if $pointer == ptr::null_mut() {
                (*$bc).0.error = Some(concat!(stringify!($pointer), " is null!").to_string());
                return $default as $ret;
            }
        )*
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
        (*$bc).0.error = Some(cause);
        $default as $ret
    };
}