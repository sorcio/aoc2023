use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::{example_tests, known_input_tests};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum Spring {
    Operational,
    Damaged,
    #[default]
    Unknown,
}

impl Spring {
    fn matches(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Operational, Self::Operational)
                | (Self::Damaged, Self::Damaged)
                | (Self::Unknown, _)
                | (_, Self::Unknown)
        )
    }
}

impl TryFrom<char> for Spring {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '.' => Ok(Self::Operational),
            '#' => Ok(Self::Damaged),
            '?' => Ok(Self::Unknown),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Spring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operational => write!(f, "."),
            Self::Damaged => write!(f, "#"),
            Self::Unknown => write!(f, "?"),
        }
    }
}

#[derive(Debug, Clone)]
struct SpringRow {
    pattern: Vec<Spring>,
    known_damaged: Vec<usize>,
}

impl FromStr for SpringRow {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, second) = s.trim().split_once(' ').ok_or(())?;
        let pattern = first.chars().map(|c| c.try_into().unwrap()).collect();
        let known_damaged = second.split(',').map(|s| s.parse().unwrap()).collect();
        Ok(Self {
            pattern,
            known_damaged,
        })
    }
}

impl SpringRow {
    fn repeat(self, count: usize) -> Self {
        let mut pattern = Vec::with_capacity((self.pattern.len() + 1) * count);
        for _ in 0..count {
            pattern.extend_from_slice(&self.pattern);
            pattern.push(Spring::Unknown);
        }
        let _ = pattern.pop();
        Self {
            pattern,
            known_damaged: self.known_damaged.repeat(count),
        }
    }
}

struct DisplayRow<'a>(&'a [Spring]);

impl fmt::Display for DisplayRow<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let representation = self.0.iter().map(|s| s.to_string()).collect::<String>();
        f.pad(&representation)
    }
}

impl fmt::Display for SpringRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}", DisplayRow(&self.pattern), &self.known_damaged)
    }
}

#[aoc_generator(day12)]
fn parse(input: &str) -> Vec<SpringRow> {
    input.lines().map(|l| l.parse().unwrap()).collect()
}

/// A collection of solvers that seemed fun or interesting but turned out not to
/// be good enough for the actual problem (especially part 2, for which it's
/// unfeasible to explicitly enumerate all matching patterns).
mod solving_the_bad_way {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Bits {
        bits: u128,
        size: u32,
    }

    impl Bits {
        fn new(bits: u128, size: u32) -> Self {
            assert!(size <= 128);
            Self { bits, size }
        }

        fn iter(&self) -> impl Iterator<Item = bool> + '_ {
            (0..self.size).map(|i| self.get(i as usize))
        }

        fn get(&self, index: usize) -> bool {
            debug_assert!(index < self.size as usize);
            self.bits & (1 << (self.size - index as u32 - 1)) != 0
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Arrangement(Bits);

    impl Arrangement {
        fn from_bits(bits: u128, size: usize) -> Self {
            Self(Bits::new(bits, size as u32))
        }

        fn damaged(&self) -> usize {
            self.0.bits.count_ones() as usize
        }

        fn springs(&self) -> impl Iterator<Item = Spring> + '_ {
            self.0.iter().map(|b| {
                if b {
                    Spring::Damaged
                } else {
                    Spring::Operational
                }
            })
        }

        fn matches_known_damaged(&self, damaged_groups: &[usize]) -> bool {
            debug_assert_ne!(damaged_groups.len(), 0);
            let mut group_index = 0;
            let mut consecutive_damaged = 0;
            for (i, spring) in self.springs().enumerate() {
                match spring {
                    Spring::Operational if consecutive_damaged == 0 => {}
                    Spring::Operational => {
                        if consecutive_damaged != damaged_groups[group_index] {
                            return false;
                        }
                        group_index += 1;
                        if group_index == damaged_groups.len() {
                            return self.springs().skip(i).all(|s| s == Spring::Operational);
                        }
                        consecutive_damaged = 0;
                    }
                    Spring::Damaged => {
                        consecutive_damaged += 1;
                    }
                    Spring::Unknown => {
                        unreachable!()
                    }
                }
            }
            group_index == damaged_groups.len() - 1
                && consecutive_damaged == damaged_groups[group_index]
        }

        fn matches_pattern(&self, pattern: &[Spring]) -> bool {
            assert!(pattern.len() == self.0.size as usize);
            pattern
                .iter()
                .zip(self.springs())
                .all(|(a, b)| a.matches(b))
        }

        fn matches_row(&self, row: &SpringRow) -> bool {
            self.matches_known_damaged(&row.known_damaged) && self.matches_pattern(&row.pattern)
        }
    }

    struct EnumerateArrangments {
        size: usize,
        current: u128,
    }

    impl EnumerateArrangments {
        fn new(size: usize) -> Self {
            assert!(size > 0 && size <= 128);
            Self { size, current: 0 }
        }
    }

    impl Iterator for EnumerateArrangments {
        type Item = Arrangement;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current == 1 << self.size {
                None
            } else {
                let result = Some(Arrangement::from_bits(self.current, self.size));
                self.current += 1;
                result
            }
        }
    }

    fn matching_arrangements(row: &SpringRow) -> impl Iterator<Item = Arrangement> + '_ {
        let total_damaged: usize = row.known_damaged.iter().sum();
        let size = row.pattern.len();
        let arrangements = EnumerateArrangments::new(size);
        arrangements.filter(move |arr| arr.damaged() == total_damaged && arr.matches_row(row))
    }

    fn bits_from_stars_and_bars(size: usize, zeroes: &[usize], ones: &[usize]) -> u128 {
        let mut shift = size;
        let mut result = 0;

        for (&zeroes, &ones) in zeroes.iter().zip(ones) {
            shift -= zeroes;
            for _ in 0..ones {
                result |= 1 << (shift - 1);
                shift -= 1;
            }
        }
        result
    }
    fn enumerate_stars_and_bars(size: usize, constraints: &[usize], pattern: &[Spring]) -> usize {
        // stars and bars problem, or "allocate N balls in K buckets"
        let max_buckets = constraints.len();
        let balls = size - constraints.iter().sum::<usize>();

        let mut result = 0;

        println!("enumerate_stars_and_bars({}, {:?})", size, constraints);
        println!(
            "binomial coefficient: {}",
            binomial_coeffiecient(balls - 1, max_buckets - 1)
        );

        #[derive(Debug, Clone)]
        struct StackItem {
            current: [usize; 128],
            level: usize,
            sum: usize,
        }

        impl StackItem {
            fn new(start_value: usize) -> Self {
                let mut current = [0; 128];
                current[0] = start_value;
                Self {
                    current,
                    level: 0,
                    sum: start_value,
                }
            }

            fn push(&mut self, value: usize) {
                self.level += 1;
                self.current[self.level] = value;
                self.sum += value;
            }

            fn values(&self) -> &[usize] {
                &self.current[..=self.level]
            }
        }

        for buckets in max_buckets.saturating_sub(1)..=max_buckets {
            // depth-first enumeration
            let mut stack = Vec::new();
            // dbg!(balls, buckets, size);
            stack.extend((0..=balls - 1).map(StackItem::new));
            while let Some(current) = stack.pop() {
                if current.level == buckets - 1 {
                    let mut next = current.clone();
                    next.push(balls - current.sum);
                    if buckets < max_buckets {
                        next.push(0);
                    }
                    let bits = bits_from_stars_and_bars(size, next.values(), constraints);
                    // result.push(bits);
                    let arr = Arrangement::from_bits(bits, size);
                    if arr.matches_pattern(pattern) {
                        result += 1;
                    }
                } else {
                    for n in 1..(balls - current.sum) {
                        let mut next = current.clone();
                        next.push(n);
                        stack.push(next);
                    }
                }
            }
        }
        result
    }

    #[allow(unused)]
    pub fn solve_the_bad_way(row: &SpringRow) -> u64 {
        matching_arrangements(row).count() as u64
    }

    #[allow(unused)]
    pub fn solve_the_second_worst_way(row: &SpringRow) -> u64 {
        enumerate_stars_and_bars(row.pattern.len(), &row.known_damaged, &row.pattern) as u64
    }

    #[allow(unused)]
    pub(super) fn solve_depth_first(row: &SpringRow) -> u64 {
        // This is actually kinda similar to the solve_recursive() that I ended
        // up using, but in its current state it's unusable for part 2 because
        // it would be hard to cache/memoize.

        let mut result = 0;

        #[derive(Debug, Clone)]
        struct StackItem {
            partial: [Spring; 128],
            next_free: usize,
            current_group: usize,
            residual_damaged: usize,
        }

        impl StackItem {
            fn new(total_damaged: usize) -> Self {
                Self {
                    partial: [Spring::Operational; 128],
                    next_free: 0,
                    current_group: 0,
                    residual_damaged: total_damaged,
                }
            }
            fn is_valid(&self, pattern: &[Spring]) -> bool {
                self.partial.iter().zip(pattern).all(|(a, b)| a.matches(*b))
            }
            fn add_group(&mut self, position: usize, length: usize) {
                debug_assert!(position >= self.next_free);
                for i in position..position + length {
                    self.partial[i] = Spring::Damaged;
                }
                self.next_free = position + length;
                self.current_group += 1;
                self.residual_damaged -= length;
            }
            fn advance(&mut self) {
                self.next_free += 1;
            }
            fn fill(&mut self, pattern_length: usize) {
                self.next_free = pattern_length;
            }
        }

        let total_damaged: usize = row.known_damaged.iter().sum();
        let mut stack = vec![StackItem::new(total_damaged)];
        while let Some(current) = stack.pop() {
            let group_length = row.known_damaged[current.current_group];
            for i in current.next_free
                ..=(row.pattern.len()
                    - current.residual_damaged
                    - (row.known_damaged.len() - current.current_group - 1))
            {
                let next = if current.current_group + 1 == row.known_damaged.len() {
                    let mut next = current.clone();
                    next.add_group(i, group_length);
                    next.fill(row.pattern.len());
                    next
                } else {
                    let mut next = current.clone();
                    next.add_group(i, group_length);
                    next.advance();
                    next
                };

                let length = row.pattern.len().min(next.next_free);
                let is_valid = next.is_valid(&row.pattern[..length]);
                if is_valid {
                    if next.current_group == row.known_damaged.len() {
                        result += 1;
                    } else {
                        stack.push(next);
                    }
                }
            }
        }
        result
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use Spring::{Damaged, Operational, Unknown};

        #[test]
        fn from_bits() {
            assert_eq!(
                Arrangement::from_bits(0, 4).springs().collect::<Vec<_>>(),
                &[Operational, Operational, Operational, Operational]
            );
            assert_eq!(
                Arrangement::from_bits(0b1111, 4)
                    .springs()
                    .collect::<Vec<_>>(),
                &[Damaged, Damaged, Damaged, Damaged]
            );
            assert_eq!(
                Arrangement::from_bits(0b1010, 4)
                    .springs()
                    .collect::<Vec<_>>(),
                &[Damaged, Operational, Damaged, Operational]
            );
        }

        #[test]
        fn matches_pattern() {
            let a = Arrangement::from_bits;
            assert!(a(0b1111, 4).matches_pattern(&[Damaged; 4]));
            assert!(a(0b1111, 4).matches_pattern(&[Unknown; 4]));
            assert!(a(0b1010, 4).matches_pattern(&[Damaged, Operational, Damaged, Operational]));
            assert!(a(0b1010, 4).matches_pattern(&[Unknown, Unknown, Damaged, Operational]));
            // cases that caused some bugs at some point
            let pattern: Vec<_> = "????.#...#... 4,1,1".parse::<SpringRow>().unwrap().pattern;
            assert!(a(0b1111010001000, 13).matches_pattern(&pattern));
        }

        #[test]
        fn matches_row() {
            let a = Arrangement::from_bits;
            assert!(a(0b1010, 4).matches_row(&"???? 1,1".parse().unwrap()));
            assert!(a(0b1101, 4).matches_row(&"???? 2,1".parse().unwrap()));
            // cases that caused some bugs at some point
            assert!(a(0b1111010001000, 13).matches_row(&"????.#...#... 4,1,1".parse().unwrap()));
            assert!(!a(0b011100000011, 12).matches_row(&"?###???????? 3,2,1".parse().unwrap()));
        }

        #[test]
        fn very_specific_broken_case() {
            let row: SpringRow = "?###???????? 3,2,1".parse().unwrap();
            let matching = matching_arrangements(&row).collect::<Vec<_>>();
            for arr in &matching {
                println!("{}", DisplayRow(&arr.springs().collect::<Vec<_>>()));
            }
            assert_eq!(matching.len(), 10);
        }

        #[test]
        fn matches_known_damaged() {
            let a = Arrangement::from_bits;
            assert!(a(0b1111, 4).matches_known_damaged(&[4]));
            assert!(!a(0b1111, 4).matches_known_damaged(&[3]));
            assert!(!a(0b1111, 4).matches_known_damaged(&[2, 2]));
            assert!(a(0b1010, 4).matches_known_damaged(&[1, 1]));
            assert!(!a(0b1010, 4).matches_known_damaged(&[1]));
            assert!(!a(0b1010, 4).matches_known_damaged(&[2]));
            // cases that caused some bugs at some point
            assert!(a(0b1111010001000, 13).matches_known_damaged(&[4, 1, 1]));
            assert!(a(0b011100000011, 12).matches_known_damaged(&[3, 2]));
            assert!(!a(0b011100000011, 12).matches_known_damaged(&[3, 2, 1]));
        }

        #[test]
        fn enumerate_arrangements() {
            const BITS: usize = 5;
            const PATTERNS: usize = 1 << BITS;
            let arrangements: Vec<_> = EnumerateArrangments::new(BITS).collect();
            assert_eq!(arrangements.len(), PATTERNS);
            for i in 0..PATTERNS {
                assert_eq!(arrangements[i], Arrangement::from_bits(i as u128, BITS));
            }
        }

        #[test]
        fn compare_solvers() {
            let input = include_str!("../input/2023/day12.txt");
            let rows = parse(input);
            for row in &rows {
                let result1 = solve_depth_first(row);
                let result2 = solve_the_second_worst_way(row);
                assert_eq!(
                    result1, result2,
                    "{row:?} depth_first={result1} old_way={result2}"
                );
            }
        }

        #[test]
        fn depth_first_broken_case() {
            let row: SpringRow = "????????#????#?.# 1,2,3,2,1".parse().unwrap();
            let result = solve_depth_first(&row);
            assert_eq!(result, 38);
        }
    }
}

fn is_valid(partial: &[Spring], pattern: &[Spring]) -> bool {
    partial.iter().zip(pattern).all(|(a, b)| a.matches(*b))
}

fn solve_partial(
    partial: &mut [Spring; 128],
    start: usize,
    current_group: usize,
    residual_damaged: usize,
    cache: &mut HashMap<(usize, usize), u64>,
    row: &SpringRow,
) -> u64 {
    let key = (start, current_group);
    if let Some(&result) = cache.get(&key) {
        return result;
    }
    let last_group = row.known_damaged.len() - 1;
    let group_length = row.known_damaged[current_group];
    let last_free =
        row.pattern.len() - residual_damaged - (row.known_damaged.len() - current_group - 1);
    let new_residual_damaged = residual_damaged - group_length;
    // println!("group {current_group} (len {group_length}) next_free {next_free} {}", DisplayRow(&partial[..next_free]));
    let result = (start..=last_free)
        .map(|i| {
            // clippy has a preference for loops that don't explicitly index the
            // array and I'm complying but there is probably a cuter way to
            // write this
            for s in partial.iter_mut().take(i).skip(start) {
                *s = Spring::Operational;
            }
            for s in partial.iter_mut().skip(i).take(group_length) {
                *s = Spring::Damaged;
            }
            let length = if current_group == last_group {
                // fill the rest with operational springs
                for s in partial.iter_mut().skip(i + group_length) {
                    *s = Spring::Operational;
                }
                row.pattern.len()
            } else {
                // add a single operational spring as a separator
                partial[i + group_length] = Spring::Operational;
                i + group_length + 1
            };
            // println!("   position {i} {}", DisplayRow(&partial[..length]));
            debug_assert!(length <= row.pattern.len());
            debug_assert!(length > start);
            let is_valid = is_valid(&partial[start..length], &row.pattern[start..length]);
            if is_valid {
                if current_group == last_group {
                    // println!("MATCH: {}", DisplayRow(&partial[..length]));
                    1
                } else {
                    solve_partial(
                        partial,
                        length,
                        current_group + 1,
                        new_residual_damaged,
                        cache,
                        row,
                    )
                }
            } else {
                0
            }
        })
        .sum();
    cache.insert(key, result);
    result
}

fn solve_recursive(row: &SpringRow) -> u64 {
    let mut cache = HashMap::new();
    let total_damaged: usize = row.known_damaged.iter().sum();
    solve_partial(
        &mut [Spring::Operational; 128],
        0,
        0,
        total_damaged,
        &mut cache,
        row,
    )
}

#[aoc(day12, part1)]
fn part1(input: &[SpringRow]) -> u64 {
    input.iter().map(solve_recursive).sum()
}

#[aoc(day12, part2)]
fn part2(input: &[SpringRow]) -> u64 {
    let unfolded: Vec<_> = input.iter().map(|row| row.clone().repeat(5)).collect();
    unfolded.iter().map(solve_recursive).sum()
}

fn binomial_coeffiecient(n: usize, k: usize) -> usize {
    let mut result = 1;
    for i in 0..k {
        result *= n - i;
        result /= i + 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_example() {
        let solve = |input| part1(&parse(input));
        assert_eq!(solve("???.### 1,1,3"), 1);
        assert_eq!(solve(".??..??...?##. 1,1,3"), 4);
        assert_eq!(solve("?#?#?#?#?#?#?#? 1,3,1,6"), 1);
        assert_eq!(solve("????.#...#... 4,1,1"), 1);
        assert_eq!(solve("????.######..#####. 1,6,5"), 4);
        assert_eq!(solve("?###???????? 3,2,1"), 10);
    }

    #[test]
    fn part2_example_already_unfolded() {
        let solve = |input| part1(&parse(input));
        assert_eq!(
            solve("???.###????.###????.###????.###????.### 1,1,3,1,1,3,1,1,3,1,1,3,1,1,3"),
            1
        );
    }

    #[test]
    fn part2_tricky() {
        let solve = |input| part2(&parse(input));
        let _ = dbg!(solve("????.??.??. 1,1"));
    }

    #[test]
    fn part2_example() {
        let solve = |input| part2(&parse(input));
        assert_eq!(solve("???.### 1,1,3"), 1);
        assert_eq!(solve(".??..??...?##. 1,1,3"), 16384);
        assert_eq!(solve("?#?#?#?#?#?#?#? 1,3,1,6"), 1);
        assert_eq!(solve("????.#...#... 4,1,1"), 16);
        assert_eq!(solve("????.######..#####. 1,6,5"), 2500);
        assert_eq!(solve("?###???????? 3,2,1"), 506250);
    }

    #[test]
    fn parse_row() {
        use Spring::{Damaged as D, Operational as O, Unknown as U};

        let row: SpringRow = "???.### 1,1,3".parse().unwrap();
        assert_eq!(&row.pattern, &[U, U, U, O, D, D, D]);
        assert_eq!(&row.known_damaged, &[1, 1, 3],);

        let row: SpringRow = "????.#...#... 4,1,1".parse().unwrap();
        assert_eq!(&row.pattern, &[U, U, U, U, O, D, O, O, O, D, O, O, O]);
        assert_eq!(&row.known_damaged, &[4, 1, 1]);
    }

    #[test]
    fn unfold_row() {
        let row: SpringRow = ".# 1".parse::<SpringRow>().unwrap().repeat(5);
        let already_unfolded_row = ".#?.#?.#?.#?.# 1,1,1,1,1".parse::<SpringRow>().unwrap();
        assert_eq!(row.pattern, already_unfolded_row.pattern);
        assert_eq!(row.known_damaged, already_unfolded_row.known_damaged);

        let row = "???.### 1,1,3".parse::<SpringRow>().unwrap().repeat(5);
        let already_unfolded_row =
            "???.###????.###????.###????.###????.### 1,1,3,1,1,3,1,1,3,1,1,3,1,1,3"
                .parse::<SpringRow>()
                .unwrap();
        assert_eq!(row.pattern, already_unfolded_row.pattern);
        assert_eq!(row.known_damaged, already_unfolded_row.known_damaged);
    }

    #[test]
    fn specific_thingy_that_takes_a_long_time() {
        let row: SpringRow = "???.??##?????.????? 1,4,1,1,1,1".parse().unwrap();
        let result = solve_recursive(&row);
        assert_eq!(result, 101);

        for i in 1..=5 {
            let row = row.clone().repeat(i);
            let result = solve_recursive(&row);
            println!("{i}: {result}");
        }
    }
}

example_tests! {
    "
    ???.### 1,1,3
    .??..??...?##. 1,1,3
    ?#?#?#?#?#?#?#? 1,3,1,6
    ????.#...#... 4,1,1
    ????.######..#####. 1,6,5
    ?###???????? 3,2,1
    ",
    part1 => 21,
    part2 => 525152,
}

known_input_tests! {
    input: include_str!("../input/2023/day12.txt"),
    part1 => 7251,
    part2 => 2128386729962,
}
