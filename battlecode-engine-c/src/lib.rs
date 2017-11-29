#[no_mangle]
pub unsafe extern "C" fn dummy(arg1: u8, arg2: *mut f64) -> u32 {
    arg1 as u32 + 5
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
