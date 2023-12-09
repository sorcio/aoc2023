use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::{example_tests, known_input_tests};

#[aoc_generator(day9)]
fn parse(input: &str) -> Vec<Vec<i64>> {
    input
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|number: &str| number.parse().unwrap())
                .collect()
        })
        .collect()
}

fn differences(line: &[i64]) -> Vec<i64> {
    line.windows(2)
        .map(|window| window[1] - window[0])
        .collect()
}

fn extrapolate_line(line: &[i64]) -> i64 {
    let mut differences_stack = vec![line.to_vec()];
    loop {
        let differences = differences(differences_stack.last().unwrap());
        if differences.iter().all(|&difference| difference == 0) {
            break;
        }
        differences_stack.push(differences);
    }
    differences_stack
        .iter()
        .map(|differences| differences.last().unwrap())
        .sum()
}

fn extrapolate_line_back(line: &[i64]) -> i64 {
    let mut differences_stack = vec![line.to_vec()];
    loop {
        let differences = differences(differences_stack.last().unwrap());
        if differences.iter().all(|&difference| difference == 0) {
            break;
        }
        differences_stack.push(differences);
    }
    differences_stack
        .iter()
        .rev()
        .map(|differences| differences[0])
        .reduce(|a, b| b - a)
        .unwrap()
}

#[aoc(day9, part1)]
fn part1(input: &[Vec<i64>]) -> i64 {
    input.iter().map(|line| extrapolate_line(line)).sum()
}

#[aoc(day9, part2)]
fn part2(input: &[Vec<i64>]) -> i64 {
    input.iter().map(|line| extrapolate_line_back(line)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrapolate() {
        assert_eq!(extrapolate_line(&[0, 3, 6, 9, 12, 15]), 18);
        assert_eq!(extrapolate_line(&[1, 3, 6, 10, 15, 21]), 28);
        assert_eq!(extrapolate_line(&[10, 13, 16, 21, 30, 45]), 68);
    }

    #[test]
    fn extrapolate_back() {
        assert_eq!(extrapolate_line_back(&[0, 3, 6, 9, 12, 15]), -3);
        assert_eq!(extrapolate_line_back(&[1, 3, 6, 10, 15, 21]), 0);
        assert_eq!(extrapolate_line_back(&[10, 13, 16, 21, 30, 45]), 5);
    }
}

example_tests! {
    "
    0 3 6 9 12 15
    1 3 6 10 15 21
    10 13 16 21 30 45
    ",

    part1 => 114,
    part2 => 2,
}

known_input_tests! {
    input: include_str!("../input/2023/day9.txt"),
    part1 => 1725987467,
    part2 => 971,
}
