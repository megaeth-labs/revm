//! The marks revm writes on a frame's result just before an inspector's end callbacks, and reads
//! just after, to tell whether the callbacks put another value in the result's place: the
//! [`StartMark`] on the result's gas tracker and the [`OutcomeMark`] on its outcome.
//!
//! A mark takes no part in the value it rides on: marks always compare equal, hash to nothing,
//! print nothing and are not serialized (a decoded value carries none). So a marked tracker or
//! outcome compares, hashes, prints and encodes as the same value unmarked. Each fits padding of
//! the type that carries it, so neither grows.

/// The mark of a start on a gas tracker: the start's identity, or none.
///
/// revm marks the tracker of a start's result for the start before an inspector's end callbacks;
/// a tracker that does not carry the mark after them is one the callbacks put in its place
/// ([`FrameSettle::gas_replaced`](crate::host::FrameSettle::gas_replaced)). A copy of the
/// tracker carries the mark with it.
#[derive(Clone, Copy, Default)]
pub struct StartMark(u32);

impl StartMark {
    /// No mark, as every tracker is built.
    pub const NONE: Self = Self(0);

    /// The mark of the start `id`.
    #[inline]
    pub const fn for_start(id: u32) -> Self {
        Self(id)
    }

    /// Whether this is the mark of the start `id`.
    #[inline]
    pub const fn is_for(self, id: u32) -> bool {
        self.0 != 0 && self.0 == id
    }
}

/// The mark of a start on a call's or creation's outcome: 16 bits of the start's identity, or
/// none.
///
/// revm marks the outcome of a start's result for the start before an inspector's end callbacks;
/// an outcome that does not carry the mark after them is one the callbacks put in its place
/// ([`FrameSettle::outcome_rebuilt`](crate::host::FrameSettle::outcome_rebuilt)), such as one a
/// constructor built around the same result. A start's mark is never none, so a built outcome
/// never carries it; two starts share a mark only when their identities are 65,535 apart.
#[derive(Clone, Copy, Default)]
pub struct OutcomeMark(u16);

impl OutcomeMark {
    /// No mark, as every constructor builds an outcome.
    pub const NONE: Self = Self(0);

    /// The mark of the start `id`: `(id - 1) % 65535 + 1`, never none.
    #[inline]
    pub const fn for_start(id: u32) -> Self {
        Self((id.wrapping_sub(1) % 65535 + 1) as u16)
    }

    /// Whether this is the mark of the start `id`.
    #[inline]
    pub const fn is_for(self, id: u32) -> bool {
        self.0 == Self::for_start(id).0
    }
}

macro_rules! transparent {
    ($mark:ident) => {
        impl PartialEq for $mark {
            #[inline]
            fn eq(&self, _: &Self) -> bool {
                true
            }
        }

        impl Eq for $mark {}

        impl core::hash::Hash for $mark {
            #[inline]
            fn hash<H: core::hash::Hasher>(&self, _: &mut H) {}
        }

        impl core::fmt::Debug for $mark {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(stringify!($mark))
            }
        }
    };
}

transparent!(StartMark);
transparent!(OutcomeMark);

#[cfg(test)]
mod tests {
    use super::{OutcomeMark, StartMark};

    #[test]
    fn test_a_start_mark_names_its_start_only() {
        let mark = StartMark::for_start(7);
        assert!(mark.is_for(7));
        assert!(!mark.is_for(8));
        assert!(!StartMark::NONE.is_for(0), "no mark names no start");
    }

    /// The outcome's mark is never none for a start, and two starts share one only 65,535
    /// identities apart.
    #[test]
    fn test_an_outcome_mark_is_never_none_and_repeats_every_65535_starts() {
        for id in [
            1u32,
            2,
            65_534,
            65_535,
            65_536,
            65_537,
            131_070,
            131_071,
            u32::MAX - 1,
            u32::MAX,
        ] {
            assert!(!OutcomeMark::NONE.is_for(id), "id {id} has no mark");
            assert!(OutcomeMark::for_start(id).is_for(id), "id {id}");
            if id > 1 {
                assert!(!OutcomeMark::for_start(id).is_for(id - 1), "id {id}");
            }
        }
        assert!(OutcomeMark::for_start(1).is_for(65_536));
        assert!(!OutcomeMark::for_start(1).is_for(65_535));
    }
}
