use std::collections::HashSet;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::{example_tests, known_input_tests};

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

    fn clockwise(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GridPos {
    x: usize,
    y: usize,
}

impl std::fmt::Debug for GridPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Debug formatting is 1-based lines/cols so that I can use it to debug
        // the examples in a code editor without having to mentally convert
        f.debug_struct("GridPos")
            .field("Ln", &(self.y + 1))
            .field("Col", &(self.x + 1))
            .finish()
    }
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

#[derive(Debug, Clone)]
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

fn print_loop_grid(grid: &Grid, loop_positions: &[GridPos]) {
    let max_x = loop_positions.iter().map(|pos| pos.x).max().unwrap();
    let max_y = loop_positions.iter().map(|pos| pos.y).max().unwrap();
    println!(
        "{}",
        std::iter::repeat("-").take(grid.width).collect::<String>()
    );
    for y in 0..=max_y {
        for x in 0..=max_x {
            let pos = GridPos { x, y };
            if loop_positions.contains(&pos) {
                match grid.cell(pos) {
                    GridCell::UpRight => print!("L"),
                    GridCell::UpDown => print!("|"),
                    GridCell::UpLeft => print!("J"),
                    GridCell::RightDown => print!("F"),
                    GridCell::RightLeft => print!("-"),
                    GridCell::DownLeft => print!("7"),
                    GridCell::Start => print!("S"),
                    GridCell::Empty => print!(" "),
                }
            } else {
                print!(" ");
            }
        }
        println!();
    }
}

#[aoc(day10, part2)]
fn part2_turns(grid: &Grid) -> usize {
    let (walker1, walker2) = grid.walk_from_start();

    // first, let's collect all the positions of the loop
    let mut collect_walker = walker1.clone();
    let mut loop_positions = vec![grid.start_pos, collect_walker.pos];
    let mut cw_turn_count = 0isize;
    while collect_walker.pos != grid.start_pos {
        let dir1 = collect_walker.come_from.opposite();
        collect_walker.step();
        let dir2 = collect_walker.come_from.opposite();
        if dir1.clockwise() == dir2 {
            cw_turn_count += 1;
        } else if dir2.clockwise() == dir1 {
            cw_turn_count -= 1;
        }
        loop_positions.push(collect_walker.pos);
    }

    dbg!(cw_turn_count);
    print_loop_grid(grid, &loop_positions);

    // let's decide which walker is the clockwise walker
    let mut walker = if cw_turn_count >= 0 { walker1 } else { walker2 };
    println!("clockwise is {start_pos:?} -> {second_pos:?}", start_pos = grid.start_pos, second_pos = walker.pos);

    // now, let's walk the loop clockwise and collect the positions that are on the inside
    let mut queue = Vec::new();
    while walker.pos != grid.start_pos {
        let inside_dir = walker.come_from.opposite().clockwise();
        if let Some(inside) = grid.adjacent(walker.pos, inside_dir) {
            if !loop_positions.contains(&inside) {
                println!(
                    "{inside:?} is inside ({pos:?} -> {inside_dir:?})",
                    pos = walker.pos
                );
                queue.push(inside);
            }
        } else {
            dbg!(walker.pos, inside_dir);
            panic!("inside direction should be inside map")
        }

        // match grid.cell(walker.pos) {
        //     GridCell::UpDown | GridCell::RightLeft => {
        //         let inside_dir = walker.come_from.opposite().clockwise();
        //         if let Some(inside) = grid.adjacent(walker.pos, inside_dir) {
        //             queue.push(inside);
        //         } else {
        //             // is this a bug?
        //             dbg!(walker.pos, inside_dir);
        //             panic!("inside direction should be inside map")
        //         }
        //     }
        //     _ => {}
        // }
        walker.step();
    }

    let mut visited: HashSet<_> = loop_positions.iter().copied().collect();
    let mut inside_count = 0;
    while let Some(pos) = queue.pop() {
        if visited.contains(&pos) {
            continue;
        }
        visited.insert(pos);
        inside_count += 1;
        for &dir in Direction::directions().iter() {
            if let Some(adj) = grid.adjacent(pos, dir) {
                if !visited.contains(&adj) {
                    queue.push(adj);
                }
            }
        }
    }
    inside_count
}

// #[aoc(day10, part2, flood_fill)]
fn part2_flood_fill(grid: &Grid) -> usize {
    let (walker1, walker2) = grid.walk_from_start();

    let loop_start_exits = [walker1.come_from.opposite(), walker2.come_from.opposite()];

    // first, let's collect all the positions of the loop
    let mut collect_walker = walker1.clone();
    let mut loop_positions = vec![grid.start_pos, collect_walker.pos];
    // let mut cw_turn_count = 0isize;
    while collect_walker.pos != grid.start_pos {
        // let dir1 = collect_walker.come_from.opposite();
        collect_walker.step();
        // let dir2 = collect_walker.come_from.opposite();
        // if dir1.clockwise() == dir2 {
        //     cw_turn_count += 1;
        // } else if dir2.clockwise() == dir1 {
        //     cw_turn_count -= 1;
        // }
        loop_positions.push(collect_walker.pos);
    }

    // find a position on the border of the grid which is definitely outside the loop
    // (this will break if the loop encloses the whole grid but let's forget about that case)
    let boundary: HashSet<_> = loop_positions.iter().copied().collect();
    // let flood_start = (0..grid.width)
    //     .map(|x| GridPos { x, y: 0 })
    //     .filter(|&pos| !boundary.contains(&pos))
    //     .unwrap();

    let mut stack = vec![];
    for &x in &[0, grid.width - 1] {
        for &y in &[0, grid.height - 1] {
            let pos = GridPos { x, y };
            if !boundary.contains(&pos) {
                stack.push((0, false, pos));
            }
        }
    }

    // let mut stack = vec![(0, flood_start)];
    let mut visited = HashSet::new();
    // visited.insert(grid.start_pos);
    let mut count_inside = 0usize;
    while let Some((number, on_boundary, pos)) = stack.pop() {
        if visited.contains(&pos) {
            continue;
        }
        visited.insert(pos);
        debug_assert_eq!(on_boundary, boundary.contains(&pos));
        if (number & 1) == 1 && !on_boundary {
            println!("{pos:?} is inside ({number})");
            count_inside += 1;
        }

        let exits = if pos == grid.start_pos {
            &loop_start_exits
        } else {
            grid.cell(pos).exits()
        };
        for &dir in Direction::directions()
            .iter()
            .filter(|&&dir| !exits.contains(&dir))
        {
            if let Some(adj) = grid.adjacent(pos, dir) {
                let new_pos_on_boundary = boundary.contains(&adj);
                if on_boundary {
                    stack.push((number + 1, new_pos_on_boundary, adj));
                } else {
                    stack.push((number, new_pos_on_boundary, adj));
                }
            }
        }

        // for dx in &[-1, 0, 1] {
        //     if pos.x == 0 && *dx == -1 {
        //         continue;
        //     }
        //     for dy in &[-1, 0, 1] {
        //         if pos.y == 0 && *dy == -1 {
        //             continue;
        //         }
        //         if dx == &0 && dy == &0 {
        //             continue;
        //         }
        //         let adj = GridPos {
        //             x: (pos.x as isize + dx) as usize,
        //             y: (pos.y as isize + dy) as usize,
        //         };
        //         if grid.contains(adj) {
        //             if on_boundary {
        //                 stack.push((number + 1, adj));
        //             } else {
        //                 stack.push((number, adj));
        //             }
        //         }
        //     }
        // }
        stack.sort_by_key(|(n, _, _)| -*n);
    }
    count_inside
}

#[allow(dead_code)]
fn part2(grid: &Grid) -> usize {
    part2_turns(grid)
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
    fn minimal_loop_walk() {
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

    #[test]
    fn minimal_loop_fill() {
        let grid = parse(&unindent::unindent(
            "
            .S7
            .LJ
            ",
        ));
        assert_eq!(part2(&grid), 0);

        let grid = parse(&unindent::unindent(
            "
            .S-7
            .|.|
            .L-J
            ",
        ));
        assert_eq!(part2(&grid), 1);

        let grid = parse(&unindent::unindent(
            "
            .S--7
            .|..|
            .|..|
            .L--J
            ",
        ));
        assert_eq!(part2(&grid), 4);
    }

    #[test]
    fn minimal_loop_2_fill() {
        // same as minimal but expose the bug where we mistake clockwise for counterclockwise
        let grid = parse(&unindent::unindent(
            "
            .F7
            .LS
            ",
        ));
        assert_eq!(part2(&grid), 0);

        let grid = parse(&unindent::unindent(
            "
            .F-7
            .|.|
            .LSJ
            ",
        ));
        assert_eq!(part2(&grid), 1);

        let grid = parse(&unindent::unindent(
            "
            .F--7
            .|..|
            .|..|
            .LS-J
            ",
        ));
        assert_eq!(part2(&grid), 4);
    }

    #[test]
    fn tricky_loop_fill() {
        let grid = parse(&unindent::unindent(
            "
            .S-----7.
            .|..F-7|.
            .L--J.||.
            .....FJ|.
            .....L-J.
            ",
        ));
        assert_eq!(part2(&grid), 2);
    }

    #[test]
    fn tricky_loop2_fill() {
        let grid = parse(&unindent::unindent(
            "
            ...S-7
            .F-J.|
            .|..FJ
            .L--J.
            ",
        ));
        assert_eq!(part2(&grid), 3);
    }

    #[test]
    fn tricky_loop3_fill() {
        // spiral pattern with only one cell inside the loop
        let grid = parse(&unindent::unindent(
            "
            .................
            ...S7F--------7..
            ...|||F------7|..
            ...|||L--7...||..
            ...||L-7.|...||..
            ...||..L-J...||..
            ...|L--------J|..
            ...L----------J..
            .................
            ",
        ));
        assert_eq!(part2(&grid), 1);
    }

    #[test]
    fn tricky_loop4_fill() {
        // floral pattern with only two cells inside
        let grid = parse(&unindent::unindent(
            "
            .........
            .F--7F-7.
            .L7.SJFJ.
            .FJ.F7L7.
            .L--JL-J.
            .........
            ",
        ));
        assert_eq!(part2(&grid), 2);
    }

    #[test]
    fn example_loop_fill() {
        let grid = parse(&unindent::unindent(
            "
            .S-------7.
            .|F-----7|.
            .||.....||.
            .||.....||.
            .|L-7.F-J|.
            .|..|.|..|.
            .L--J.L--J.
            ...........
            ",
        ));
        assert_eq!(part2(&grid), 4);
    }

    #[test]
    fn example_loop2_fill() {
        let grid = parse(&unindent::unindent(
            "
            .S------7.
            .|F----7|.
            .||....||.
            .||....||.
            .|L-7F-J|.
            .|--||--|.
            .L--JL--J.
            ..........
            ",
        ));
        assert_eq!(part2(&grid), 4);


        let grid = parse(&unindent::unindent(
            "
            .S-------7.
            .|F-----7|.
            .||.F7F7||.
            .||.||||||.
            .|L-J||LJ|.
            .|...||--|.
            .L---JL--J.
            ...........
            ",
        ));
        assert_eq!(part2(&grid), 5);
    }

    #[test]
    fn example_loop3_fill() {
        let grid = parse(&unindent::unindent(
            "
            .F----7F7F7F7F-7....
            .|F--7||||||||FJ....
            .||.FJ||||||||L7....
            FJL7L7LJLJ||LJ.L-7..
            L--J.L7...LJS7F-7L7.
            ....F-J..F7FJ|L7L7L7
            ....L7.F7||L7|.L7L7|
            .....|FJLJ|FJ|F7|.LJ
            ....FJL-7.||.||||...
            ....L---J.LJ.LJLJ...
            ",
        ));
        assert_eq!(part2(&grid), 8);
    }
}

example_tests! {
    parser: super::parse,

    "
    ..F7.
    .FJ|.
    SJ.L7
    |F--J
    LJ...
    ",
    part1 => 8,

    "
    FF7FSF7F7F7F7F7F---7
    L|LJ||||||||||||F--J
    FL-7LJLJ||||||LJL-77
    F--JF--7||LJLJ7F7FJ-
    L---JF-JLJ.||-FJLJJ7
    |F|F-JF---7F7-L7L|7|
    |FFJF7L7F-JF7|JL---7
    7-L-JL7||F7|L7F-7F7|
    L.L7LFJ|||||FJL7||LJ
    L7JLJL-JLJLJL--JLJ.L
    ",
    part2 => 10,
}

known_input_tests! {
    input: include_str!("../input/2023/day10.txt"),
    part1 => 6820,
}
