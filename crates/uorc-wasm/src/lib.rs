//! WebAssembly exports of the uorc client, with no unsafe code and no bindgen.
//!
//! The page asks for an input buffer of a given size, writes the bytes into
//! wasm memory at the returned offset, calls an operation, then reads the
//! result back through `output_ptr`/`output_len`.
//!
//! Large files stream through the frame operations: the page slices the file
//! into [`frame_size`] pieces (or, to decompress, into the archives that
//! [`frame_length`] delimits) and converts one at a time, so wasm memory stays
//! bounded by one frame whatever the file size.

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

/// Compress the input buffer as a single frame (at most [`frame_size`] bytes).
#[no_mangle]
pub extern "C" fn compress_frame(steps: u64) -> i32 {
    finish(INPUT.with(|input| uorc::compress_frame(&input.borrow(), steps)))
}

/// Decompress the input buffer as a single archive.
#[no_mangle]
pub extern "C" fn decompress_frame() -> i32 {
    finish(INPUT.with(|input| uorc::decompress_frame(&input.borrow(), uorc::DEFAULT_DECODE_WORK)))
}

/// Input bytes per frame.
#[no_mangle]
pub extern "C" fn frame_size() -> usize {
    uorc::FRAME
}

/// Bytes of header the page should read before [`frame_length`].
#[no_mangle]
pub extern "C" fn header_size() -> usize {
    uorc::archive::HEADER_MAX
}

/// Total length of the archive whose first bytes are in the input buffer, or
/// -1 when they are not a UORC archive header.
#[no_mangle]
pub extern "C" fn frame_length() -> f64 {
    INPUT.with(|input| uorc::archive::frame_length(&input.borrow()).map_or(-1.0, |n| n as f64))
}
