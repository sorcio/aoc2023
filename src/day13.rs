use std::str::FromStr;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::example_tests;

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

#[aoc_generator(day13)]
fn parse(input: &str) -> Vec<Mirror> {
    input.split("\n\n").map(|s| s.parse().unwrap()).collect()
}

#[aoc(day13, part1)]
fn part1(input: &[Mirror]) -> usize {
    input
        .iter()
        .enumerate()
        .inspect(|(i, m)| {
            println!(
                "Mirror {}:\n{}\n------------------------------",
                i,
                DisplayGrid(*m)
            );
        })
        .map(|(i, m)| {
            if let Some(cols) = find_reflection(m) {
                cols
            } else {
                100 * find_reflection(HorizontalMiror(m)).expect(&format!(
                    "mirror {i} should be either vertical or horizontal"
                ))
            }
        })
        .sum()
}

#[aoc(day13, part2)]
fn part2(_input: &[Mirror]) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_example_vertical() {
        let input = unindent::unindent(
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
        let mirrors = parse(&input);
        assert_eq!(mirrors.len(), 1);
        assert_eq!(find_reflection(&mirrors[0]), Some(5));
        assert_eq!(find_reflection(HorizontalMiror(&mirrors[0])), None);
    }

    #[test]
    fn part1_example_horizontal() {
        let input = unindent::unindent(
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
        let mirrors = parse(&input);
        assert_eq!(mirrors.len(), 1);
        assert_eq!(find_reflection(&mirrors[0]), None);
        assert_eq!(find_reflection(HorizontalMiror(&mirrors[0])), Some(4));
    }

    #[test]
    fn part1_case_1() {
        // first mirror in input file
        let input = unindent::unindent(
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
        let [ref mirror] = parse(&input)[..] else { panic!() };
        assert_eq!(find_reflection(mirror), Some(11));
        assert_eq!(find_reflection(HorizontalMiror(mirror)), None);
    }

    #[test]
    fn part1_case_2() {
        // second mirror in input file
        let input = unindent::unindent(
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
        let mirrors = parse(&input);
        assert_eq!(mirrors.len(), 1);
        assert_eq!(find_reflection(&mirrors[0]), Some(8));
        assert_eq!(find_reflection(HorizontalMiror(&mirrors[0])), None);
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
