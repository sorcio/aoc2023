use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::example_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    // convention: directions are named clockwise starting up
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn directions() -> [Self; 4] {
        [Self::Up, Self::Right, Self::Down, Self::Left]
    }

    fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GridPos {
    x: usize,
    y: usize,
}

impl GridPos {
    fn apply(self, dir: Direction) -> Option<Self> {
        let GridPos { x, y } = self;
        Some(match dir {
            Direction::Up => GridPos {
                x,
                y: y.checked_sub(1)?,
            },
            Direction::Down => GridPos { x, y: y + 1 },
            Direction::Left => GridPos {
                x: x.checked_sub(1)?,
                y,
            },
            Direction::Right => GridPos { x: x + 1, y },
        })
    }
}

#[derive(Debug)]
enum GridCell {
    // Just as a naming convention, directions are named clockwise starting up
    UpRight,
    UpDown,
    UpLeft,
    RightDown,
    RightLeft,
    DownLeft,
    Start,
    Empty,
}

impl From<char> for GridCell {
    fn from(c: char) -> Self {
        use GridCell::*;
        match c {
            'L' => UpRight,
            '|' => UpDown,
            'J' => UpLeft,
            'F' => RightDown,
            '-' => RightLeft,
            '7' => DownLeft,
            'S' => Start,
            '.' => Empty,
            _ => panic!("Invalid grid cell: {}", c),
        }
    }
}

impl GridCell {
    fn exits(&self) -> &'static [Direction] {
        use Direction::*;
        use GridCell::*;
        match self {
            UpRight => &[Up, Right],
            UpDown => &[Up, Down],
            UpLeft => &[Up, Left],
            RightDown => &[Right, Down],
            RightLeft => &[Right, Left],
            DownLeft => &[Down, Left],
            Start => &[],
            Empty => &[],
        }
    }
}

struct Grid {
    grid: Vec<GridCell>,
    width: usize,
    height: usize,
    start_pos: GridPos,
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Grid")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("start_pos", &self.start_pos)
            .finish()
    }
}

impl FromIterator<char> for Grid {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut start_pos = None;
        let mut x = 0;
        let mut y = 0;
        let grid: Vec<_> = iter
            .into_iter()
            .filter_map(|c| {
                if c == '\n' {
                    x = 0;
                    y += 1;
                    None
                } else {
                    if c == 'S' {
                        assert!(start_pos.is_none());
                        start_pos = Some(GridPos { x, y });
                    }
                    x += 1;
                    Some(c.into())
                }
            })
            .collect();
        let height = if x == 0 { y } else { y + 1 };
        let width = grid.len() / height;
        // dbg!(x, y, width, height, grid.len());
        assert_eq!(grid.len() % height, 0);
        assert_eq!(grid.len(), width * height);
        Self {
            grid,
            width,
            height,
            start_pos: start_pos.unwrap(),
        }
    }
}

impl Grid {
    fn walk_from_start(&self) -> (Walker, Walker) {
        // find the two starting positions adjacent to start_pos
        let mut walker1 = None;
        let mut walker2 = None;
        for (dir, pos) in Direction::directions()
            .into_iter()
            .filter_map(|dir| Some((dir, self.adjacent(self.start_pos, dir)?)))
        {
            for &exit in self.cell(pos).exits() {
                if exit == dir.opposite() {
                // if self.adjacent(pos, exit) == Some(self.start_pos) {
                    let walker = Walker {
                        grid: self,
                        pos,
                        come_from: exit,
                    };
                    if walker1.is_none() {
                        walker1 = Some(walker);
                    } else if walker2.is_none() {
                        walker2 = Some(walker);
                    } else {
                        panic!("More than two start positions found");
                    }
                }
            }
        }
        (walker1.unwrap(), walker2.unwrap())
    }

    fn cell(&self, pos: GridPos) -> &GridCell {
        debug_assert!(self.contains(pos), "{pos:?} out of bounds");
        &self.grid[pos.y * self.width + pos.x]
    }

    fn adjacent(&self, pos: GridPos, dir: Direction) -> Option<GridPos> {
        let pos = pos.apply(dir)?;
        self.contains(pos).then_some(pos)
    }

    fn contains(&self, pos: GridPos) -> bool {
        pos.x < self.width && pos.y < self.height
    }
}

#[derive(Debug)]
struct Walker<'g> {
    grid: &'g Grid,
    pos: GridPos,
    come_from: Direction,
}

impl<'g> Walker<'g> {
    fn step(&mut self) {
        let dir = self
            .grid
            .cell(self.pos)
            .exits()
            .iter()
            .find(|&&dir| dir != self.come_from)
            .unwrap();
        self.pos = self.pos.apply(*dir).unwrap();
        self.come_from = dir.opposite();
    }
}

#[aoc_generator(day10)]
fn parse(input: &str) -> Grid {
    input.chars().collect()
}

#[aoc(day10, part1)]
fn part1(grid: &Grid) -> usize {
    let (mut walker1, mut walker2) = grid.walk_from_start();
    let mut steps = 1;
    while walker1.pos != walker2.pos {
        walker1.step();
        walker2.step();
        steps += 1;
    }
    steps
}

#[aoc(day10, part2)]
fn part2(input: &Grid) -> usize {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_trailing_newline() {
        let grid = parse("....\n.S..\n....\n");
        assert_eq!(grid.width, 4);
        assert_eq!(grid.height, 3);
        assert_eq!(grid.start_pos, GridPos { x: 1, y: 1 });
    }

    #[test]
    fn parse_without_trailing_newline() {
        let grid = parse("....\n.S..\n....");
        assert_eq!(grid.width, 4);
        assert_eq!(grid.height, 3);
        assert_eq!(grid.start_pos, GridPos { x: 1, y: 1 });
    }

    #[test]
    fn minimal_loop() {
        let grid = parse(&unindent::unindent(
            "
            .S-7
            .|.|
            .L-J
            ",
        ));
        let (mut walker1, mut walker2) = grid.walk_from_start();
        // let's benefit from our naming convention to deterministically know which walker is which
        assert_eq!(walker1.pos, GridPos { x: 2, y: 0 });
        assert_eq!(walker2.pos, GridPos { x: 1, y: 1 });
        walker1.step();
        walker2.step();
        assert_eq!(walker1.pos, GridPos { x: 3, y: 0 });
        assert_eq!(walker2.pos, GridPos { x: 1, y: 2 });
        walker1.step();
        walker2.step();
        assert_eq!(walker1.pos, GridPos { x: 3, y: 1 });
        assert_eq!(walker2.pos, GridPos { x: 2, y: 2 });
        walker1.step();
        walker2.step();
        assert_eq!(walker1.pos, walker2.pos);
        assert_eq!(walker1.pos, GridPos { x: 3, y: 2 });
        assert_eq!(walker2.pos, GridPos { x: 3, y: 2 });
        assert_eq!(part1(&grid), 4);
    }
}

example_tests! {
    "
    ..F7.
    .FJ|.
    SJ.L7
    |F--J
    LJ...
    ",
    part1 => 8,
}
