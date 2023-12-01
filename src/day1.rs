use aoc_runner_derive::{aoc, aoc_generator};

#[aoc_generator(day1)]
fn parse(input: &str) -> Vec<String> {
    input.lines().map(str::to_owned).collect()
}

#[aoc(day1, part1)]
fn part1(input: &[String]) -> u32 {
    input
        .iter()
        .map(|line| {
            let digits = line
                .chars()
                .filter_map(|c| c.to_digit(10))
                .fold(None, |acc, item| {
                    if let Some((first, _)) = acc {
                        Some((first, item))
                    } else {
                        Some((item, item))
                    }
                });
            if let Some((d1, d2)) = digits {
                debug_assert!(d1 < 10);
                debug_assert!(d2 < 10);
                d1 * 10 + d2
            } else {
                0
            }
        })
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
                for (value, name) in (1..10).zip(&digit_names) {
                    if substring.starts_with(name) {
                        return Some(value as u32);
                    }
                }
            }
        }
        None
    }
}

#[aoc(day1, part2)]
fn part2(input: &[String]) -> u32 {
    input
        .iter()
        .map(|line| {
            let digits =
                DigitIterator::new(line).fold(None, |acc, item| {
                    if let Some((first, _)) = acc {
                        Some((first, item))
                    } else {
                        Some((item, item))
                    }
                });
            if let Some((d1, d2)) = digits {
                debug_assert!(d1 < 10);
                debug_assert!(d2 < 10);
                d1 * 10 + d2
            } else {
                0
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_example() {
        assert_eq!(
            part1(&parse(
                "1abc2
        pqr3stu8vwx
        a1b2c3d4e5f
        treb7uchet"
            )),
            142
        );
    }

    #[test]
    fn part2_example() {
        assert_eq!(
            part2(&parse(
                "two1nine
        eightwothree
        abcone2threexyz
        xtwone3four
        4nineeightseven2
        zoneight234
        7pqrstsixteen"
            )),
            281
        );
    }

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
            digits_in_str("onetwothreefourfivesixseveneightnine",),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }
}
