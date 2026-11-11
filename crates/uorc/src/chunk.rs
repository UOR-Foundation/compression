//! Content-defined units: how the encoder finds repeats at any offset, and
//! how the decoder finds the bytes a reference frame names.
//!
//! Every frame's output is cut into units at anchors that depend on content
//! alone ([`units`]): a 64-bit gear hash rolls over the bytes, and a unit ends
//! where its top [`ANCHOR_BITS`] bits are zero (at least [`MIN_UNIT`] and at
//! most [`MAX_UNIT`] bytes after the unit starts). The cut depends only on the
//! bytes since the unit's start, so cutting restarts identically at any unit
//! boundary, and two copies of the same content fall into the same units a
//! few anchors after they begin, wherever they sit in the stream.
//!
//! A reference frame ([`crate::archive::reference_archive`]) names a run of
//! consecutive units of one earlier frame by the run's κ-label
//! ([`crate::archive::frame_label`]). A whole frame is one such run, so the
//! v0.3 reference frames (identical aligned frames) are the case of a run
//! that covers its frame. The decoder cuts each frame's output the same way
//! and labels only the runs whose length a reference names ([`runs`]), so
//! finding them costs a gear hash over the output and one label per
//! candidate.
//!
//! The encoder ([`Framer`]) keeps the units of the frames it has written in
//! an index by κ-label. Input is cut into [`crate::FRAME`]-byte frames as
//! before; where a run of at least [`MIN_RUN`] bytes repeats units of an
//! earlier frame outside the window of the frame being cut, the frame before
//! it ends there and the run becomes a reference frame. A repeat inside the
//! window is left to the ncm match model, which codes it in a few bytes per
//! kilobyte without changing the framing of what follows, unless it repeats
//! a whole frame. An input with no such run is framed exactly as it was
//! without this, so its stream is unchanged byte for byte.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{archive, FRAME};

/// Fewest bytes in a unit (a frame's last unit may be shorter).
pub const MIN_UNIT: usize = 4 << 10;
/// Most bytes in a unit: past this many the unit ends whatever the content.
pub const MAX_UNIT: usize = 128 << 10;
/// Zero top bits of the gear hash that make an anchor: units average about
/// `MIN_UNIT + 2^ANCHOR_BITS` bytes.
pub const ANCHOR_BITS: u32 = 14;
/// Fewest bytes a repeated run must hold to become a reference frame. A
/// shorter repeat stays in its frame, where the ncm match model codes it.
pub const MIN_RUN: usize = 64 << 10;
/// Units the encoder's index holds (about 20 GiB of input at the mean unit
/// size); past that the oldest frames' units are forgotten, so the index
/// stays bounded whatever the input length.
pub const INDEX_UNITS: usize = 1 << 20;
/// Earlier occurrences of one unit the encoder tries to extend into a run.
const OCCURRENCES: usize = 4;

/// The gear table: 256 fixed pseudo-random words (SplitMix64 from 0).
const GEAR: [u64; 256] = {
    let mut table = [0u64; 256];
    let mut state = 0u64;
    let mut i = 0;
    while i < 256 {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        table[i] = z ^ (z >> 31);
        i += 1;
    }
    table
};

/// Length of the unit that starts `data`, and whether it ended at an anchor
/// or at [`MAX_UNIT`] (`true`) rather than at the end of `data`.
fn next_unit(data: &[u8]) -> (usize, bool) {
    if data.len() <= MIN_UNIT {
        return (data.len(), false);
    }
    let limit = data.len().min(MAX_UNIT);
    let mut h = 0u64;
    for (i, &b) in data[MIN_UNIT..limit].iter().enumerate() {
        h = (h << 1).wrapping_add(GEAR[b as usize]);
        if h >> (64 - ANCHOR_BITS) == 0 {
            return (MIN_UNIT + i + 1, true);
        }
    }
    (limit, limit == MAX_UNIT)
}

/// Where the units of `data` end, in order; the last is `data.len()`.
pub fn units(data: &[u8]) -> Vec<usize> {
    let mut ends = Vec::new();
    let mut at = 0;
    while at < data.len() {
        at += next_unit(&data[at..]).0;
        ends.push(at);
    }
    ends
}

/// The runs of consecutive units of `frame` whose length is in `wanted` and
/// whose κ-label is one of the labels wanted at that length, each once:
/// what the decoder keeps for the reference frames still to come. A label
/// found is not looked for again, and a length with no label left is skipped.
pub fn runs(
    frame: &[u8],
    mut wanted: HashMap<u64, HashSet<[u8; 32]>>,
) -> Vec<([u8; 32], std::ops::Range<usize>)> {
    let mut found = Vec::new();
    let Some(longest) = wanted.keys().copied().max() else {
        return found;
    };
    let mut bounds = vec![0];
    bounds.extend(units(frame));
    for (i, &start) in bounds.iter().enumerate() {
        for &end in &bounds[i + 1..] {
            let len = (end - start) as u64;
            if len > longest {
                break;
            }
            let Some(labels) = wanted.get_mut(&len) else {
                continue;
            };
            // A v0.3 stream's reference frames name whole frames, labelled
            // over the v0.3 library.
            let named: &[[u8; 32]] = if start == 0 && end == frame.len() {
                &archive::frame_labels(frame)
            } else {
                &[archive::frame_label(&frame[start..end])]
            };
            for &label in named {
                if labels.remove(&label) {
                    found.push((label, start..end));
                }
            }
            if labels.is_empty() {
                wanted.remove(&len);
            }
        }
    }
    found
}

/// The κ-labels of the units of the frames the encoder has written, by
/// label, for finding repeated runs (see the module documentation).
#[derive(Default)]
struct Index {
    /// Unit labels of each indexed frame, oldest first; frame `first + i`
    /// is `frames[i]`.
    frames: VecDeque<Vec<[u8; 32]>>,
    first: u64,
    units: usize,
    /// The latest occurrences of each label: (frame, unit), newest last.
    at: HashMap<[u8; 32], Vec<(u64, u32)>>,
}

impl Index {
    fn add(&mut self, labels: Vec<[u8; 32]>) {
        let frame = self.first + self.frames.len() as u64;
        for (j, label) in labels.iter().enumerate() {
            let seen = self.at.entry(*label).or_default();
            if seen.len() == OCCURRENCES {
                seen.remove(0);
            }
            seen.push((frame, j as u32));
        }
        self.units += labels.len();
        self.frames.push_back(labels);
        while self.units > INDEX_UNITS && self.frames.len() > 1 {
            let old = self.frames.pop_front().expect("an indexed frame");
            for label in &old {
                if let Some(seen) = self.at.get_mut(label) {
                    seen.retain(|&(f, _)| f != self.first);
                    if seen.is_empty() {
                        self.at.remove(label);
                    }
                }
            }
            self.units -= old.len();
            self.first += 1;
        }
    }

    /// The longest run of `labels` from its start that repeats consecutive
    /// units of one indexed frame: how many units it covers, whether it ran
    /// to the end of `labels` with the indexed frame continuing, and whether
    /// that frame is before `near` (outside the window of the frame being cut).
    fn run(&self, labels: &[[u8; 32]], near: u64) -> Option<Run> {
        let seen = self.at.get(labels.first()?)?;
        let mut best: Option<Run> = None;
        for &(frame, unit) in seen.iter().rev() {
            let stored = &self.frames[(frame - self.first) as usize][unit as usize..];
            let units = labels
                .iter()
                .zip(stored)
                .take_while(|(a, b)| a == b)
                .count();
            let run = Run {
                units,
                open: units == labels.len() && units < stored.len(),
                far: frame < near,
            };
            // A far source is preferred: a near one is in the window already.
            if best.is_none_or(|b| (run.far, run.units) > (b.far, b.units)) {
                best = Some(run);
            }
        }
        best
    }
}

/// A repeated run the index found (see [`Index::run`]).
#[derive(Clone, Copy)]
struct Run {
    units: usize,
    open: bool,
    far: bool,
}

/// How a stream's objects are split into the pieces each frame holds (see
/// the crate documentation): the first piece of every frame after the first
/// continues the last object of the frame before.
enum Pieces {
    /// One object of unknown length: every frame is one piece.
    Plain,
    /// Objects of the given lengths, the one in progress having `left` bytes
    /// still to place.
    Objects { lengths: VecDeque<u64>, left: u64 },
}

impl Pieces {
    /// The pieces of the next frame, of `n` bytes; `end` when no bytes follow.
    /// An object that starts exactly where a full frame ends goes to the next
    /// frame; one that starts at the end of a shorter last frame stays in it.
    fn cut(&mut self, n: u64, first: bool, end: bool) -> Vec<u64> {
        let Pieces::Objects { lengths, left } = self else {
            return vec![n];
        };
        let mut pieces = Vec::new();
        let mut used = 0;
        if !first {
            let take = (*left).min(n);
            pieces.push(take);
            *left -= take;
            used = take;
        }
        while *left == 0 {
            let Some(&len) = lengths.front() else { break };
            if used == n && !(end && n < FRAME as u64) {
                break;
            }
            lengths.pop_front();
            let take = len.min(n - used);
            pieces.push(take);
            used += take;
            *left = len - take;
        }
        pieces
    }

    fn done(&self) -> bool {
        match self {
            Pieces::Plain => true,
            Pieces::Objects { lengths, left } => lengths.is_empty() && *left == 0,
        }
    }
}

/// A frame the [`Framer`] cut: its bytes, the pieces of objects it holds,
/// and, for a repeated run, the κ-label of the earlier bytes it repeats.
pub struct Frame {
    /// The frame's input bytes.
    pub bytes: Vec<u8>,
    /// The pieces of objects it holds, which sum to its length.
    pub pieces: Vec<u64>,
    /// For a repeated run, its κ-label, which an earlier frame's run has.
    pub reference: Option<[u8; 32]>,
    /// How many frames before it the frame reads as its window (0 for a
    /// reference frame and for a frame that starts a block).
    pub window: usize,
}

/// Cuts a stream's input into frames (see the module documentation). The
/// frames depend on the input alone, not on how it is pushed.
pub struct Framer {
    /// Input from the start of the next frame.
    pending: Vec<u8>,
    /// Units of `pending` from its start: where each ends and its label.
    units: Vec<(usize, [u8; 32])>,
    /// Whether the last of `units` ended at an anchor (or at [`MAX_UNIT`]),
    /// rather than where `pending` or the frame ran out.
    settled: bool,
    index: Index,
    pieces: Pieces,
    frames: u64,
    /// Frames since the current block began, and coded (not reference)
    /// frames among them: a block holds [`crate::BLOCK`] coded frames, so
    /// reference frames do not shorten the stretch a core decodes.
    in_block: usize,
    coded_in_block: usize,
    /// The last frame was a reference frame. The next coded frame starts a
    /// block: a reference may wait for a frame another core is still
    /// decoding, and nothing after it should wait with it.
    after_reference: bool,
}

impl Framer {
    fn with(pieces: Pieces) -> Self {
        Framer {
            pending: Vec::new(),
            units: Vec::new(),
            settled: true,
            index: Index::default(),
            pieces,
            frames: 0,
            in_block: 0,
            coded_in_block: 0,
            after_reference: false,
        }
    }

    /// A plain byte stream: one object.
    pub fn plain() -> Self {
        Self::with(Pieces::Plain)
    }

    /// A stream of objects of `lengths`, whose bytes are pushed back to back.
    pub fn objects(lengths: &[u64]) -> Self {
        Self::with(Pieces::Objects {
            lengths: lengths.iter().copied().collect(),
            left: 0,
        })
    }

    /// Add input; returns the frames it completes.
    pub fn push(&mut self, input: &[u8]) -> Vec<Frame> {
        self.pending.extend_from_slice(input);
        let mut out = Vec::new();
        // More than a frame is pending, so the next frame's bytes are known
        // and so is that more input follows them.
        while self.pending.len() > FRAME {
            out.push(self.step(false));
        }
        out
    }

    /// End of input: the remaining frames (one empty frame for empty input).
    pub fn finish(&mut self) -> Vec<Frame> {
        let mut out = Vec::new();
        while !self.pending.is_empty() {
            out.push(self.step(true));
        }
        if self.frames == 0 || !self.pieces.done() {
            out.push(self.emit(0, None, true));
        }
        out
    }

    /// Cut units over the first frame's worth of pending input.
    fn refresh(&mut self) {
        let window = self.pending.len().min(FRAME);
        if !self.settled {
            self.units.pop();
            self.settled = true;
        }
        let mut at = self.units.last().map_or(0, |u| u.0);
        while at < window {
            let (len, anchored) = next_unit(&self.pending[at..window]);
            let label = archive::frame_label(&self.pending[at..at + len]);
            at += len;
            self.units.push((at, label));
            self.settled = anchored;
        }
    }

    /// Cut the next frame. `end`: no input follows `pending`.
    fn step(&mut self, end: bool) -> Frame {
        self.refresh();
        let more = !end || self.pending.len() > FRAME;
        // A last unit cut where the frame runs out still names exactly its
        // bytes, and a frame that takes it ends there, so it matches too.
        let labels: Vec<[u8; 32]> = self.units.iter().map(|u| u.1).collect();
        let firm = labels.len();
        // The frames the next frame reads as its window: the ncm match model
        // already codes a repeat of them, so only a repeat of that whole
        // frame becomes a reference there, which leaves the framing of what
        // follows unchanged.
        let near = self.frames - self.reach() as u64;
        for i in 0..firm {
            let Some(run) = self.index.run(&labels[i..firm], near) else {
                continue;
            };
            let n = run.units;
            if n == 0 {
                continue;
            }
            let start = if i == 0 { 0 } else { self.units[i - 1].0 };
            let end_at = self.units[i + n - 1].0;
            let whole = i == 0 && n == labels.len();
            if !run.far && !whole {
                continue;
            }
            let long = end_at - start >= MIN_RUN;
            if i > 0 && (long || (run.open && more)) {
                // The frame before the run ends where the run begins.
                return self.literal(i, end && !more);
            }
            if i == 0 && long {
                let label = if n == 1 {
                    labels[0]
                } else {
                    archive::frame_label(&self.pending[..end_at])
                };
                return self.take(n, Some(label), end && !more);
            }
        }
        let all = self.units.len();
        self.literal(all, end && !more)
    }

    fn literal(&mut self, units: usize, last: bool) -> Frame {
        self.take(units, None, last)
    }

    /// Emit the first `units` units of `pending` as the next frame.
    fn take(&mut self, units: usize, reference: Option<[u8; 32]>, last: bool) -> Frame {
        let n = self.units[units - 1].0;
        let labels: Vec<[u8; 32]> = self.units.drain(..units).map(|u| u.1).collect();
        for unit in &mut self.units {
            unit.0 -= n;
        }
        if self.units.is_empty() {
            self.settled = true;
        }
        self.index.add(labels);
        let last = last && n == self.pending.len();
        self.emit(n, reference, last)
    }

    /// How many frames before it the next coded frame reads: up to
    /// [`archive::WINDOW`] frames of its block, none when it starts one.
    fn reach(&self) -> usize {
        if self.coded_in_block == crate::BLOCK || self.after_reference {
            0
        } else {
            self.in_block.min(archive::WINDOW)
        }
    }

    fn emit(&mut self, n: usize, reference: Option<[u8; 32]>, last: bool) -> Frame {
        let bytes: Vec<u8> = self.pending.drain(..n).collect();
        let pieces = self.pieces.cut(n as u64, self.frames == 0, last);
        let window = match reference {
            Some(_) => 0,
            None => {
                let window = self.reach();
                if self.coded_in_block == crate::BLOCK || self.after_reference {
                    self.in_block = 0;
                    self.coded_in_block = 0;
                }
                self.coded_in_block += 1;
                window
            }
        };
        self.in_block += 1;
        self.frames += 1;
        self.after_reference = reference.is_some();
        Frame {
            bytes,
            pieces,
            reference,
            window,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noise(n: usize, seed: u64) -> Vec<u8> {
        let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                (x >> 24) as u8
            })
            .collect()
    }

    /// Cutting restarts identically at any unit boundary, and a copy of
    /// content at another offset falls into the same units after an anchor.
    #[test]
    fn units_resynchronize_at_any_offset() {
        let data = noise(2 << 20, 1);
        let ends = units(&data);
        assert_eq!(*ends.last().expect("units"), data.len());
        let sizes: Vec<usize> = std::iter::once(ends[0])
            .chain(ends.windows(2).map(|w| w[1] - w[0]))
            .collect();
        assert!(sizes[..sizes.len() - 1]
            .iter()
            .all(|&s| (MIN_UNIT..=MAX_UNIT).contains(&s)));
        let mean = data.len() / ends.len();
        assert!((12 << 10..=32 << 10).contains(&mean), "{mean}");
        let from = units(&data[ends[3]..]);
        assert_eq!(
            from,
            ends[4..].iter().map(|e| e - ends[3]).collect::<Vec<_>>()
        );
        let shifted = [&noise(12_345, 2)[..], &data].concat();
        let moved: HashSet<usize> = units(&shifted).into_iter().map(|e| e - 12_345).collect();
        let shared = ends.iter().filter(|e| moved.contains(e)).count();
        assert!(shared + 3 >= ends.len(), "{shared} of {}", ends.len());
    }

    fn frames_of(framer: &mut Framer, input: &[u8], step: usize) -> Vec<Frame> {
        let mut out = Vec::new();
        for piece in input.chunks(step.max(1)) {
            out.extend(framer.push(piece));
        }
        out.extend(framer.finish());
        out
    }

    /// Input without repeats is framed exactly as fixed frames; a run
    /// repeated at an unaligned offset becomes reference frames, and the
    /// frames do not depend on how the input is pushed.
    #[test]
    fn repeats_at_any_offset_become_references() {
        let plain = noise(FRAME * 2 + 1000, 3);
        let frames = frames_of(&mut Framer::plain(), &plain, FRAME);
        let sizes: Vec<usize> = frames.iter().map(|f| f.bytes.len()).collect();
        assert_eq!(sizes, vec![FRAME, FRAME, 1000]);
        let windows: Vec<usize> = frames.iter().map(|f| f.window).collect();
        assert_eq!(windows, vec![0, 1, 2]);
        assert!(frames.iter().all(|f| f.reference.is_none()));

        // The copy starts in frame 3, whose window (frames 1 and 2) does not
        // reach the first.
        let a = noise(3 << 20, 4);
        let input = [&a[..], &noise(3 * FRAME + 777, 5), &a, &noise(5_000, 6)].concat();
        let frames = frames_of(&mut Framer::plain(), &input, 1 << 20);
        let refs: Vec<&Frame> = frames.iter().filter(|f| f.reference.is_some()).collect();
        let repeated: usize = refs.iter().map(|f| f.bytes.len()).sum();
        assert!(repeated + 2 * MAX_UNIT > a.len(), "{repeated}");
        for f in &refs {
            assert_eq!(f.reference, Some(archive::frame_label(&f.bytes)));
            assert_eq!(f.window, 0);
        }
        // Blocks hold four coded frames; reference frames do not count.
        let coded: Vec<usize> = frames
            .iter()
            .filter(|f| f.reference.is_none())
            .map(|f| f.window)
            .collect();
        assert_eq!(&coded[..5], &[0, 1, 2, 2, 0]);
        let joined: Vec<u8> = frames.iter().flat_map(|f| f.bytes.clone()).collect();
        assert!(joined == input);
        for step in [1 << 16, 3 << 20, input.len()] {
            let again = frames_of(&mut Framer::plain(), &input, step);
            assert!(again
                .iter()
                .map(|f| (&f.bytes, &f.pieces, f.reference))
                .eq(frames.iter().map(|f| (&f.bytes, &f.pieces, f.reference))));
        }
    }

    /// The decoder finds every run a reference names in the frame that holds it.
    #[test]
    fn runs_find_named_bytes() {
        let frame = noise(1 << 20, 7);
        let ends = units(&frame);
        let run = ends[2]..ends[9];
        let label = archive::frame_label(&frame[run.clone()]);
        let whole = archive::frame_label(&frame);
        let mut wanted: HashMap<u64, HashSet<[u8; 32]>> = HashMap::new();
        wanted.entry(run.len() as u64).or_default().insert(label);
        wanted.entry(frame.len() as u64).or_default().insert(whole);
        wanted.entry(999).or_default().insert([7; 32]);
        let mut found = runs(&frame, wanted);
        found.sort_by_key(|f| f.1.start);
        assert_eq!(found, vec![(whole, 0..frame.len()), (label, run)]);
    }
}
