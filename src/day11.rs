use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::{example_tests, known_input_tests};
use crate::utils::SliceUtils;

struct UnparsedGrid {
    grid: Box<[u8]>,
    width: usize,
    height: usize,
}

fn trim_input_eol(input: &[u8]) -> &[u8] {
    if input.ends_with(b"\n") {
        &input[..input.len() - 1]
    } else {
        input
    }
}

impl UnparsedGrid {
    fn new(grid: &[u8]) -> Self {
        // I don't know what to expect in part 2 but I suspect it will change
        // the way we do the expansion. So, just in doubt, I'm copying the input
        // string and doing the expansion in the solver function
        let grid = trim_input_eol(grid).to_vec().into_boxed_slice();
        let width = grid
            .split(|&c| c == b'\n')
            .next()
            .expect("should have at least one line")
            .len();
        let height = grid.iter().filter(|&&c| c == b'\n').count() + 1;
        Self {
            grid,
            width,
            height,
        }
    }

    fn get(&self, row: usize, col: usize) -> u8 {
        self.grid[row * (self.width + 1) + col]
    }

    /// Get the unexpanded positions
    fn unexpanded_positions(&self) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        for row in 0..self.height {
            for col in 0..self.width {
                if self.get(row, col) == b'#' {
                    positions.push((row, col));
                }
            }
        }
        positions
    }

    fn expand(&self, expansion_factor: usize) -> Vec<Position> {
        let mut row_to_y = vec![0; self.height];
        let mut y = 0;
        for (row, value) in row_to_y.iter_mut().enumerate() {
            let is_empty = (0..self.width).all(|col| self.get(row, col) == b'.');
            *value = y;
            y += if is_empty { expansion_factor } else { 1 };
        }

        let mut col_to_x = vec![0; self.width];
        let mut x = 0;
        for (col, value) in col_to_x.iter_mut().enumerate() {
            let is_empty = (0..self.height).all(|row| self.get(row, col) == b'.');
            *value = x;
            x += if is_empty { expansion_factor } else { 1 };
        }

        self.unexpanded_positions()
            .into_iter()
            .map(|(row, col)| position(col_to_x[col], row_to_y[row]))
            .collect()
    }
}

#[cfg(feature = "extra-debug-prints")]
fn print_locations(positions: &[Position]) {
    let width = positions.iter().map(|p| p.x).max().unwrap();
    let height = positions.iter().map(|p| p.y).max().unwrap();
    let mut grid = vec![b'.'; (width + 2) * (height + 1)];
    for line in 0..=height {
        grid[line * (width + 2) + width + 1] = b'\n';
    }
    for position in positions {
        grid[position.y * (width + 1) + position.x] = b'#';
    }
    println!("{}", String::from_utf8_lossy(&grid));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn manhattan_distance(&self, other: &Self) -> usize {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}
fn position(x: usize, y: usize) -> Position {
    Position { x, y }
}

#[aoc_generator(day11)]
fn parse(input: &[u8]) -> UnparsedGrid {
    let grid = UnparsedGrid::new(input);
    assert_eq!(grid.width, grid.height, "input should be square");
    grid
}

#[aoc(day11, part1)]
fn part1(input: &UnparsedGrid) -> usize {
    let positions = input.expand(2);
    #[cfg(feature = "extra-debug-prints")]
    print_locations(&positions);
    positions
        .pairs()
        .map(|(p1, p2)| p1.manhattan_distance(p2))
        .sum()
}

#[aoc(day11, part2)]
fn part2(input: &UnparsedGrid) -> usize {
    let positions = input.expand(1000000);
    #[cfg(feature = "extra-debug-prints")]
    print_locations(&positions);
    positions
        .pairs()
        .map(|(p1, p2)| p1.manhattan_distance(p2))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand() {
        let input = unindent::unindent_bytes(
            b"
            ...#......
            .......#..
            #.........
            ..........
            ......#...
            .#........
            .........#
            ..........
            .......#..
            #...#.....
            ",
        );
        let grid = UnparsedGrid::new(&input);
        let mut positions = grid.expand(2);
        assert_eq!(positions.len(), 9);
        positions.sort_by_key(|p| (p.y, p.x));
        assert_eq!(positions[0], position(4, 0));
        assert_eq!(positions[1], position(9, 1));
        assert_eq!(positions[2], position(0, 2));
        // compare with the distances given in the example
        let distance = |a: usize, b: usize| positions[a - 1].manhattan_distance(&positions[b - 1]);
        assert_eq!(distance(5, 9), 9);
        assert_eq!(distance(1, 7), 15);
        assert_eq!(distance(3, 6), 17);
        assert_eq!(distance(8, 9), 5);
    }
}

example_tests! {
    b"
    ...#......
    .......#..
    #.........
    ..........
    ......#...
    .#........
    .........#
    ..........
    .......#..
    #...#.....
    ",

    part1 => 374,

    // note: the problem description only includes example of expansion by
    // factor of 10 or 100, but this macro is too limited to include those; so I
    // computed the answer for factor 1000000 and pasted it here
    part2 => 82000210,
}

known_input_tests! {
    input: include_bytes!("../input/2023/day11.txt"),
    part1 => 9274989,
    part2 => 357134560737,
}
