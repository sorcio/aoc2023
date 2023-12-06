/// Kinda like Range/RangeInclusive but the end might be > u32::MAX
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Range {
    start: u32,
    length: u32,
}

impl Range {
    pub(crate) const fn new(start: u32, length: u32) -> Self {
        Range { start, length }
    }

    pub(crate) fn excl(start: u32, end: u32) -> Self {
        debug_assert!(end >= start);
        (start..end).into()
    }

    pub(crate) fn len(&self) -> u32 {
        self.length
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn contains(&self, n: u32) -> bool {
        n >= self.start && n - self.start < self.length
    }

    pub(crate) fn distance_from_start(&self, n: u32) -> Option<u32> {
        self.contains(n).then(|| n.checked_sub(self.start))?
    }

    pub(crate) fn start(&self) -> u32 {
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

impl Overlaps for Range {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn range() {
        assert_eq!(Range::new(10, 10), Range::excl(10, 20));
        assert_eq!(Range::new(0x8000_0000, 0xF000_0000).len(), 0xF000_0000);
        assert_eq!(Range::new(0x8000_0000, 0xF000_0000).end(), 0x1_7000_0000);
    }

    #[test]
    fn intersection() {
        assert_eq!(Range::excl(0, 10).intersection(&Range::excl(10, 50)), None);
        assert_eq!(Range::excl(0, 11).intersection(&Range::excl(10, 50)), Some(Range::excl(10, 11)));
        assert_eq!(Range::excl(0, 50).intersection(&Range::excl(10, 50)), Some(Range::excl(10, 50)));
        assert_eq!(Range::excl(0, 60).intersection(&Range::excl(10, 50)), Some(Range::excl(10, 50)));
        assert_eq!(Range::excl(10, 10).intersection(&Range::excl(10, 50)), None);
        assert_eq!(Range::excl(10, 15).intersection(&Range::excl(10, 50)), Some(Range::excl(10, 15)));
        assert_eq!(Range::excl(10, 50).intersection(&Range::excl(10, 50)), Some(Range::excl(10, 50)));
        assert_eq!(Range::excl(10, 60).intersection(&Range::excl(10, 50)), Some(Range::excl(10, 50)));
        assert_eq!(Range::excl(20, 20).intersection(&Range::excl(10, 50)), None);
        assert_eq!(Range::excl(20, 25).intersection(&Range::excl(10, 50)), Some(Range::excl(20, 25)));
        assert_eq!(Range::excl(20, 50).intersection(&Range::excl(10, 50)), Some(Range::excl(20, 50)));
        assert_eq!(Range::excl(20, 60).intersection(&Range::excl(10, 50)), Some(Range::excl(20, 50)));
        assert_eq!(Range::excl(50, 50).intersection(&Range::excl(10, 50)), None);
        assert_eq!(Range::excl(50, 60).intersection(&Range::excl(10, 50)), None);
        assert_eq!(Range::excl(60, 70).intersection(&Range::excl(10, 50)), None);       
    }

    #[test]
    fn overlaps() {
        assert!(!Range::excl(0, 10).overlaps(&Range::excl(10, 50)));
        assert!(Range::excl(0, 11).overlaps(&Range::excl(10, 50)));
        assert!(Range::excl(0, 50).overlaps(&Range::excl(10, 50)));
        assert!(Range::excl(0, 60).overlaps(&Range::excl(10, 50)));
        assert!(!Range::excl(10, 10).overlaps(&Range::excl(10, 50)));
        assert!(Range::excl(10, 15).overlaps(&Range::excl(10, 50)));
        assert!(Range::excl(10, 50).overlaps(&Range::excl(10, 50)));
        assert!(Range::excl(10, 60).overlaps(&Range::excl(10, 50)));
        assert!(!Range::excl(20, 20).overlaps(&Range::excl(10, 50)));
        assert!(Range::excl(20, 25).overlaps(&Range::excl(10, 50)));
        assert!(Range::excl(20, 50).overlaps(&Range::excl(10, 50)));
        assert!(Range::excl(20, 60).overlaps(&Range::excl(10, 50)));
        assert!(!Range::excl(50, 50).overlaps(&Range::excl(10, 50)));
        assert!(!Range::excl(50, 60).overlaps(&Range::excl(10, 50)));
        assert!(!Range::excl(60, 70).overlaps(&Range::excl(10, 50)));
    }    
}

pub(crate) trait Overlaps {
    fn overlaps(&self, other: &Self) -> bool;
}

impl From<std::ops::Range<u32>> for Range {
    fn from(value: std::ops::Range<u32>) -> Self {
        Self {
            start: value.start,
            length: value.end - value.start,
        }
   }
}