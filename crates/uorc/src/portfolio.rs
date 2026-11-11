//! The strategy portfolio (`Uorc.Client.portfolio`): how the encoder picks
//! the coding of a frame, after the schedule engine of afflom/rule30
//! (`Engine.lex.tex`) and the PrismPM factory.
//!
//! A strategy is a transform (none, or the x86 branch filter) and an ncm
//! model profile. A frame's family comes from cheap statistics of its bytes
//! ([`family`]). Every (family, strategy) pair carries a bounded integer
//! weight in 1..=16; the schedule of a family is every strategy ordered by
//! weight (ties by portfolio order), so no strategy is ever unreachable. The
//! encoder codes the first [`BUDGET`] strategies of the schedule and keeps
//! the smallest archive (raw when that is smaller): the weights order the
//! work, and the measured sizes decide.
//!
//! [`WEIGHTS`] is learned by the rule30 rule ([`learn`]: +2 for the strategy
//! whose archive was smallest, -1 for every other, bounded to 1..=16) over
//! the measured archive lengths in `portfolio-outcomes.tsv` (V03 writes the
//! v0.3 entry, 7-8 B shorter than a profiled one), starting from 8; the
//! test `weights_are_learned_from_the_outcomes` repeats that learning. The
//! table is part of the client's model, so the output is a function of the
//! input alone, whatever `--threads` is.

use crate::ncm;

/// A way to code a frame: over the x86 filter or not, and the ncm profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Strategy {
    /// Coded over the x86 branch filter.
    pub x86: bool,
    /// The ncm model profile.
    pub profile: u32,
}

/// Every ncm feature but the match-state mixer selection: the profile that
/// short inputs favour.
pub const MID: u32 = ncm::FULL & !ncm::F_SEL;

/// The portfolio, in order.
pub const STRATEGIES: [Strategy; 6] = [
    Strategy {
        x86: false,
        profile: ncm::FULL,
    },
    Strategy {
        x86: true,
        profile: ncm::FULL,
    },
    Strategy {
        x86: false,
        profile: MID,
    },
    Strategy {
        x86: true,
        profile: MID,
    },
    Strategy {
        x86: false,
        profile: ncm::V03,
    },
    Strategy {
        x86: true,
        profile: ncm::V03,
    },
];

/// Families: tiny (< 4 KiB), small (< 64 KiB), x86 code (any size), text, other.
pub const FAMILIES: usize = 5;

/// Strategies the encoder codes per frame.
pub const BUDGET: usize = 2;

/// The learned weight of each strategy (columns, portfolio order) for each
/// family (rows).
pub const WEIGHTS: [[u8; 6]; FAMILIES] = [
    [1, 1, 1, 1, 16, 1],
    [1, 1, 14, 1, 16, 1],
    [4, 1, 1, 15, 1, 1],
    [10, 1, 10, 1, 4, 1],
    [12, 1, 12, 1, 1, 1],
];

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

/// The family of a frame: x86 code ([`crate::x86::dense`], at any size, so
/// a small executable still reaches the filter), then tiny below 4 KiB,
/// small below 64 KiB, text (at least 90% of the bytes are tab, line feed,
/// carriage return or printable ASCII), else other.
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
pub fn schedule_with(weights: &[[u8; 6]; FAMILIES], family: usize) -> [usize; 6] {
    let mut s = [0, 1, 2, 3, 4, 5];
    // stable, so equal weights keep portfolio order
    s.sort_by_key(|&i| std::cmp::Reverse(weights[family][i]));
    s
}

/// The schedule of `family` under the learned [`WEIGHTS`].
pub fn schedule(family: usize) -> [usize; 6] {
    schedule_with(&WEIGHTS, family)
}

/// The strategies the encoder codes for `frame`, in schedule order.
pub fn plan(frame: &[u8]) -> [Strategy; BUDGET] {
    let s = schedule(family(frame));
    std::array::from_fn(|k| STRATEGIES[s[k]])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One outcome per training frame: its family and each strategy's size.
    fn outcomes() -> Vec<(usize, [u64; 6])> {
        include_str!("portfolio-outcomes.tsv")
            .lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| {
                let v: Vec<u64> = l.split('\t').map(|x| x.parse().expect("number")).collect();
                (v[1] as usize, v[2..8].try_into().expect("six sizes"))
            })
            .collect()
    }

    #[test]
    fn weights_are_learned_from_the_outcomes() {
        let mut w = [[8u8; 6]; FAMILIES];
        for (f, sizes) in outcomes() {
            let best = (0..6).min_by_key(|&i| (sizes[i], i)).expect("six");
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
            for k in 1..6 {
                assert!(w[s[k - 1]] >= w[s[k]]);
            }
            s.sort_unstable();
            assert_eq!(s, [0, 1, 2, 3, 4, 5]);
        }
    }

    /// On the training frames, the budget-2 schedule is within 0.01% of
    /// coding every strategy and keeping the smallest.
    #[test]
    fn the_schedule_nearly_reaches_the_best_strategy() {
        let (mut got, mut best) = (0, 0);
        for (f, sizes) in outcomes() {
            got += schedule(f)[..BUDGET]
                .iter()
                .map(|&i| sizes[i])
                .min()
                .expect("budget");
            best += sizes.iter().min().expect("six");
        }
        assert!(got * 10_000 <= best * 10_001, "{got} vs {best}");
    }

    #[test]
    fn families() {
        assert_eq!(family(&[0; 100]), 0);
        assert_eq!(family(&[b'a'; 10_000]), 1);
        assert_eq!(family(&[b'a'; 100_000]), 3);
        assert_eq!(family(&[0; 100_000]), 4);
        // x86 code is x86 at any size, so a small executable tries the filter
        let calls: Vec<u8> = (0..400u32)
            .flat_map(|k| [0xE8, (k % 200) as u8, 0, 0, 0, 0x90])
            .collect();
        assert!(calls.len() < 4096);
        assert_eq!(family(&calls), 2);
    }
}
