//! The strategy portfolio (`Uorc.Client.portfolio`): how the encoder picks
//! the coding of a frame, after the schedule engine of afflom/rule30
//! (`Engine.lex.tex`) and the PrismPM factory.
//!
//! A strategy is a point of the codec space: a transform (none, or the x86
//! branch filter), an ncm model profile, and a model state (none, or one of
//! the κ-named states of the basis, [`crate::states`]). A frame's family
//! comes from cheap statistics of its bytes ([`family`]). Every (family,
//! strategy) pair carries a bounded integer weight in 1..=16; the schedule of
//! a family is every strategy ordered by weight (ties by portfolio order), so
//! no strategy is ever unreachable. The encoder codes the first
//! [`budget`] strategies of the schedule and keeps the smallest archive (raw
//! when that is smaller): the weights order the work, and the measured sizes
//! decide.
//!
//! The strategies and weights are data the model reads (`Client.compress
//! strategies weights states ..`, [`strategy_bytes`], [`weight_bytes`]), so
//! a new table needs no new export, and every table is safe: an entry whose
//! profile is not a profile word or whose state the basis lacks yields no
//! candidate, and every candidate decodes to the frame.
//!
//! [`WEIGHTS`] is learned by the rule30 rule ([`learn`]: +2 for the strategy
//! that coded a frame smallest, -1 for every other, bounded to 1..=16) over
//! the measured outcomes in `portfolio-outcomes.tsv`, starting from 8; the
//! test `weights_are_learned_from_the_outcomes` repeats that learning. The
//! output is a function of the input and these tables alone, whatever
//! `--threads` is.

use crate::ncm;

/// A way to code a frame: over the x86 filter or not, the ncm profile, and
/// the model state the frame continues (0 for none, `i` for state `i`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Strategy {
    /// Coded over the x86 branch filter.
    pub x86: bool,
    /// The ncm model profile.
    pub profile: u32,
    /// The κ-named model state (1-based; 0 is none).
    pub state: u8,
}

/// Every ncm feature but the match-state mixer selection: the profile that
/// short inputs favour.
pub const MID: u32 = ncm::FULL & !ncm::F_SEL;

const fn st(x86: bool, profile: u32, state: u8) -> Strategy {
    Strategy {
        x86,
        profile,
        state,
    }
}

/// The portfolio, in order: FULL, MID and V03, then the same over the x86
/// filter, then FULL and MID continuing state 1 (plain before filtered before
/// warmed, so equal weights schedule the simpler strategy first).
pub const STRATEGIES: [Strategy; COUNT] = [
    st(false, ncm::FULL, 0),
    st(false, MID, 0),
    st(false, ncm::V03, 0),
    st(true, ncm::FULL, 0),
    st(true, MID, 0),
    st(true, ncm::V03, 0),
    st(false, ncm::FULL, 1),
    st(false, MID, 1),
];

/// Strategies in the portfolio.
pub const COUNT: usize = 8;

/// Families: x86 code ([`crate::x86::dense`]), tiny (< 4 KiB), small
/// (< 64 KiB), text, other; numbered tiny 0, small 1, x86 2, text 3, other 4.
pub const FAMILIES: usize = 5;

/// Strategies the encoder codes for a frame of `family` (`Client.budgetFor`):
/// four for tiny and small frames, three for text, two for x86 and other
/// frames. With these, each family's schedule holds the two strategies v0.4
/// coded for it, and every strategy without a state learns its mask, so a
/// frame is never coded larger than v0.4 coded it in the same family.
pub const fn budget(family: usize) -> usize {
    match family {
        0 | 1 => 4,
        3 => 3,
        _ => 2,
    }
}

/// The learned weight of each strategy (columns, portfolio order) for each
/// family (rows).
pub const WEIGHTS: [[u8; COUNT]; FAMILIES] = [
    [1, 1, 15, 1, 1, 1, 3, 1],
    [1, 16, 5, 1, 1, 1, 1, 3],
    [1, 1, 1, 1, 16, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 16, 1],
    [16, 4, 1, 2, 1, 1, 1, 1],
];

/// The strategy table the model reads: per strategy its profile word
/// (bit 31 the x86 filter) little-endian, then its state byte.
pub fn strategy_bytes() -> Vec<u8> {
    STRATEGIES
        .iter()
        .flat_map(|s| {
            let word = s.profile | (u32::from(s.x86) << 31);
            word.to_le_bytes().into_iter().chain([s.state])
        })
        .collect()
}

/// The weight table the model reads: family-major, [`COUNT`] per family.
pub fn weight_bytes() -> Vec<u8> {
    WEIGHTS.iter().flatten().copied().collect()
}

/// The rule30 learning rule: a success adds 2 up to 16, a failure takes 1
/// down to 1.
pub const fn learn(w: u8, success: bool) -> u8 {
    if success {
        if w + 2 <= 16 {
            w + 2
        } else {
            16
        }
    } else if w >= 2 {
        w - 1
    } else {
        1
    }
}

/// The family of a frame: x86 code ([`crate::x86::dense`]) first, at any
/// size, then tiny below 4 KiB, small below 64 KiB, text (at least 90% of the
/// bytes are tab, line feed, carriage return or printable ASCII), else other.
pub fn family(frame: &[u8]) -> usize {
    let n = frame.len();
    if crate::x86::dense(frame) {
        2
    } else if n < 4096 {
        0
    } else if n < 65536 {
        1
    } else if 10 * printable(frame) >= 9 * n {
        3
    } else {
        4
    }
}

fn printable(frame: &[u8]) -> usize {
    frame
        .iter()
        .filter(|&&b| b == 9 || b == 10 || b == 13 || (32..127).contains(&b))
        .count()
}

/// Every strategy index, ordered by its weight for `family` (highest first,
/// ties in portfolio order): a permutation of the portfolio.
pub fn schedule_with(weights: &[[u8; COUNT]; FAMILIES], family: usize) -> [usize; COUNT] {
    let mut s: [usize; COUNT] = std::array::from_fn(|i| i);
    // stable, so equal weights keep portfolio order
    s.sort_by_key(|&i| std::cmp::Reverse(weights[family][i]));
    s
}

/// The schedule of `family` under the learned [`WEIGHTS`].
pub fn schedule(family: usize) -> [usize; COUNT] {
    schedule_with(&WEIGHTS, family)
}

/// The strategies the encoder codes for `frame`, in schedule order.
pub fn plan(frame: &[u8]) -> Vec<Strategy> {
    let f = family(frame);
    schedule(f)[..budget(f)]
        .iter()
        .map(|&i| STRATEGIES[i])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One outcome per training frame: its family and each strategy's size.
    fn outcomes() -> Vec<(usize, [u64; COUNT])> {
        include_str!("portfolio-outcomes.tsv")
            .lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| {
                let v: Vec<u64> = l.split('\t').map(|x| x.parse().expect("number")).collect();
                (
                    v[1] as usize,
                    v[2..2 + COUNT].try_into().expect("a size per strategy"),
                )
            })
            .collect()
    }

    #[test]
    fn weights_are_learned_from_the_outcomes() {
        let mut w = [[8u8; COUNT]; FAMILIES];
        for (f, sizes) in outcomes() {
            let best = (0..COUNT)
                .min_by_key(|&i| (sizes[i], i))
                .expect("strategies");
            for (i, w) in w[f].iter_mut().enumerate() {
                *w = learn(*w, i == best);
            }
        }
        assert_eq!(w, WEIGHTS);
    }

    #[test]
    fn learning_stays_bounded_and_schedules_are_permutations() {
        for w in 1..=16 {
            assert!((1..=16).contains(&learn(w, true)));
            assert!((1..=16).contains(&learn(w, false)));
        }
        for (f, w) in WEIGHTS.iter().enumerate() {
            let mut s = schedule(f);
            for k in 1..COUNT {
                assert!(w[s[k - 1]] >= w[s[k]]);
            }
            s.sort_unstable();
            assert_eq!(s, std::array::from_fn(|i| i));
        }
    }

    /// On the training frames, the budgeted schedule is within 0.25% of
    /// coding every strategy and keeping the smallest.
    #[test]
    fn the_schedule_nearly_reaches_the_best_strategy() {
        let (mut got, mut best) = (0, 0);
        for (f, sizes) in outcomes() {
            got += schedule(f)[..budget(f)]
                .iter()
                .map(|&i| sizes[i])
                .min()
                .expect("budget");
            best += sizes.iter().min().expect("strategies");
        }
        assert!(got * 10_000 <= best * 10_025, "{got} vs {best}");
    }

    #[test]
    fn families() {
        assert_eq!(family(&[0; 100]), 0);
        // x86 code is x86 at any size
        assert_eq!(family(&[0xE8, 0x10, 0, 0, 0, 0x90, 0x90, 0x90]), 2);
        assert_eq!(family(&[b'a'; 10_000]), 1);
        assert_eq!(family(&[b'a'; 100_000]), 3);
        assert_eq!(family(&[0; 100_000]), 4);
    }
}
