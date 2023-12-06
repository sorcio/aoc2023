//! Utilities for working with ranges and intervals.

pub(crate) trait Overlaps {
    fn overlaps(&self, other: &Self) -> bool;
}

/// Get the length of a sized collection.
pub(crate) trait HasExtent {
    type Extent;
    /// Returns the length of a sized collection. Similar to [`&[T].len`] or
    /// [`ExactSizeIterator::len`].
    fn extent(&self) -> Self::Extent;
}

impl<T> HasExtent for std::ops::Range<T>
where
    T: std::ops::Sub<T, Output = T> + Copy,
{
    type Extent = T;
    fn extent(&self) -> T {
        self.end - self.start
    }
}

/// Kinda like Range/RangeInclusive but the end might be > u32::MAX
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Interval<T = u32> {
    start: T,
    length: T,
}

impl<T: Copy> Interval<T> {
    pub(crate) const fn new(start: T, length: T) -> Self {
        Interval { start, length }
    }

    pub(crate) fn len(&self) -> T {
        self.length
    }
}

impl<T: Copy> HasExtent for Interval<T> {
    type Extent = T;
    fn extent(&self) -> T {
        self.length
    }
}

macro_rules! interval_impl {
    ($t:ty) => {
        #[allow(dead_code)]
        impl Interval<$t> {
            pub(crate) fn excl(start: $t, end: $t) -> Self {
                debug_assert!(end >= start);
                (start..end).into()
            }

            pub(crate) fn is_empty(&self) -> bool {
                self.len() == 0
            }

            pub(crate) fn contains(&self, n: $t) -> bool {
                n >= self.start && n - self.start < self.length
            }

            pub(crate) fn distance_from_start(&self, n: $t) -> Option<$t> {
                self.contains(n).then(|| n.checked_sub(self.start))?
            }

            pub(crate) fn start(&self) -> $t {
                self.start
            }

            pub(crate) fn end(&self) -> u64 {
                self.start as u64 + self.length as u64
            }

            pub(crate) fn intersection(&self, other: &Self) -> Option<Self> {
                if self.start >= other.start {
                    let diff = self.start - other.start;
                    if other.len() > diff {
                        let length = self.length.min(other.length - diff);
                        if length == 0 {
                            None
                        } else {
                            Some(Self::new(self.start, length))
                        }
                    } else {
                        None
                    }
                } else {
                    other.intersection(self)
                }
            }
        }

        impl Overlaps for Interval<$t> {
            fn overlaps(&self, other: &Self) -> bool {
                // I'm too lazy to think how to simplify this so let's just use u64 everywhere
                let a_start = self.start as u64;
                let b_start = other.start as u64;
                let a_end = self.end();
                let b_end = other.end();
                // self.start < other.end && other.start < self.end && !self.is_empty() && !other.is_empty()
                a_start < b_end && b_start < a_end && !self.is_empty() && !other.is_empty()
            }
        }

        impl From<std::ops::Range<$t>> for Interval<$t> {
            fn from(value: std::ops::Range<$t>) -> Self {
                Self {
                    start: value.start,
                    length: value.end - value.start,
                }
            }
        }
    };
}

interval_impl!(u32);
interval_impl!(u64);

#[cfg(test)]
mod interval_tests {
    macro_rules! test_interval_impl {
        ($t:ident) => {
            mod $t {
                use $crate::range::Overlaps;
                type Interval = $crate::range::Interval<$t>;

                #[test]
                fn range() {
                    assert_eq!(Interval::new(10, 10), Interval::excl(10, 20));
                    assert_eq!(Interval::new(0x8000_0000, 0xF000_0000).len(), 0xF000_0000);
                    assert_eq!(Interval::new(0x8000_0000, 0xF000_0000).end(), 0x1_7000_0000);
                }

                #[test]
                fn intersection() {
                    assert_eq!(
                        Interval::excl(0, 10).intersection(&Interval::excl(10, 50)),
                        None
                    );
                    assert_eq!(
                        Interval::excl(0, 11).intersection(&Interval::excl(10, 50)),
                        Some(Interval::excl(10, 11))
                    );
                    assert_eq!(
                        Interval::excl(0, 50).intersection(&Interval::excl(10, 50)),
                        Some(Interval::excl(10, 50))
                    );
                    assert_eq!(
                        Interval::excl(0, 60).intersection(&Interval::excl(10, 50)),
                        Some(Interval::excl(10, 50))
                    );
                    assert_eq!(
                        Interval::excl(10, 10).intersection(&Interval::excl(10, 50)),
                        None
                    );
                    assert_eq!(
                        Interval::excl(10, 15).intersection(&Interval::excl(10, 50)),
                        Some(Interval::excl(10, 15))
                    );
                    assert_eq!(
                        Interval::excl(10, 50).intersection(&Interval::excl(10, 50)),
                        Some(Interval::excl(10, 50))
                    );
                    assert_eq!(
                        Interval::excl(10, 60).intersection(&Interval::excl(10, 50)),
                        Some(Interval::excl(10, 50))
                    );
                    assert_eq!(
                        Interval::excl(20, 20).intersection(&Interval::excl(10, 50)),
                        None
                    );
                    assert_eq!(
                        Interval::excl(20, 25).intersection(&Interval::excl(10, 50)),
                        Some(Interval::excl(20, 25))
                    );
                    assert_eq!(
                        Interval::excl(20, 50).intersection(&Interval::excl(10, 50)),
                        Some(Interval::excl(20, 50))
                    );
                    assert_eq!(
                        Interval::excl(20, 60).intersection(&Interval::excl(10, 50)),
                        Some(Interval::excl(20, 50))
                    );
                    assert_eq!(
                        Interval::excl(50, 50).intersection(&Interval::excl(10, 50)),
                        None
                    );
                    assert_eq!(
                        Interval::excl(50, 60).intersection(&Interval::excl(10, 50)),
                        None
                    );
                    assert_eq!(
                        Interval::excl(60, 70).intersection(&Interval::excl(10, 50)),
                        None
                    );
                }

                #[test]
                fn overlaps() {
                    assert!(!Interval::excl(0, 10).overlaps(&Interval::excl(10, 50)));
                    assert!(Interval::excl(0, 11).overlaps(&Interval::excl(10, 50)));
                    assert!(Interval::excl(0, 50).overlaps(&Interval::excl(10, 50)));
                    assert!(Interval::excl(0, 60).overlaps(&Interval::excl(10, 50)));
                    assert!(!Interval::excl(10, 10).overlaps(&Interval::excl(10, 50)));
                    assert!(Interval::excl(10, 15).overlaps(&Interval::excl(10, 50)));
                    assert!(Interval::excl(10, 50).overlaps(&Interval::excl(10, 50)));
                    assert!(Interval::excl(10, 60).overlaps(&Interval::excl(10, 50)));
                    assert!(!Interval::excl(20, 20).overlaps(&Interval::excl(10, 50)));
                    assert!(Interval::excl(20, 25).overlaps(&Interval::excl(10, 50)));
                    assert!(Interval::excl(20, 50).overlaps(&Interval::excl(10, 50)));
                    assert!(Interval::excl(20, 60).overlaps(&Interval::excl(10, 50)));
                    assert!(!Interval::excl(50, 50).overlaps(&Interval::excl(10, 50)));
                    assert!(!Interval::excl(50, 60).overlaps(&Interval::excl(10, 50)));
                    assert!(!Interval::excl(60, 70).overlaps(&Interval::excl(10, 50)));
                }
            }
        };
    }

    test_interval_impl!(u32);
    test_interval_impl!(u64);
}
