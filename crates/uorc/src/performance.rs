//! The performance vocabulary of the Prism model (`Uorc.Performance`), as the
//! native client computes it: the step measure, the codec point and the
//! choice rules (least in bytes then steps, dominance, consolidation).
//!
//! A **step** is one model input evaluated for one coded bit. It is exact and
//! deterministic, and it is read from a frame's header alone: a raw frame
//! costs one step per output byte (the copy), a reference frame costs none
//! (its bytes were produced once already), and an ncm frame costs eight bits
//! times the inputs its profile word keeps, per output byte, with or without
//! the x86 filter. The constants are the model's; `tests/performance.rs`
//! reads them from the module text.

use crate::frame::{Info, Representation};
use crate::ncm;
use sha2::{Digest, Sha256};

/// Coded bits per output byte.
pub const BITS_PER_BYTE: u64 = 8;
/// Inputs every ncm profile evaluates beside its contexts: the match model's
/// two and the bias.
pub const FIXED_INPUTS: u32 = 3;
/// Inputs each optional model (short match, repeat distance) adds.
pub const FEATURE_INPUTS: u32 = 2;
/// Context bits of a profile word.
pub const CONTEXT_BITS: u32 = 17;
/// The most inputs any profile evaluates: 17 contexts, the fixed 3, two features.
pub const MAX_INPUTS: u32 = CONTEXT_BITS + FIXED_INPUTS + 2 * FEATURE_INPUTS;
/// The bound of `Uorc.Performance.decodeStepsBound`: no frame costs more
/// steps per output byte than this (the full model).
pub const MAX_STEPS_PER_BYTE: u64 = BITS_PER_BYTE * MAX_INPUTS as u64;
/// The bit of a codec point's profile word that says the frame is coded over
/// the x86 filter (the strategy table's encoding: bits 0-19 the ncm profile,
/// bit 31 the filter). The filter changes the bytes a frame codes, not its steps.
pub const POINT_X86: u32 = 1 << 31;
/// Domain tag of a codec point's identity.
pub const CODEC_POINT_TAG: &[u8] = b"uorc/codec-point/1";

/// A declared decode host (`Uorc.Performance.Host`): the bytes one frame's
/// decoder may hold there, counted as that host holds them
/// ([`Host::footprint`]). A facet that sets decoder memory (the tables'
/// bits) steps only while the frame still fits the host the archive is
/// written for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Host {
    /// Bytes one frame's decoder may hold.
    pub memory: u64,
    /// The host keeps a spare set of the coder's tables between frames (the
    /// page's WebAssembly worker does, so the next frame allocates nothing)
    /// and a copy of the frame's output.
    pub keeps_tables: bool,
    /// Frames the host decodes at once, each holding its own decoder (one
    /// for a page worker: the page's pool gives each frame its own worker).
    pub workers: u64,
    /// Threads one frame's decoder runs on: a frame is cut into streams
    /// ([`crate::ncm::streams`]) only for a host with more than one. Unlike
    /// memory this boundary is not hard: a host with fewer threads still
    /// decodes the frame, with more work, and one with more is not fully
    /// used.
    pub threads: u64,
}

impl Host {
    /// Bytes this host holds decoding a frame of `n` bytes after `history`
    /// bytes (with `warm`, a model state) under `profile`:
    /// [`crate::ncm::decoder_bytes`], and on a host that keeps them, the
    /// spare tables ([`crate::ncm::table_bytes`]) and the output copy.
    pub fn footprint(self, n: u64, history: u64, warm: bool, profile: u32) -> u64 {
        let decoder = crate::ncm::decoder_bytes(n, history, warm, profile);
        if self.keeps_tables {
            decoder
                .saturating_add(crate::ncm::table_bytes(n, history, warm, profile))
                .saturating_add(n)
        } else {
            decoder
        }
    }

    /// Warmed models (a model state replayed through the coder, kept so a
    /// later frame continuing the same state replays nothing) this host keeps
    /// beside the decoder of a frame of `n` bytes coded after a state of
    /// `state` bytes under `profile`: as many as the memory the frame's
    /// decoders leave fits, each holding [`crate::ncm::warmed_bytes`]; none
    /// where nothing is left (`Uorc.Performance::warmedKept`). Keeping one changes
    /// the time a decode takes, never its bytes.
    pub fn warmed_kept(self, n: u64, state: u64, profile: u32) -> u64 {
        let decoders = self
            .footprint(n, state, true, profile)
            .saturating_mul(self.workers.max(1));
        let each = crate::ncm::warmed_bytes(n, state, profile).max(1);
        self.memory.saturating_sub(decoders) / each
    }

    /// Whether this host holds the frame's decoder.
    pub fn holds(self, n: u64, history: u64, warm: bool, profile: u32) -> bool {
        self.footprint(n, history, warm, profile)
            .saturating_mul(self.workers.max(1))
            <= self.memory
    }
}

/// Page workers per GiB of device memory: the page's host rule
/// (`Uorc.Page::pageWorker`, field `workersPerGiB`; held equal by the test
/// `page_worker_is_the_pages_host_rule`). Stated by the page, not yet
/// derived from the device.
pub const WORKERS_PER_GIB: u64 = 2;

/// The product's smallest declared decode host, and the default: one of the
/// page's WebAssembly workers, whose share of the device is
/// 1 GiB / [`WORKERS_PER_GIB`] under the page's host rule
/// (`Uorc.Page::pageWorker`). Every archive the client writes by default
/// decodes there.
pub const PAGE_WORKER: Host = Host {
    memory: (1 << 30) / WORKERS_PER_GIB,
    keeps_tables: true,
    workers: 1,
    // a WebAssembly worker runs one thread (`Uorc.Page::pageWorker`, field
    // `frameThreads`; `Client.hostThreads`)
    threads: 1,
};

static DECLARED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Declare a native decoder of `memory` bytes as the host the archives this
/// process writes are for (the CLI's `--host-memory`). Archives for a larger host may not decode on the
/// page's worker.
pub fn declare_host(memory: u64) {
    DECLARED.store(memory, std::sync::atomic::Ordering::Relaxed);
}

/// The decode host archives are written for: [`PAGE_WORKER`] unless the
/// process declared a native decoder of some bytes ([`declare_host`]).
/// Its frame decoder runs on one thread: in v0.5 a frame is one stream
/// (the frame word's streams and epoch fields are reserved).
pub fn declared_host() -> Host {
    let threads = PAGE_WORKER.threads;
    match DECLARED.load(std::sync::atomic::Ordering::Relaxed) {
        0 => Host {
            threads,
            ..PAGE_WORKER
        },
        memory => Host {
            memory,
            keeps_tables: false,
            workers: 1,
            threads,
        },
    }
}

/// Model inputs an ncm frame coded under `profile` evaluates per bit.
pub fn inputs(profile: u32) -> u32 {
    (profile & ((1 << CONTEXT_BITS) - 1)).count_ones()
        + FIXED_INPUTS
        + FEATURE_INPUTS * u32::from(profile & ncm::F_SHORT != 0)
        + FEATURE_INPUTS * u32::from(profile & ncm::F_REP != 0)
}

/// Decode steps of an ncm trial coding `n` modelled bytes under `profile`
/// (`Client.trialSteps`): 8 bits, each evaluating [`inputs`] model inputs.
/// A state frame models n + |S| bytes (the reference measure).
pub fn trial_steps(profile: u32, n: u64) -> u64 {
    BITS_PER_BYTE * u64::from(inputs(profile)) * n
}

/// Steps to decode `n` output bytes of a frame of this representation and
/// profile word; `None` for a representation the model states no measure for
/// (the ulz decoders kept for earlier archives, and searched programs).
pub fn steps(representation: Representation, profile: Option<u32>, n: u64) -> Option<u64> {
    match representation {
        Representation::Raw => Some(n),
        Representation::Reference => Some(0),
        Representation::Ncm | Representation::NcmX86 => {
            Some(BITS_PER_BYTE * u64::from(inputs(profile.unwrap_or(ncm::V03))) * n)
        }
        Representation::Ulz | Representation::UlzKernel | Representation::Program => None,
    }
}

/// [`steps`] of the frame a header describes.
pub fn frame_steps(info: &Info) -> Option<u64> {
    steps(info.representation, info.ncm_profile, info.output_len)
}

/// A point of the codec space: which decoder basis, profile word, consolidated
/// model state and lane count code a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecPoint {
    /// The basis identity the frame names (the export's library), zeros when
    /// the frame is self-contained.
    pub basis: [u8; 32],
    /// The ncm profile word, with [`POINT_X86`] set when the frame is coded
    /// over the x86 filter; 0 when the frame applies no ncm decoder.
    pub profile: u32,
    /// κ of the consolidated model state the frame starts from, zeros for none.
    pub state: [u8; 32],
    /// Lanes the frame is coded in (1 for a single stream).
    pub lanes: u32,
}

impl CodecPoint {
    /// The point of the frame a header describes: its basis, profile word,
    /// the κ of the state it continues (zero for none) and one lane.
    pub fn of(info: &Info) -> CodecPoint {
        CodecPoint {
            basis: info.basis.unwrap_or([0; 32]),
            profile: info.ncm_profile.unwrap_or(0)
                | if info.representation == Representation::NcmX86 {
                    POINT_X86
                } else {
                    0
                },
            state: info.state.unwrap_or([0; 32]),
            lanes: 1,
        }
    }

    /// κ of the point: SHA-256 over the tag, the basis, the profile word
    /// (little-endian), the state and the lane count (little-endian).
    pub fn kappa(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(CODEC_POINT_TAG);
        h.update(self.basis);
        h.update(self.profile.to_le_bytes());
        h.update(self.state);
        h.update(self.lanes.to_le_bytes());
        h.finalize().into()
    }
}

/// κ of a corpus: SHA-256 over `uorc/corpus/1` and, for each object in order,
/// its name length (u32 LE), name, SHA-256 and length (u64 LE).
pub fn corpus_kappa<'a>(objects: impl IntoIterator<Item = (&'a str, [u8; 32], u64)>) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"uorc/corpus/1");
    for (name, digest, len) in objects {
        h.update((name.len() as u32).to_le_bytes());
        h.update(name.as_bytes());
        h.update(digest);
        h.update(len.to_le_bytes());
    }
    h.finalize().into()
}

/// A coded trial as the model's choice rules see it: its archive bytes and
/// its decode steps (`Uorc.Performance.Trial.coded`, without the defect).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trial {
    /// Archive bytes.
    pub bytes: u64,
    /// Decode steps.
    pub steps: u64,
}

impl Trial {
    /// `cost`: the potential the engine lowers, bytes plus steps.
    pub fn cost(self) -> u64 {
        self.bytes + self.steps
    }
}

/// `gainOf`: how far `candidate` dominates `incumbent` (no larger in bytes
/// and in steps): the drop in cost, 0 when it does not dominate.
pub fn gain(candidate: Trial, incumbent: Trial) -> u64 {
    if candidate.bytes <= incumbent.bytes && candidate.steps <= incumbent.steps {
        incumbent.cost() - candidate.cost()
    } else {
        0
    }
}

/// `lexGap`: 0 exactly when `a` is no later than `c` in the order "fewer
/// bytes, then fewer steps".
pub fn lex_gap(a: Trial, c: Trial) -> u64 {
    a.bytes.saturating_sub(c.bytes)
        + a.steps
            .saturating_sub(c.steps)
            .min(1)
            .saturating_sub(c.bytes.saturating_sub(a.bytes))
}

/// `pick`: the kept trial after `head` is considered. A failed trial (`None`)
/// changes nothing; a coded one replaces the kept trial when it has fewer
/// bytes, or the same bytes and fewer steps.
pub fn pick(head: Option<Trial>, kept: Trial) -> Trial {
    match head {
        Some(h) if h.bytes < kept.bytes || (h.bytes == kept.bytes && h.steps < kept.steps) => h,
        _ => kept,
    }
}

/// `chosen`: the trial kept from the raw frame and the scheduled trials
/// (`chosenIsLeast`: least in bytes, then steps; `leastIsUndominated`).
pub fn chosen(raw: Trial, trials: &[Option<Trial>]) -> Trial {
    trials
        .iter()
        .rev()
        .fold(raw, |kept, &head| pick(head, kept))
}

/// `consolidate`: the incumbent after `candidate` is decided: the candidate
/// when it dominates with a positive gain, else the incumbent
/// (`consolidationSpendsItsGain`: the cost drops by exactly the gain).
pub fn consolidate(candidate: Trial, incumbent: Trial) -> Trial {
    if gain(candidate, incumbent) > 0 {
        candidate
    } else {
        incumbent
    }
}

#[cfg(test)]
mod tests {
    /// The value of `key` in the body of the definition `name` of a model
    /// module (`key := value`, or the whole literal body when `key` is empty).
    fn stated(module: &str, name: &str, key: &str) -> u64 {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(module);
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let decl = text
            .split("definition{")
            .find(|d| d.contains(&format!("name: {name},")))
            .unwrap_or_else(|| panic!("{module} defines no {name}"));
        let body = &decl[decl.find("body: ").expect("a body") + 6..];
        let value = if key.is_empty() {
            body
        } else {
            &body[body.find(&format!("{key} := ")).expect("the field") + key.len() + 4..]
        };
        let end = value
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(value.len());
        value[..end].parse().expect("a natural")
    }

    #[test]
    fn page_worker_is_the_pages_host_rule() {
        let per_gib = stated("../../src/Uorc/Page.lex.tex", "pageWorker", "workersPerGiB");
        assert_eq!(super::WORKERS_PER_GIB, per_gib);
        let client = stated("../../client/src/Uorc/Client.lex.tex", "hostMemory", "");
        assert_eq!(super::PAGE_WORKER.memory, client);
        let gate = stated("../../src/Uorc/Page.lex.tex", "workerMemory", "threshold");
        assert_eq!(super::PAGE_WORKER.memory, gate << 20);
        let threads = stated("../../src/Uorc/Page.lex.tex", "pageWorker", "frameThreads");
        assert_eq!(super::PAGE_WORKER.threads, threads);
        let client = stated("../../client/src/Uorc/Client.lex.tex", "hostThreads", "");
        assert_eq!(super::PAGE_WORKER.threads, client);
    }

    use super::*;

    /// The kept warmed models are the page's rule (`Uorc.Performance::warmedKept`):
    /// the host's memory less the frame's decoders, over one warmed model.
    #[test]
    fn warmed_models_kept_are_the_hosts_room() {
        let state = crate::states::STATE_1.len() as u64;
        for tb in [0, 22, 24, 25, 26] {
            let profile = ncm::with_table_bits(ncm::FULL, tb);
            for host in [
                PAGE_WORKER,
                Host {
                    memory: 1 << 33,
                    keeps_tables: false,
                    workers: 1,
                    threads: 1,
                },
            ] {
                let n = 4096;
                let kept = host.warmed_kept(n, state, profile);
                let decoders = host.footprint(n, state, true, profile) * host.workers;
                let each = ncm::warmed_bytes(n, state, profile);
                assert_eq!(kept, host.memory.saturating_sub(decoders) / each);
                eprintln!(
                    "tb {tb} host {} MiB: decoder {} MiB, warmed {} MiB, kept {kept}",
                    host.memory >> 20,
                    decoders >> 20,
                    each >> 20
                );
            }
        }
        // a host the frame's decoder fills keeps none
        let full = Host {
            memory: 1 << 20,
            keeps_tables: false,
            workers: 1,
            threads: 1,
        };
        assert_eq!(full.warmed_kept(4096, state, ncm::FULL), 0);
    }

    #[test]
    fn steps_follow_the_profile_word() {
        assert_eq!(inputs(ncm::V03), 20);
        assert_eq!(inputs(ncm::FULL), MAX_INPUTS);
        assert_eq!(steps(Representation::Raw, None, 10), Some(10));
        assert_eq!(steps(Representation::Reference, None, 10), Some(0));
        assert_eq!(steps(Representation::Ncm, Some(ncm::FULL), 10), Some(1920));
        assert_eq!(
            steps(Representation::NcmX86, Some(ncm::FULL), 10),
            Some(10 * MAX_STEPS_PER_BYTE)
        );
        for profile in [ncm::V03, ncm::FULL, 1, 0x7ffff] {
            assert!(inputs(profile) <= MAX_INPUTS);
        }
    }

    #[test]
    fn a_codec_point_is_named_by_all_its_coordinates() {
        let p = CodecPoint {
            basis: [1; 32],
            profile: ncm::FULL,
            state: [0; 32],
            lanes: 1,
        };
        assert_ne!(p.kappa(), CodecPoint { lanes: 2, ..p }.kappa());
        assert_ne!(
            p.kappa(),
            CodecPoint {
                state: [2; 32],
                ..p
            }
            .kappa()
        );
        assert_ne!(
            p.kappa(),
            CodecPoint {
                profile: ncm::V03,
                ..p
            }
            .kappa()
        );
        assert_ne!(
            p.kappa(),
            CodecPoint {
                profile: ncm::FULL | POINT_X86,
                ..p
            }
            .kappa()
        );
        assert_eq!(p.kappa(), p.kappa());
    }
}
