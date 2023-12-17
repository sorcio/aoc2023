use std::cmp::Ordering;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::{
    testing::{example_tests, known_input_tests},
    utils::{AsciiUtils, FromGridLike, InvalidCharacter},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cell {
    cost: u8,
}

impl Cell {
    fn cost(&self) -> u32 {
        self.cost.into()
    }
}

impl TryFrom<u8> for Cell {
    type Error = InvalidCharacter;
    fn try_from(c: u8) -> Result<Self, InvalidCharacter> {
        match c {
            b'0'..=b'9' => Ok(Self { cost: c - b'0' }),
            c => Err(InvalidCharacter(c)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn all() -> impl Iterator<Item = Self> {
        use Direction::*;
        [Up, Down, Left, Right].iter().copied()
    }

    fn opposite(self) -> Self {
        use Direction::*;
        match self {
            Up => Down,
            Down => Up,
            Left => Right,
            Right => Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pos {
    x: u32,
    y: u32,
}

impl Pos {
    fn new(x: usize, y: usize) -> Self {
        Self {
            x: x.try_into().unwrap(),
            y: y.try_into().unwrap(),
        }
    }

    fn x(&self) -> usize {
        self.x as _
    }

    fn y(&self) -> usize {
        self.y as _
    }

    fn manhattan_distance(&self, other: Self) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as _
    }

    fn step(self, dir: Direction) -> Option<Self> {
        use Direction::*;
        Some(match dir {
            Up => Self {
                x: self.x,
                y: self.y.checked_sub(1)?,
            },
            Down => Self {
                x: self.x,
                y: self.y + 1,
            },
            Left => Self {
                x: self.x.checked_sub(1)?,
                y: self.y,
            },
            Right => Self {
                x: self.x + 1,
                y: self.y,
            },
        })
    }
}

#[derive(Debug, Clone)]
struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl FromGridLike for Grid {
    type Cell = Cell;
    fn from_cells(cells: Vec<Self::Cell>, width: usize, height: usize) -> Self {
        Self {
            cells,
            width,
            height,
        }
    }
}

impl Grid {
    fn contains(&self, pos: Pos) -> bool {
        pos.x() < self.width && pos.y() < self.height
    }

    fn get(&self, pos: Pos) -> Cell {
        self.cells[pos.y() * self.width + pos.x()]
    }

    fn neighbors(&self, pos: Pos) -> impl Iterator<Item = (Direction, Pos, Cell)> + '_ {
        Direction::all().filter_map(move |dir| {
            let new_pos = pos.step(dir)?;
            if self.contains(new_pos) {
                let cell = self.get(new_pos);
                Some((dir, new_pos, cell))
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchNode {
    pos: Pos,
    cost: u32,
    heuristic: u32,
    direction: Direction,
    steps_in_direction: u32,
}

impl SearchNode {
    fn new(pos: Pos, cost: u32, heuristic: u32, direction: Direction) -> Self {
        Self {
            pos,
            cost,
            heuristic,
            direction,
            steps_in_direction: 0,
        }
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.cost + self.heuristic)
            .cmp(&(other.cost + other.heuristic))
            .reverse()
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn find_path(
    grid: &Grid,
    start: Pos,
    end: Pos,
    min_steps_in_direction: u32,
    max_steps_in_direction: u32,
) -> Option<u32> {
    use std::collections::BinaryHeap;

    let mut queue = BinaryHeap::new();
    let mut visited = std::collections::HashSet::new();

    let eval_heuristic = |pos: Pos| pos.manhattan_distance(end);

    // initialize queue with neighbors of start position so that we always have
    // a valid direction in search nodes
    for (dir, pos, cell) in grid.neighbors(start) {
        let mut node = SearchNode::new(pos, cell.cost(), eval_heuristic(pos), dir);
        node.steps_in_direction = 1;
        queue.push(node);
    }

    while let Some(node) = queue.pop() {
        if node.pos == end {
            return Some(node.cost);
        }

        if !visited.insert((node.pos, node.direction, node.steps_in_direction)) {
            continue;
        }

        for (direction, pos, cell) in grid.neighbors(node.pos) {
            let steps_in_direction = if node.direction == direction.opposite() {
                continue;
            } else if node.direction == direction {
                if node.steps_in_direction >= max_steps_in_direction {
                    continue;
                }
                node.steps_in_direction + 1
            } else if node.steps_in_direction >= min_steps_in_direction {
                // reset steps in direction when changing direction
                1
            } else {
                continue;
            };
            let successor = SearchNode {
                pos,
                direction,
                steps_in_direction,
                cost: node.cost + cell.cost(),
                heuristic: eval_heuristic(pos),
            };
            queue.push(successor);
        }
    }
    None
}

#[aoc_generator(day17)]
fn parse(input: &[u8]) -> Grid {
    input.grid_like().unwrap().into_grid()
}

#[aoc(day17, part1)]
fn part1(grid: &Grid) -> u32 {
    let start = Pos::new(0, 0);
    let end = Pos::new(grid.width - 1, grid.height - 1);
    const MIN_STEPS_IN_DIRECTION: u32 = 1;
    const MAX_STEPS_IN_DIRECTION: u32 = 3;

    find_path(
        grid,
        start,
        end,
        MIN_STEPS_IN_DIRECTION,
        MAX_STEPS_IN_DIRECTION,
    )
    .unwrap()
}

#[aoc(day17, part2)]
fn part2(grid: &Grid) -> u32 {
    let start = Pos::new(0, 0);
    let end = Pos::new(grid.width - 1, grid.height - 1);
    const MIN_STEPS_IN_DIRECTION: u32 = 4;
    const MAX_STEPS_IN_DIRECTION: u32 = 10;

    find_path(
        grid,
        start,
        end,
        MIN_STEPS_IN_DIRECTION,
        MAX_STEPS_IN_DIRECTION,
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_cost() {
        let input = b"1111\n".repeat(4);
        let grid = parse(&input);
        assert_eq!(part1(&grid), 6);
    }

    #[test]
    fn forced_turn() {
        let input = b"911111\n119991".repeat(4);
        let grid = parse(&input);
        let start = Pos::new(0, 0);
        let end = Pos::new(grid.width - 1, grid.height - 1);

        let result = find_path(&grid, start, end, 1, 3);
        assert_eq!(result, Some(17));
    }
}

example_tests! {
    b"
    2413432311323
    3215453535623
    3255245654254
    3446585845452
    4546657867536
    1438598798454
    4457876987766
    3637877979653
    4654967986887
    4564679986453
    1224686865563
    2546548887735
    4322674655533
    ",

    part1 => 102,
    part2 => 94,
}

known_input_tests! {
    input: include_bytes!("../input/2023/day17.txt"),
    part1 => 668,
    part2 => 788,
}
