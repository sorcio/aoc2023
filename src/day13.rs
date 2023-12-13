use std::str::FromStr;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::{example_tests, known_input_tests};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Ash,
    Rock,
}

impl TryFrom<char> for Tile {
    type Error = ();
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '#' => Ok(Tile::Rock),
            '.' => Ok(Tile::Ash),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Ash => write!(f, "⬜️"),
            Tile::Rock => write!(f, "⬛️"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: usize,
    y: usize,
}

fn pos(x: usize, y: usize) -> Pos {
    Pos { x, y }
}

#[derive(Debug, Clone)]
struct Mirror {
    data: Vec<Tile>,
    width: usize,
    height: usize,
}

impl FromStr for Mirror {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = s.trim().chars().filter_map(|c| c.try_into().ok()).collect();
        let width = s.lines().next().unwrap().len();
        let height = s.lines().count();
        Ok(Mirror {
            data,
            width,
            height,
        })
    }
}

trait Grid {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, position: Pos) -> Tile;
}

impl Grid for &Mirror {
    #[track_caller]
    fn get(&self, position: Pos) -> Tile {
        self.data[position.y * self.width + position.x]
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

/// Flip x-y coordinates of a mirror
struct HorizontalMiror<'a>(&'a Mirror);

impl Grid for HorizontalMiror<'_> {
    #[track_caller]
    fn get(&self, position: Pos) -> Tile {
        // invert x and y
        self.0.get(pos(position.y, position.x))
    }

    fn width(&self) -> usize {
        self.0.height()
    }

    fn height(&self) -> usize {
        self.0.width()
    }
}

fn find_reflection<G: Grid>(grid: G) -> Option<usize> {
    for x in 1..grid.width() {
        let width = (grid.width() - x).min(x);
        debug_assert!(width > 0);
        let found = (0..grid.height()).all(|y| {
            (0..width)
                .map(|i| (grid.get(pos(x - i - 1, y)), grid.get(pos(x + i, y))))
                .all(|(a, b)| a == b)
        });
        if found {
            return Some(x);
        }
    }
    None
}

fn find_reflection_with_tolerance<G: Grid>(grid: G, tolerance: u32) -> Option<usize> {
    // Squish columns into bitfields to make comparisons cheaper. But I never
    // proved that this is actually faster, but it works fine for counting with
    // tolerance, so I'm keeping it.
    debug_assert!(grid.width() <= 64);
    debug_assert!(grid.height() <= 64);
    let mut columns = [0; 64];
    (0..grid.width()).for_each(|x| {
        columns[x] = (0..grid.height())
            .map(|y| grid.get(pos(x, y)))
            .fold(0, |acc, tile| (acc << 1) | (tile == Tile::Rock) as u64);
    });
    for x in 1..grid.width() {
        let width = (grid.width() - x).min(x);
        debug_assert!(width > 0);
        let found: u32 = (0..width)
            .map(|i| (columns[x - i - 1], columns[x + i]))
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();
        if found == tolerance {
            return Some(x);
        }
    }
    None
}

struct DisplayGrid<G: Grid>(G);

impl<G: Grid> std::fmt::Display for DisplayGrid<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "width: {}, height: {}", self.0.width(), self.0.height())?;
        for y in 0..self.0.height() {
            for x in 0..self.0.width() {
                write!(f, "{}", self.0.get(pos(x, y)))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn parse_mirrors(input: &str) -> impl Iterator<Item = Mirror> + '_ {
    input.split("\n\n").map(|s| s.parse().unwrap())
}

#[aoc_generator(day13)]
fn parse(input: &str) -> Vec<Mirror> {
    parse_mirrors(input).collect()
}

#[aoc(day13, part1)]
fn part1(input: &[Mirror]) -> usize {
    input
        .iter()
        .enumerate()
        .inspect(|(_i, _m)| {
            #[cfg(feature = "extra-debug-prints")]
            println!(
                "Mirror {}:\n{}\n------------------------------",
                _i,
                DisplayGrid(*_m)
            );
        })
        .map(|(i, m)| {
            if let Some(cols) = find_reflection(m) {
                cols
            } else {
                100 * find_reflection(HorizontalMiror(m))
                    .unwrap_or_else(|| panic!("mirror {i} should be either vertical or horizontal"))
            }
        })
        .sum()
}

#[aoc(day13, part1, bit_columns)]
fn part1_bit_columns(input: &[Mirror]) -> usize {
    input
        .iter()
        .enumerate()
        .inspect(|(_i, _m)| {
            #[cfg(feature = "extra-debug-prints")]
            println!(
                "Mirror {}:\n{}\n------------------------------",
                _i,
                DisplayGrid(*_m)
            );
        })
        .map(|(i, m)| {
            if let Some(cols) = find_reflection_with_tolerance(m, 0) {
                cols
            } else {
                100 * find_reflection_with_tolerance(HorizontalMiror(m), 0)
                    .unwrap_or_else(|| panic!("mirror {i} should be either vertical or horizontal"))
            }
        })
        .sum()
}

#[aoc(day13, part2)]
fn part2(input: &[Mirror]) -> usize {
    input
        .iter()
        .enumerate()
        .inspect(|(_i, _m)| {
            #[cfg(feature = "extra-debug-prints")]
            println!(
                "Mirror {}:\n{}\n------------------------------",
                _i,
                DisplayGrid(*_m)
            );
        })
        .map(|(i, m)| {
            if let Some(cols) = find_reflection_with_tolerance(m, 1) {
                cols
            } else {
                100 * find_reflection_with_tolerance(HorizontalMiror(m), 1)
                    .unwrap_or_else(|| panic!("mirror {i} should be either vertical or horizontal"))
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse a single mirror from a string literal, unindenting it first.
    fn parse_one_example(input: &'static str) -> Mirror {
        let input = unindent::unindent(input);
        let mut iter = parse_mirrors(&input);
        let mirror = iter
            .next()
            .expect("there should be at least one mirror in input");
        assert!(
            iter.next().is_none(),
            "there should be exactly one mirror in input"
        );
        mirror
    }

    #[test]
    fn part1_example_vertical() {
        let mirror = parse_one_example(
            "
            #.##..##.
            ..#.##.#.
            ##......#
            ##......#
            ..#.##.#.
            ..##..##.
            #.#.##.#.
            ",
        );
        assert_eq!(find_reflection(&mirror), Some(5));
        assert_eq!(find_reflection(HorizontalMiror(&mirror)), None);

        assert_eq!(find_reflection_with_tolerance(&mirror, 0), Some(5));
        assert_eq!(
            find_reflection_with_tolerance(HorizontalMiror(&mirror), 0),
            None
        );
    }

    #[test]
    fn part1_example_horizontal() {
        let mirror = parse_one_example(
            "
            #...##..#
            #....#..#
            ..##..###
            #####.##.
            #####.##.
            ..##..###
            #....#..#
            ",
        );
        assert_eq!(find_reflection(&mirror), None);
        assert_eq!(find_reflection(HorizontalMiror(&mirror)), Some(4));

        assert_eq!(find_reflection_with_tolerance(&mirror, 0), None);
        assert_eq!(
            find_reflection_with_tolerance(HorizontalMiror(&mirror), 0),
            Some(4)
        );
    }

    #[test]
    fn part1_case_1() {
        // first mirror in input file
        let mirror = parse_one_example(
            "
            ##..#..#......#
            .........#..#..
            .####.#.######.
            #....#.###..###
            ..##..#.#.##.#.
            ######...#..#..
            #.##.#.#.#..#.#
            #....#..######.
            .#..#...#.##.#.
            #....#....##...
            .#..#.#..####..
            ......#.######.
            ##..##.#.####.#
            ",
        );
        assert_eq!(find_reflection(&mirror), Some(11));
        assert_eq!(find_reflection(HorizontalMiror(&mirror)), None);

        assert_eq!(find_reflection_with_tolerance(&mirror, 0), Some(11));
        assert_eq!(
            find_reflection_with_tolerance(HorizontalMiror(&mirror), 0),
            None
        );
    }

    #[test]
    fn part1_case_2() {
        // second mirror in input file
        let mirror = parse_one_example(
            "
            .#...##..
            ..##.#.##
            .#.###...
            ###..#.##
            ##.#.####
            ..#.#..##
            .###...##
            .#...#.##
            #####.#..
            ...#..###
            ###.##.##
            ####...##
            ####..###
            ###.##.##
            ...#..###
            ",
        );
        assert_eq!(find_reflection(&mirror), Some(8));
        assert_eq!(find_reflection(HorizontalMiror(&mirror)), None);

        assert_eq!(find_reflection_with_tolerance(&mirror, 0), Some(8));
        assert_eq!(
            find_reflection_with_tolerance(HorizontalMiror(&mirror), 0),
            None
        );
    }

    #[test]
    fn part2_example() {
        let mirror = parse_one_example(
            "
            #.##..##.
            ..#.##.#.
            ##......#
            ##......#
            ..#.##.#.
            ..##..##.
            #.#.##.#.
            ",
        );
        assert_eq!(find_reflection_with_tolerance(&mirror, 1), None);
        assert_eq!(
            find_reflection_with_tolerance(HorizontalMiror(&mirror), 1),
            Some(3)
        );
    }
}

example_tests! {
    "
    #.##..##.
    ..#.##.#.
    ##......#
    ##......#
    ..#.##.#.
    ..##..##.
    #.#.##.#.
    
    #...##..#
    #....#..#
    ..##..###
    #####.##.
    #####.##.
    ..##..###
    #....#..#
    ",
    part1 => 405,
}

known_input_tests! {
    input: include_str!("../input/2023/day13.txt"),
    part1 => 37113,
    part2 => 30449,
}
