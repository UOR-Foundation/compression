//! WebAssembly exports of the uorc client, with no unsafe code and no bindgen.
//!
//! The page asks for an input buffer of a given size, writes the bytes into
//! wasm memory at the returned offset, calls an operation, then reads the
//! result back through `output_ptr`/`output_len`.

use std::cell::RefCell;

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Operation succeeded; the result is in the output buffer.
pub const OK: i32 = 0;
/// See [`uorc::Error::Budget`].
pub const BUDGET: i32 = 1;
/// See [`uorc::Error::Overflow`].
pub const OVERFLOW: i32 = 2;
/// See [`uorc::Error::Decode`].
pub const DECODE: i32 = 3;

/// Resize the input buffer to `len` bytes and return its address in memory.
#[no_mangle]
pub extern "C" fn input_buffer(len: usize) -> *mut u8 {
    INPUT.with(|input| {
        let mut input = input.borrow_mut();
        input.clear();
        input.resize(len, 0);
        input.as_mut_ptr()
    })
}

/// Address of the last operation's output.
#[no_mangle]
pub extern "C" fn output_ptr() -> *const u8 {
    OUTPUT.with(|output| output.borrow().as_ptr())
}

/// Length of the last operation's output.
#[no_mangle]
pub extern "C" fn output_len() -> usize {
    OUTPUT.with(|output| output.borrow().len())
}

fn finish(result: Result<Vec<u8>, uorc::Error>) -> i32 {
    let (code, bytes) = match result {
        Ok(bytes) => (OK, bytes),
        Err(uorc::Error::Budget) => (BUDGET, Vec::new()),
        Err(uorc::Error::Overflow) => (OVERFLOW, Vec::new()),
        Err(uorc::Error::Decode) => (DECODE, Vec::new()),
    };
    OUTPUT.with(|output| *output.borrow_mut() = bytes);
    code
}

/// Compress the input buffer with a search budget of `steps`.
#[no_mangle]
pub extern "C" fn compress(steps: u64) -> i32 {
    finish(INPUT.with(|input| uorc::compress(&input.borrow(), steps)))
}

/// Decompress the input buffer.
#[no_mangle]
pub extern "C" fn decompress() -> i32 {
    finish(INPUT.with(|input| uorc::decompress(&input.borrow(), uorc::DEFAULT_DECODE_WORK)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &[u8], op: impl FnOnce() -> i32) -> (i32, Vec<u8>) {
        INPUT.with(|buffer| *buffer.borrow_mut() = input.to_vec());
        let code = op();
        (code, OUTPUT.with(|output| output.borrow().clone()))
    }

    /// The page's exports run the library's compress and decompress unchanged.
    #[test]
    fn wasm_exports_match_the_library_uc_api_01() {
        let input = [0u8, 0xff, 0xc3, 0x28, b'a', b'b', b'a', b'b', 0];
        let (code, archive) = run(&input, || compress(0));
        assert_eq!(code, OK);
        assert_eq!(archive, uorc::compress(&input, 0).expect("library archive"));
        assert_eq!(run(&archive, || decompress()), (OK, input.to_vec()));
        assert_eq!(
            run(b"not an archive", || decompress()),
            (DECODE, Vec::new())
        );
    }
}
