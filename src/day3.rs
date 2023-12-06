use std::collections::HashSet;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::{example_tests, known_input_tests};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct GridPos(usize);

struct Grid {
    data: Vec<u8>,
    row_length: usize,
}

fn is_symbol(b: u8) -> bool {
    b.is_ascii_graphic() && b != b'.' && !b.is_ascii_digit()
}

impl Grid {
    fn new(input: &[u8]) -> Self {
        let row_length = input
            .iter()
            .position(|&c| c == b'\n')
            .unwrap_or(input.len());
        Self {
            data: input.into(),
            row_length,
        }
    }

    #[cfg_attr(not(test), allow(unused))]
    fn pos(&self, row: usize, col: usize) -> GridPos {
        GridPos(row * (self.row_length + 1) + col)
    }

    fn symbols(&self) -> impl Iterator<Item = GridPos> + '_ {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(i, &b)| is_symbol(b).then_some(GridPos(i)))
    }

    fn is_star(&self, pos: GridPos) -> bool {
        self.data[pos.0] == b'*'
    }

    fn row_above(&self, pos: GridPos) -> Option<GridPos> {
        if pos.0 > self.row_length {
            Some(GridPos(pos.0 - self.row_length - 1))
        } else {
            None
        }
    }

    fn row_below(&self, pos: GridPos) -> Option<GridPos> {
        if pos.0 + self.row_length < self.data.len() - 1 {
            Some(GridPos(pos.0 + self.row_length + 1))
        } else {
            None
        }
    }

    fn col_left(&self, pos: GridPos) -> Option<GridPos> {
        if pos.0 > 0 && self.data[pos.0 - 1] != b'\n' {
            Some(GridPos(pos.0 - 1))
        } else {
            None
        }
    }

    fn col_right(&self, pos: GridPos) -> Option<GridPos> {
        if pos.0 < self.data.len() - 1 && self.data[pos.0 + 1] != b'\n' {
            Some(GridPos(pos.0 + 1))
        } else {
            None
        }
    }

    fn numbers_adjacent_to(&self, pos: GridPos) -> Vec<GridPos> {
        let mut numbers = Vec::new();
        // left/right
        if let Some(number) = self.col_left(pos).and_then(|pos| self.find_number(pos)) {
            numbers.push(number);
        }
        if let Some(number) = self.col_right(pos).and_then(|pos| self.find_number(pos)) {
            numbers.push(number);
        }
        // if a number is right above/below, we don't need to check
        // the diagonals because no other number can be there
        if let Some(above) = self.row_above(pos) {
            if let Some(number) = self.find_number(above) {
                numbers.push(number)
            } else {
                // diagonals
                if let Some(number) = self.col_left(above).and_then(|pos| self.find_number(pos)) {
                    numbers.push(number);
                }
                if let Some(number) = self.col_right(above).and_then(|pos| self.find_number(pos)) {
                    numbers.push(number);
                }
            }
        }
        if let Some(below) = self.row_below(pos) {
            if let Some(number) = self.find_number(below) {
                numbers.push(number)
            } else {
                // diagonals
                if let Some(number) = self.col_left(below).and_then(|pos| self.find_number(pos)) {
                    numbers.push(number);
                }
                if let Some(number) = self.col_right(below).and_then(|pos| self.find_number(pos)) {
                    numbers.push(number);
                }
            }
        }

        numbers
    }

    /// Find a number's starting position
    fn find_number(&self, pos: GridPos) -> Option<GridPos> {
        let pos = pos.0;
        if self.data[pos].is_ascii_digit() {
            let start = (0..pos)
                .rev()
                .find(|&i| !self.data[i].is_ascii_digit())
                .map(|x| x + 1)
                .unwrap_or(0);
            Some(GridPos(start))
        } else {
            None
        }
    }
    fn number_at(&self, pos: GridPos) -> Option<u32> {
        let pos = pos.0;
        if self.data[pos].is_ascii_digit() {
            let start = (0..pos)
                .rev()
                .find(|&i| !self.data[i].is_ascii_digit())
                .map(|x| x + 1)
                .unwrap_or(0);
            let end = (pos + 1..self.data.len())
                .find(|&i| !self.data[i].is_ascii_digit())
                .map(|x| x - 1)
                .expect("file should end with a line break");
            // SAFETY: we checked that string is made of ascii digits
            let s = unsafe { std::str::from_utf8_unchecked(&self.data[start..=end]) };
            Some(s.parse().expect("should be a valid number"))
        } else {
            None
        }
    }
}

#[aoc_generator(day3)]
fn parse(input: &[u8]) -> Grid {
    Grid::new(input)
}

#[aoc(day3, part1)]
fn part1(grid: &Grid) -> u32 {
    let numbers: HashSet<_> = grid
        .symbols()
        .flat_map(|symbol| grid.numbers_adjacent_to(symbol))
        .collect();
    numbers
        .iter()
        .map(|pos| {
            grid.number_at(*pos)
                .expect("should be a valid number position")
        })
        .sum()
}

#[aoc(day3, part2)]
fn part2(grid: &Grid) -> u32 {
    grid.symbols()
        .filter(|pos| grid.is_star(*pos))
        .map(|symbol| {
            let adjacent = grid.numbers_adjacent_to(symbol);
            match &adjacent[..] {
                &[g1, g2] => grid.number_at(g1).unwrap() * grid.number_at(g2).unwrap(),
                _ => 0,
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_grid(input: &[u8]) -> Grid {
        let data = unindent::unindent_bytes(input);
        Grid::new(&data)
    }

    #[test]
    fn grid_number_at() {
        let grid = make_test_grid(
            b"
        467..114..
        ...*......
        ..35..6345
        ",
        );

        assert_eq!(grid.number_at(grid.pos(0, 0)), Some(467));
        assert_eq!(grid.number_at(grid.pos(0, 1)), Some(467));
        assert_eq!(grid.number_at(grid.pos(0, 2)), Some(467));
        assert_eq!(grid.number_at(grid.pos(0, 3)), None);
        assert_eq!(grid.number_at(grid.pos(1, 0)), None);
        assert_eq!(grid.number_at(grid.pos(2, 3)), Some(35));
        assert_eq!(grid.number_at(grid.pos(2, 6)), Some(6345));
        assert_eq!(grid.number_at(grid.pos(2, 7)), Some(6345));
        assert_eq!(grid.number_at(grid.pos(2, 8)), Some(6345));
        assert_eq!(grid.number_at(grid.pos(2, 9)), Some(6345));
    }

    #[test]
    fn grid_adjacent_numbers() {
        let grid = make_test_grid(
            b"
        467..114..
        ...*......
        ..35..633.
        ......#...
        617*......
        .....+.58.
        ..592.....
        ......755.
        ...$.*....
        .664.598..
        ",
        );

        assert_eq!(
            grid.numbers_adjacent_to(grid.pos(0, 3)),
            vec![grid.pos(0, 0)]
        );
        assert_eq!(
            grid.numbers_adjacent_to(grid.pos(0, 4)),
            vec![grid.pos(0, 5)]
        );
        assert_eq!(grid.numbers_adjacent_to(grid.pos(0, 9)), vec![]);
        assert_eq!(
            grid.numbers_adjacent_to(grid.pos(1, 3)),
            vec![grid.pos(0, 0), grid.pos(2, 2)]
        );
    }

    #[test]
    fn grid_symbols() {
        let grid = make_test_grid(
            b"
        467..114..
        ...*......
        ..35..633.
        ......#...
        617*......
        .....+.58.
        ..592.....
        ......755.
        ...$.*....
        .664.598..
        ",
        );
        let symbols: Vec<_> = grid.symbols().collect();
        assert_eq!(
            symbols,
            vec![
                grid.pos(1, 3),
                grid.pos(3, 6),
                grid.pos(4, 3),
                grid.pos(5, 5),
                grid.pos(8, 3),
                grid.pos(8, 5)
            ]
        );
    }
}

example_tests! {
    b"
    467..114..
    ...*......
    ..35..633.
    ......#...
    617*......
    .....+.58.
    ..592.....
    ......755.
    ...$.*....
    .664.598..
    ",

    part1 => 4361,
    part2 => 467835
}

known_input_tests! {
    input: include_bytes!("../input/2023/day3.txt"),
    part1 => 556367,
    part2 => 89471771,
}
