use aoc_runner_derive::aoc;

use crate::testing::example_tests;

fn sum_first_last<I>(mut iterator: I) -> u32
where
    I: Iterator<Item = u32>,
{
    let first = iterator.next().unwrap_or(0);
    let last = iterator.last().unwrap_or(first);
    debug_assert!(first < 10);
    debug_assert!(last < 10);
    first * 10 + last
}

#[aoc(day1, part1)]
fn part1(input: &str) -> u32 {
    input
        .lines()
        .map(|line| sum_first_last(line.chars().filter_map(|c| c.to_digit(10))))
        .sum()
}

struct DigitIterator<'s> {
    s: &'s str,
    iter: std::str::CharIndices<'s>,
}

impl<'s> DigitIterator<'s> {
    fn new(s: &'s str) -> Self {
        Self {
            s,
            iter: s.char_indices(),
        }
    }
}

impl Iterator for DigitIterator<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let digit_names = [
            "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        ];
        for (i, c) in self.iter.by_ref() {
            if let Some(value) = c.to_digit(10) {
                return Some(value);
            } else {
                let substring = &self.s[i..];
                for (value, name) in (1..10_u32).zip(&digit_names) {
                    if substring.starts_with(name) {
                        return Some(value);
                    }
                }
            }
        }
        None
    }
}

#[aoc(day1, part2)]
fn part2(input: &str) -> u32 {
    input
        .lines()
        .map(|line| sum_first_last(DigitIterator::new(line)))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digits_in_str(s: &str) -> Vec<u32> {
        DigitIterator::new(s).collect()
    }

    #[test]
    fn digit_iterator() {
        assert_eq!(digits_in_str("onetwothree"), vec![1, 2, 3]);
        assert_eq!(digits_in_str("twone"), vec![2, 1]);
        assert_eq!(digits_in_str("fsdf"), vec![]);
        assert_eq!(digits_in_str("1two3"), vec![1, 2, 3]);
        assert_eq!(digits_in_str("123456789"), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(
            digits_in_str("onetwothreefourfivesixseveneightnine"),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }

    #[test]
    fn test_sum_first_last() {
        assert_eq!(13, sum_first_last([1, 2, 3].into_iter()));
        assert_eq!(55, sum_first_last([5].into_iter()));
        assert_eq!(0, sum_first_last([].into_iter()));
    }
}

example_tests! {
    parser: None,

    "
    1abc2
    pqr3stu8vwx
    a1b2c3d4e5f
    treb7uchet
    ",

    part1 => 142,

    "
    two1nine
    eightwothree
    abcone2threexyz
    xtwone3four
    4nineeightseven2
    zoneight234
    7pqrstsixteen",

    part2 => 281
}
