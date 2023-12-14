/// Iterate over all unique pairs of elements in a slice
pub(crate) struct PairsIterator<'a, T> {
    slice: &'a [T],
    index1: usize,
    index2: usize,
}

impl<'a, T> PairsIterator<'a, T> {
    fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            index1: 0,
            index2: 1,
        }
    }
}

impl<'a, T> Iterator for PairsIterator<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index1 < self.slice.len() {
            if self.index2 < self.slice.len() {
                let pair = (&self.slice[self.index1], &self.slice[self.index2]);
                self.index2 += 1;
                Some(pair)
            } else {
                self.index1 += 1;
                self.index2 = self.index1 + 1;
                self.next()
            }
        } else {
            None
        }
    }
}

pub(crate) trait SliceUtils<T> {
    fn pairs(&self) -> PairsIterator<T>;
}

impl<T> SliceUtils<T> for [T] {
    fn pairs(&self) -> PairsIterator<T> {
        PairsIterator::new(self)
    }
}

/// Extensions to [[u8]] for ASCII-specific operations
pub(crate) trait AsciiUtils<'a> {
    type Lines: Iterator<Item = &'a [u8]>;
    /// Iterate over the lines in a slice of ASCII bytes
    fn ascii_lines(&self) -> Self::Lines;

    fn parse<'f, F>(self) -> Result<F, F::Error>
    where
        F: FromAscii<Slice<'f> = Self>,
        Self: Sized,
    {
        F::from_ascii(self)
    }

    /// Interpret the slice as a grid of cells that can be converted from ASCII
    /// characters, where each line is the same length.
    fn grid_like<Cell: TryFrom<u8>>(&self) -> Result<GridLike<Cell>, Cell::Error> {
        // TODO: probably not optimized
        let cells = self
            .ascii_lines()
            .flat_map(|line| line.iter().map(|&c| c.try_into()))
            .collect::<Result<Vec<Cell>, Cell::Error>>()?;
        let width = self
            .ascii_lines()
            .next()
            .map(|line| line.len())
            .unwrap_or(0);
        let height = self.ascii_lines().count();
        Ok(GridLike {
            cells,
            width,
            height,
        })
    }
}

impl<'a> AsciiUtils<'a> for &'a [u8] {
    type Lines = LinesIterator<'a>;
    fn ascii_lines(&self) -> LinesIterator<'a> {
        LinesIterator::new(self)
    }
}

/// Iterate over the lines in a slice of ASCII bytes
pub(crate) struct LinesIterator<'a> {
    slice: &'a [u8],
    index: usize,
}

impl<'a> LinesIterator<'a> {
    fn new(slice: &'a [u8]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'a> Iterator for LinesIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            let start = self.index;
            let slice = &self.slice[start..];
            let end = if let Some(newline) = slice.iter().position(|&c| c == b'\n') {
                self.index += newline + 1;
                start + newline
            } else {
                self.index = self.slice.len();
                self.slice.len()
            };
            Some(&self.slice[start..end])
        } else {
            None
        }
    }
}

/// Similar to FromStr, but for ASCII bytes
pub(crate) trait FromAscii: Sized {
    type Slice<'a>;
    type Error;
    fn from_ascii(s: Self::Slice<'_>) -> Result<Self, Self::Error>;
}

/// A grid of cells that can be converted from ASCII characters.
///
/// This is a helper struct for implementing [FromGridLike] for a type. It does
/// not directly implement any grid utility methods, because they might be
/// problem-specific and are left to the implementer of [FromGridLike].
pub(crate) struct GridLike<Cell> {
    pub(crate) cells: Vec<Cell>,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl<Cell> GridLike<Cell> {
    pub(crate) fn into_grid<G>(self) -> G
    where
        G: FromGridLike<Cell = Cell>,
        Cell: TryFrom<u8>,
    {
        let GridLike {
            cells,
            width,
            height,
        } = self;
        G::from_cells(cells, width, height)
    }
}

pub(crate) trait FromGridLike
where
    Self: Sized,
{
    type Cell: TryFrom<u8>;
    fn from_cells(cells: Vec<Self::Cell>, width: usize, height: usize) -> Self;
}

#[derive(Debug)]
pub(crate) struct InvalidCharacter(pub(crate) u8);

macro_rules! grid_cell_enum {
    (
        $(#[$attrs:meta])?
        enum $name:ident {
            $($variant:ident => $value:expr),*$(,)?
        }
    )
        => {
            $(#[$attrs])?
            enum $name {
                $($variant,)*
            }

            impl TryFrom<u8> for $name {
                type Error = $crate::utils::InvalidCharacter;
                fn try_from(c: u8) -> Result<Self, $crate::utils::InvalidCharacter> {
                    match c {
                        $($value => Ok(Self::$variant),)*
                        c => Err($crate::utils::InvalidCharacter(c)),
                    }
                }
            }

            impl core::fmt::Display for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        $($name::$variant => write!(f, "{}", $value as char),)*
                    }
                }
            }
        }
}

pub(crate) use grid_cell_enum;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_iterator() {
        let mut iter = PairsIterator::new(&[1, 2, 3, 4]);
        assert_eq!(iter.next(), Some((&1, &2)));
        assert_eq!(iter.next(), Some((&1, &3)));
        assert_eq!(iter.next(), Some((&1, &4)));
        assert_eq!(iter.next(), Some((&2, &3)));
        assert_eq!(iter.next(), Some((&2, &4)));
        assert_eq!(iter.next(), Some((&3, &4)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn pairs_iterator_too_small() {
        let mut iter = PairsIterator::new(&[1]);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn pairs_iterator_empty() {
        let mut iter = PairsIterator::new(&[1]);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ascii_lines() {
        let mut iter = LinesIterator::new(b"abc\ndef\nghi\n");
        assert_eq!(iter.next(), Some(&b"abc"[..]));
        assert_eq!(iter.next(), Some(&b"def"[..]));
        assert_eq!(iter.next(), Some(&b"ghi"[..]));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ascii_lines_unterminated() {
        let mut iter = LinesIterator::new(b"abc\ndef\nghi");
        assert_eq!(iter.next(), Some(&b"abc"[..]));
        assert_eq!(iter.next(), Some(&b"def"[..]));
        assert_eq!(iter.next(), Some(&b"ghi"[..]));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ascii_lines_single_line() {
        let mut iter = LinesIterator::new(b"abc");
        assert_eq!(iter.next(), Some(&b"abc"[..]));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ascii_lines_empty() {
        let mut iter = LinesIterator::new(b"");
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ascii_lines_empty_lines() {
        let mut iter = LinesIterator::new(b"abc\n\nghi");
        assert_eq!(iter.next(), Some(&b"abc"[..]));
        assert_eq!(iter.next(), Some(&b""[..]));
        assert_eq!(iter.next(), Some(&b"ghi"[..]));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn ascii_parse() {
        struct Foo;
        impl FromAscii for Foo {
            type Slice<'a> = &'a [u8];
            type Error = ();
            fn from_ascii(s: &[u8]) -> Result<Self, Self::Error> {
                assert_eq!(s, b"abc");
                Ok(Foo)
            }
        }
        assert!(matches!(b"abc".parse::<Foo>(), Ok(Foo)));
        let foo = vec![b'a', b'b', b'c'];
        assert!(matches!(foo.as_slice().parse::<Foo>(), Ok(Foo)));
    }

    #[test]
    fn ascii_grid() {
        let grid = b"abc\ndef\nghi\njkl".as_slice().grid_like::<u8>().unwrap();
        assert_eq!(grid.width, 3);
        assert_eq!(grid.height, 4);
        assert_eq!(grid.cells, b"abcdefghijkl".to_vec(),);
    }
}
