//! WebAssembly exports of the uorc client, with no unsafe code and no bindgen.
//!
//! The page asks for an input buffer of a given size, writes the bytes into
//! wasm memory at the returned offset, calls an operation, then reads the
//! result back through `output_ptr`/`output_len`.
//!
//! Large files stream through the frame operations: the page slices the file
//! into [`frame_size`] pieces (or, to decompress, into the archives that
//! [`frame_length`] delimits) and converts one at a time after
//! [`window_reset`], so wasm memory stays bounded by a few frames (the one in
//! flight, the window it reads, and any frame a later reference frame names)
//! whatever the file size. The frame operations run the library's
//! [`uorc::FrameEncoder`] and [`uorc::FrameDecoder`], so the page writes and
//! reads exactly the streams the native client does.

use std::cell::RefCell;

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static ORIGINAL: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    /// The stream being compressed frame by frame since [`window_reset`].
    static ENCODER: RefCell<uorc::FrameEncoder> = RefCell::new(uorc::FrameEncoder::new());
    /// The stream being decompressed archive by archive since [`window_reset`].
    static DECODER: RefCell<uorc::FrameDecoder> =
        RefCell::new(uorc::FrameDecoder::new(uorc::DEFAULT_DECODE_WORK));
}

/// Start a new stream for the frame operations.
#[no_mangle]
pub extern "C" fn window_reset() {
    ENCODER.with(|encoder| *encoder.borrow_mut() = uorc::FrameEncoder::new());
    DECODER.with(|decoder| {
        *decoder.borrow_mut() = uorc::FrameDecoder::new(uorc::DEFAULT_DECODE_WORK);
    });
}

/// Operation succeeded; the result is in the output buffer.
pub const OK: i32 = 0;
/// See [`uorc::Error::Budget`].
pub const BUDGET: i32 = 1;
/// See [`uorc::Error::Overflow`].
pub const OVERFLOW: i32 = 2;
/// See [`uorc::Error::Decode`].
pub const DECODE: i32 = 3;
/// See [`uorc::Error::Mismatch`].
pub const MISMATCH: i32 = 4;

fn resize(buffer: &RefCell<Vec<u8>>, len: usize) -> *mut u8 {
    let mut buffer = buffer.borrow_mut();
    buffer.clear();
    buffer.resize(len, 0);
    buffer.as_mut_ptr()
}

/// Resize the input buffer to `len` bytes and return its address in memory.
#[no_mangle]
pub extern "C" fn input_buffer(len: usize) -> *mut u8 {
    INPUT.with(|input| resize(input, len))
}

/// Resize the original buffer (the bytes [`verify`] compares against) to
/// `len` bytes and return its address in memory.
#[no_mangle]
pub extern "C" fn original_buffer(len: usize) -> *mut u8 {
    ORIGINAL.with(|original| resize(original, len))
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
        Err(uorc::Error::Mismatch) => (MISMATCH, Vec::new()),
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

/// Compress the input buffer as the next frame (exactly [`frame_size`]
/// bytes, except the last) of the stream since [`window_reset`]: the archives
/// written back to back are the stream the native client writes.
#[no_mangle]
pub extern "C" fn compress_frame(steps: u64) -> i32 {
    finish(
        INPUT.with(|input| {
            ENCODER.with(|encoder| encoder.borrow_mut().push(&input.borrow(), steps))
        }),
    )
}

/// Whether the archive whose header is in the input buffer may be a
/// reference frame (it has a reference frame's length), so the page should
/// [`note_frame`] it whole.
#[no_mangle]
pub extern "C" fn may_be_reference() -> i32 {
    INPUT.with(|input| i32::from(uorc::may_be_reference(&input.borrow())))
}

/// Note the whole archive in the input buffer before decompressing the stream
/// since [`window_reset`]: a reference frame names an earlier frame the
/// decoder must keep. The page reads the headers first and notes every
/// archive that [`may_be_reference`].
#[no_mangle]
pub extern "C" fn note_frame() {
    INPUT.with(|input| DECODER.with(|decoder| decoder.borrow_mut().note(&input.borrow())));
}

/// Decompress the input buffer as the next archive of the stream since
/// [`window_reset`].
#[no_mangle]
pub extern "C" fn decompress_frame() -> i32 {
    finish(INPUT.with(|input| DECODER.with(|decoder| decoder.borrow_mut().push(&input.borrow()))))
}

/// Input bytes per frame.
#[no_mangle]
pub extern "C" fn frame_size() -> usize {
    uorc::FRAME
}

/// Bytes of header the page should read before a first [`frame_length`].
#[no_mangle]
pub extern "C" fn header_size() -> usize {
    uorc::HEADER_HINT
}

/// Total length of the archive whose first bytes are in the input buffer;
/// -2 when more bytes are needed to see its header, -1 when they are not a
/// UORC archive header.
#[no_mangle]
pub extern "C" fn frame_length() -> f64 {
    INPUT.with(|input| match uorc::frame_length(&input.borrow()) {
        Ok(Some(n)) => n as f64,
        Ok(None) => -2.0,
        Err(_) => -1.0,
    })
}

/// Read the frame header and integrity seal of every archive in the input
/// stream without decoding; the output is a JSON array of descriptions.
#[no_mangle]
pub extern "C" fn inspect() -> i32 {
    finish(INPUT.with(|input| {
        let input = input.borrow();
        let all = uorc::frames(&input)?
            .into_iter()
            .map(|a| uorc::inspect(a).map(|info| info.to_json()))
            .collect::<Result<Vec<_>, _>>()?;
        if all.is_empty() {
            return Err(uorc::Error::Decode);
        }
        Ok(format!("[{}]", all.join(",")).into_bytes())
    }))
}

/// Decode the input (one archive or several back to back) in memory and check
/// it; with `compare` nonzero, also check that it reproduces the original
/// buffer. The output is a JSON array describing each archive.
#[no_mangle]
pub extern "C" fn verify(compare: i32) -> i32 {
    finish(INPUT.with(|input| {
        ORIGINAL.with(|original| {
            let original = original.borrow();
            uorc::verify(
                &input.borrow(),
                (compare != 0).then_some(original.as_slice()),
                uorc::DEFAULT_DECODE_WORK,
            )
            .map(|infos| {
                let all: Vec<String> = infos.iter().map(uorc::Info::to_json).collect();
                format!("[{}]", all.join(",")).into_bytes()
            })
        })
    }))
}

/// Entries cross the wasm boundary as `u32 count` then, per entry,
/// `u8 kind` (0 file, 1 executable, 2 directory), `u32 pathLength`, path,
/// `u32 dataLength`, data; integers little-endian.
fn read_entries(bytes: &[u8]) -> Result<Vec<uorc::tree::Entry>, uorc::Error> {
    use uorc::tree::{Entry, Kind};
    struct Cursor<'a>(&'a [u8]);
    impl<'a> Cursor<'a> {
        fn take(&mut self, n: usize) -> Result<&'a [u8], uorc::Error> {
            if n > self.0.len() {
                return Err(uorc::Error::Decode);
            }
            let (head, tail) = self.0.split_at(n);
            self.0 = tail;
            Ok(head)
        }
        fn word(&mut self) -> Result<usize, uorc::Error> {
            Ok(u32::from_le_bytes(self.take(4)?.try_into().expect("4 bytes")) as usize)
        }
    }
    let mut c = Cursor(bytes);
    let count = c.word()?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let kind = match c.take(1)?[0] {
            0 => Kind::File,
            1 => Kind::Executable,
            2 => Kind::Directory,
            _ => return Err(uorc::Error::Decode),
        };
        let len = c.word()?;
        let path = String::from_utf8(c.take(len)?.to_vec()).map_err(|_| uorc::Error::Decode)?;
        let len = c.word()?;
        let data = c.take(len)?.to_vec();
        entries.push(Entry { path, kind, data });
    }
    if !c.0.is_empty() {
        return Err(uorc::Error::Decode);
    }
    Ok(entries)
}

fn write_entries(entries: &[uorc::tree::Entry]) -> Vec<u8> {
    use uorc::tree::Kind;
    let mut out = (entries.len() as u32).to_le_bytes().to_vec();
    for entry in entries {
        out.push(match entry.kind {
            Kind::File => 0,
            Kind::Executable => 1,
            Kind::Directory => 2,
        });
        out.extend((entry.path.len() as u32).to_le_bytes());
        out.extend(entry.path.as_bytes());
        out.extend((entry.data.len() as u32).to_le_bytes());
        out.extend(&entry.data);
    }
    out
}

/// Compress the entries in the input buffer (see [`read_entries`]) into one
/// archive of ordered objects.
#[no_mangle]
pub extern "C" fn compress_tree(steps: u64) -> i32 {
    finish(INPUT.with(|input| {
        read_entries(&input.borrow()).and_then(|entries| uorc::compress_tree(&entries, steps))
    }))
}

/// Restore the entries a tree archive in the input buffer stores; the output
/// is in the [`read_entries`] layout. A one-object archive is not a tree.
#[no_mangle]
pub extern "C" fn decompress_tree() -> i32 {
    finish(INPUT.with(|input| {
        uorc::decompress_tree(&input.borrow(), uorc::DEFAULT_DECODE_WORK)
            .map(|entries| write_entries(&entries))
    }))
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

        let (code, json) = run(&archive, || inspect());
        assert_eq!(code, OK);
        let info = uorc::inspect(&archive).expect("library inspect");
        assert_eq!(json, format!("[{}]", info.to_json()).into_bytes());
        ORIGINAL.with(|original| *original.borrow_mut() = input.to_vec());
        let (code, json) = run(&archive, || verify(1));
        assert_eq!(
            (code, json),
            (OK, format!("[{}]", info.to_json()).into_bytes())
        );
        ORIGINAL.with(|original| original.borrow_mut()[0] ^= 1);
        assert_eq!(run(&archive, || verify(1)).0, MISMATCH);
    }

    /// The frame exports stream as the page drives them: a repeated frame is
    /// a reference frame, noted in a header pass, and decodes.
    #[test]
    fn frame_exports_stream_reference_frames() {
        let (x, y): (Vec<u8>, Vec<u8>) = (
            b"frame x ".iter().copied().cycle().take(3_000).collect(),
            (0..3_000u32).map(|i| (i * 7 % 251) as u8).collect(),
        );
        window_reset();
        let archives: Vec<Vec<u8>> = [&x, &y, &x]
            .iter()
            .map(|frame| {
                let (code, archive) = run(frame, || compress_frame(0));
                assert_eq!(code, OK);
                archive
            })
            .collect();
        let info = uorc::inspect(&archives[2]).expect("inspect");
        assert_eq!(info.representation, uorc::Representation::Reference);
        window_reset();
        for archive in &archives {
            if run(archive, || may_be_reference()).0 == 1 {
                run(archive, || {
                    note_frame();
                    OK
                });
            }
        }
        for (archive, frame) in archives.iter().zip([&x, &y, &x]) {
            assert_eq!(run(archive, || decompress_frame()), (OK, frame.to_vec()));
        }
    }
}
