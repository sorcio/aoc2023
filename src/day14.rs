use std::ops::Rem;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::{
    testing::{example_tests, known_input_tests},
    utils::{grid_cell_enum, AsciiUtils, FromGridLike},
};

grid_cell_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Tile {
        Empty => b'.',
        Obstacle => b'#',
        Ball => b'O',
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Grid {
    tiles: Vec<Tile>,
    width: usize,
    height: usize,
}

impl FromGridLike for Grid {
    type Cell = Tile;
    fn from_cells(tiles: Vec<Self::Cell>, width: usize, height: usize) -> Self {
        Self {
            tiles,
            width,
            height,
        }
    }
}

enum RollDirection {
    North,
    South,
    East,
    West,
}

impl Grid {
    fn clone_without_balls(&self) -> Self {
        let mut tiles = Vec::with_capacity(self.tiles.len());
        tiles.extend(self.tiles.iter().map(|&tile| {
            if tile == Tile::Ball {
                Tile::Empty
            } else {
                tile
            }
        }));
        Self {
            tiles,
            width: self.width,
            height: self.height,
        }
    }

    fn get(&self, x: usize, y: usize) -> Tile {
        self.tiles[y * self.width + x]
    }

    fn set(&mut self, x: usize, y: usize, tile: Tile) {
        debug_assert!(self.get(x, y) == Tile::Empty);
        self.tiles[y * self.width + x] = tile;
    }

    fn roll(&self, mut new_grid: Grid, direction: RollDirection) -> Self {
        // Very repetitive code but I can't be bothered to make it generic and
        // there are little differences between positive directions (nort/west)
        // and negative directions (south/east) because we need to account for
        // (un)signedness.
        match direction {
            RollDirection::North => {
                for x in 0..self.width {
                    let mut first_empty = 0;
                    for y in 0..self.height {
                        match self.get(x, y) {
                            Tile::Empty => {}
                            Tile::Ball => {
                                new_grid.set(x, first_empty, Tile::Ball);
                                first_empty += 1;
                            }
                            Tile::Obstacle => {
                                first_empty = y + 1;
                            }
                        }
                    }
                }
            }
            RollDirection::South => {
                for x in 0..self.width {
                    let mut last_obstacle = self.height;
                    for y in (0..self.height).rev() {
                        match self.get(x, y) {
                            Tile::Empty => {}
                            Tile::Ball => {
                                new_grid.set(x, last_obstacle - 1, Tile::Ball);
                                last_obstacle -= 1;
                            }
                            Tile::Obstacle => {
                                last_obstacle = y;
                            }
                        }
                    }
                }
            }
            RollDirection::West => {
                for y in 0..self.height {
                    let mut first_empty = 0;
                    for x in 0..self.width {
                        match self.get(x, y) {
                            Tile::Empty => {}
                            Tile::Ball => {
                                new_grid.set(first_empty, y, Tile::Ball);
                                first_empty += 1;
                            }
                            Tile::Obstacle => {
                                first_empty = x + 1;
                            }
                        }
                    }
                }
            }
            RollDirection::East => {
                for y in 0..self.height {
                    let mut last_obstacle = self.width;
                    for x in (0..self.width).rev() {
                        match self.get(x, y) {
                            Tile::Empty => {}
                            Tile::Ball => {
                                new_grid.set(last_obstacle - 1, y, Tile::Ball);
                                last_obstacle -= 1;
                            }
                            Tile::Obstacle => {
                                last_obstacle = x;
                            }
                        }
                    }
                }
            }
        }
        new_grid
    }

    fn roll_cycle(&self, template: &Grid) -> Self {
        let mut rolled = self.roll(template.clone(), RollDirection::North);
        rolled = rolled.roll(template.clone(), RollDirection::West);
        rolled = rolled.roll(template.clone(), RollDirection::South);
        rolled = rolled.roll(template.clone(), RollDirection::East);
        rolled
    }

    fn weight(&self) -> usize {
        let mut row_weight = self.height;
        let mut total_weight = 0;
        for row in self.tiles.chunks(self.width) {
            total_weight += row_weight * row.iter().filter(|&&tile| tile == Tile::Ball).count();
            row_weight -= 1;
        }
        total_weight
    }
}

struct DisplayGrid<'a>(&'a Grid);

impl core::fmt::Display for DisplayGrid<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.0.height {
            for x in 0..self.0.width {
                write!(f, "{}", self.0.get(x, y))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[aoc_generator(day14)]
fn parse(input: &[u8]) -> Grid {
    input.grid_like().unwrap().into_grid()
}

#[aoc(day14, part1)]
fn part1(grid: &Grid) -> usize {
    let template = grid.clone_without_balls();
    let rolled = grid.roll(template, RollDirection::North);
    rolled.weight()
}

#[aoc(day14, part2)]
fn part2(grid: &Grid) -> usize {
    let template = grid.clone_without_balls();
    let mut history = std::collections::HashMap::new();
    let mut rolled = grid.clone();
    const TARGET_ROLL_CYCLES: usize = 1_000_000_000;
    for i in 0..TARGET_ROLL_CYCLES {
        if let Some(&prev_i) = history.get(&rolled) {
            let remaining = (TARGET_ROLL_CYCLES - i).rem(i - prev_i);
            for _ in 0..remaining {
                rolled = rolled.roll_cycle(&template);
            }
            break;
        } else {
            history.insert(rolled.clone(), i);
        }
        rolled = rolled.roll_cycle(&template);
    }
    rolled.weight()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_unindented(input: &'static [u8]) -> Grid {
        let input = unindent::unindent_bytes(input);
        parse(&input)
    }

    const EXAMPLE: &'static [u8] = b"
        O....#....
        O.OO#....#
        .....##...
        OO.#O....O
        .O.....O#.
        O.#..O.#.#
        ..O..#O..O
        .......O..
        #....###..
        #OO..#....
        ";

    #[test]
    fn part1_roll_north() {
        let input = parse_unindented(EXAMPLE);
        let expected = parse_unindented(
            b"
            OOOO.#.O..
            OO..#....#
            OO..O##..O
            O..#.OO...
            ........#.
            ..#....#.#
            ..O..#.O.O
            ..O.......
            #....###..
            #....#....
            ",
        );
        let result = input.roll(input.clone_without_balls(), RollDirection::North);
        assert_eq!(result, expected);
    }

    #[test]
    fn part1_roll_south() {
        let input = parse_unindented(EXAMPLE);
        let expected = parse_unindented(
            b"
            .....#....
            ....#....#
            ...O.##...
            ...#......
            O.O....O#O
            O.#..O.#.#
            O....#....
            OO....OO..
            #OO..###..
            #OO.O#...O
            ",
        );
        let result = input.roll(input.clone_without_balls(), RollDirection::South);
        assert_eq!(result, expected);
    }

    #[test]
    fn part1_roll_east() {
        let input = parse_unindented(EXAMPLE);
        let expected = parse_unindented(
            b"
            ....O#....
            .OOO#....#
            .....##...
            .OO#....OO
            ......OO#.
            .O#...O#.#
            ....O#..OO
            .........O
            #....###..
            #..OO#....
            ",
        );
        let result = input.roll(input.clone_without_balls(), RollDirection::East);
        assert_eq!(result, expected);
    }

    #[test]
    fn part1_roll_west() {
        let input = parse_unindented(EXAMPLE);
        let expected = parse_unindented(
            b"
            O....#....
            OOO.#....#
            .....##...
            OO.#OO....
            OO......#.
            O.#O...#.#
            O....#OO..
            O.........
            #....###..
            #OO..#....
            ",
        );
        let result = input.roll(input.clone_without_balls(), RollDirection::West);
        assert_eq!(result, expected);
    }
}

example_tests! {
    b"
    O....#....
    O.OO#....#
    .....##...
    OO.#O....O
    .O.....O#.
    O.#..O.#.#
    ..O..#O..O
    .......O..
    #....###..
    #OO..#....
    ",

    part1 => 136,
    part2 => 64,
}

known_input_tests! {
    input: include_bytes!("../input/2023/day14.txt"),
    part1 => 109654,
    part2 => 94876,
}
