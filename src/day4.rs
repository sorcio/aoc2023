use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::example_tests;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct AsciiNumber<const LEN: usize>([u8; LEN]);

macro_rules! ascii_to_number {
    ($t:ty) => {
        impl<const LEN: usize> From<AsciiNumber<LEN>> for $t {
            fn from(value: AsciiNumber<LEN>) -> Self {
                Self::from(&value)
            }
        }

        impl<const LEN: usize> From<&AsciiNumber<LEN>> for $t {
            fn from(value: &AsciiNumber<LEN>) -> Self {
                debug_assert!(LEN > 0);
                value
                    .0
                    .iter()
                    .skip_while(|&&b| b == b' ')
                    .map(|&b| (b - b'0') as $t)
                    .fold(0, |acc, x| acc * 10 + x)
            }
        }
    };
}

ascii_to_number!(u8);
ascii_to_number!(u16);

struct Card<const A: usize, const B: usize> {
    id: u16,
    winning: [u8; A],
    own: [u8; B],
}

impl<const A: usize, const B: usize> Card<A, B> {
    fn own_winning(&self) -> impl Iterator<Item = u8> + '_ {
        self.winning
            .iter()
            .filter(|n| self.own.contains(*n))
            .copied()
    }

    /// score according to part 1
    fn score(&self) -> usize {
        match self.own_winning().count() {
            0 => 0,
            n => 2_usize.pow((n - 1).try_into().unwrap()),
        }
    }

    /// range of cards won by this card (assuming cards are in a stack indexed by id - 1)
    fn won_range(&self) -> std::ops::Range<usize> {
        let winning_count = self.own_winning().count();
        let start = self.id as usize;  // id is always 1 + index
        let end = start + winning_count;
        start..end
    }
}

#[aoc_generator(day4)]
fn parse(input: &[u8]) -> Vec<Card<10, 25>> {
    parse_generic(input)
}

#[aoc(day4, part1)]
fn part1(cards: &[Card<10, 25>]) -> usize {
    part1_generic(cards)
}

fn parse_generic<const A: usize, const B: usize>(input: &[u8]) -> Vec<Card<A, B>> {
    // input file is neatly aligned text so just for fun and because we can let's
    // treat it as a binary file in the most unsafe way

    #[repr(packed)]
    #[derive(Clone, Copy)]
    struct Number {
        _fuffa: u8,
        number: AsciiNumber<2>,
    }
    #[repr(packed)]
    struct Line<const A: usize, const B: usize> {
        _card: [u8; 5],
        card_id: AsciiNumber<3>,
        _colon: u8,
        winning: [Number; A],
        _sep: [u8; 2],
        own: [Number; B],
        _newline: u8,
    }

    // SAFETY: obviously none, since we are reading a file and skipping a bunch
    // of checks. But as long as it works and Miri is happy so am I.
    let data = input.as_ptr();
    // dbg!(input.len(), std::mem::size_of::<Line<A, B>>(), input.len() / std::mem::size_of::<Line<A, B>>());
    let len = (input.len() + 1) / std::mem::size_of::<Line<A, B>>();
    let records: &[Line<A, B>] = unsafe { std::slice::from_raw_parts(data as _, len) };
    #[cfg(debug_assertions)]
    for record in records {
        debug_assert!(unsafe { std::ptr::read_unaligned(&record._newline) } == b'\n');
    }

    records
        .iter()
        .map(|record| {
            let id = u16::from(&record.card_id);
            let winning = record.winning.map(|ascii| ascii.number.into());
            let own = record.own.map(|ascii| ascii.number.into());
            Card::<A, B> { id, winning, own }
        })
        .collect()
}

fn part1_generic<const A: usize, const B: usize>(cards: &[Card<A, B>]) -> usize {
    cards
        .into_iter()
        .map(|card| card.score())
        .sum()
}

fn part2_generic<const A: usize, const B: usize>(input_cards: &[Card<A, B>]) -> usize {
    let mut cards = vec![0; input_cards.len()];
    for i in (0..input_cards.len()).rev() {
        let won_range = input_cards[i].won_range();
        cards[i] = won_range.len() + won_range.map(|won_i| cards[won_i]).sum::<usize>();
    }
    input_cards.len() + cards.into_iter().sum::<usize>()
}

#[aoc(day4, part2)]
fn part2(cards: &[Card<10, 25>]) -> usize {
    assert!(cards.len() == 201);
    part2_generic(cards)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number() {
        assert_eq!(u8::from(AsciiNumber(*b" 123")), 123);
        assert_eq!(u16::from(AsciiNumber(*b" 123")), 123);
        assert_eq!(u16::from(AsciiNumber(*b"123")), 123);
        assert_eq!(u16::from(AsciiNumber(*b"12345")), 12345);
    }

    #[test]
    fn number_unaligned() {
        let data = b"x1234";
        #[repr(packed)]
        struct Foo {
            _foo: u8,
            number: AsciiNumber<4>,
        }
        let foo: Foo = unsafe { std::mem::transmute_copy(data) };
        assert_eq!(u16::from(&foo.number), 1234);
        assert_eq!(u16::from(foo.number), 1234);
    }

    #[test]
    fn parser() {
        let parsed = parse_generic::<10, 25>(include_bytes!("../input/2023/day4.txt"));
        assert_eq!(parsed.len(), 201);
        assert_eq!(parsed[0].id, 1);
        assert_eq!(parsed[0].winning, [91, 73, 74, 57, 24, 99, 31, 70, 60, 8]);
        assert_eq!(
            parsed[0].own,
            [
                89, 70, 43, 24, 62, 30, 91, 87, 60, 57, 90, 2, 27, 3, 31, 25, 39, 83, 64, 73, 99,
                8, 74, 37, 49
            ]
        );
    }
}

#[cfg(test)]
fn parse_example(input: &[u8]) -> Vec<Card<5, 8>> {
    parse_generic(input)
}

#[cfg(test)]
fn part1_example(cards: &[Card<5, 8>]) -> usize {
    part1_generic(cards)
}

#[cfg(test)]
fn part2_example(cards: &[Card<5, 8>]) -> usize {
    part2_generic(cards)
}

example_tests! {
    parser: crate::day4::parse_example,

    // note we added some extra space compared to original input because
    // uhhh we decided that record headers are fixed size of course
    b"
    Card   1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
    Card   2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
    Card   3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
    Card   4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
    Card   5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
    Card   6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11
    ",

    part1_example => 13,
    part2_example => 30
}
