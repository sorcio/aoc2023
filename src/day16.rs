use std::{collections::HashSet, vec};

use aoc_runner_derive::{aoc, aoc_generator};

use crate::{
    testing::{example_tests, known_input_tests},
    utils::{grid_cell_enum, AsciiUtils, FromGridLike},
};

grid_cell_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Cell {
        Empty => b'.',
        MirrorNwSe => b'\\',
        MirrorNeSw => b'/',
        SplitterNS => b'|',
        SplitterEW => b'-',
    }
}

impl Cell {
    fn passes_through(&self, direction: BeamDirection) -> bool {
        use BeamDirection::*;
        use Cell::*;
        matches!(
            (self, direction),
            (Empty, _) | (SplitterNS, North | South) | (SplitterEW, East | West)
        )
    }

    fn mirror_turn_beam(&self, beam: Beam) -> Beam {
        use BeamDirection::*;
        match self {
            Self::MirrorNwSe => {
                // '\'
                beam.with_direction(match beam.direction {
                    North => West,
                    South => East,
                    East => South,
                    West => North,
                })
            }
            Self::MirrorNeSw => {
                // '/'
                beam.with_direction(match beam.direction {
                    North => East,
                    South => West,
                    East => North,
                    West => South,
                })
            }
            _ => unreachable!(),
        }
    }

    fn splitter_split_beam(&self, beam: Beam) -> Option<(Beam, Beam)> {
        use BeamDirection::*;
        match self {
            Self::SplitterNS => match beam.direction {
                North | South => None,
                East | West => Some((beam.with_direction(North), beam.with_direction(South))),
            },
            Self::SplitterEW => match beam.direction {
                North | South => Some((beam.with_direction(East), beam.with_direction(West))),
                East | West => None,
            },
            _ => unreachable!(),
        }
    }

    fn is_mirror(&self) -> bool {
        matches!(self, Self::MirrorNwSe | Self::MirrorNeSw)
    }

    fn is_splitter(&self) -> bool {
        matches!(self, Self::SplitterNS | Self::SplitterEW)
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pos {
    x: u32,
    y: u32,
}

impl Pos {
    fn x(&self) -> usize {
        self.x as usize
    }

    fn y(&self) -> usize {
        self.y as usize
    }

    fn step(self, direction: BeamDirection) -> Option<Self> {
        let (x, y) = match direction {
            BeamDirection::North => (self.x, self.y.checked_sub(1)?),
            BeamDirection::South => (self.x, self.y + 1),
            BeamDirection::East => (self.x + 1, self.y),
            BeamDirection::West => (self.x.checked_sub(1)?, self.y),
        };
        Some(Self { x, y })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BeamDirection {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Beam {
    pos: Pos,
    direction: BeamDirection,
}

impl Beam {
    fn new(pos: Pos, direction: BeamDirection) -> Self {
        Self { pos, direction }
    }

    fn with_pos(self, pos: Pos) -> Self {
        Self {
            pos,
            direction: self.direction,
        }
    }

    fn with_direction(self, direction: BeamDirection) -> Self {
        Self {
            pos: self.pos,
            direction,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    fn contains(&self, pos: Pos) -> bool {
        pos.x() < self.width && pos.y() < self.height
    }

    fn get(&self, pos: Pos) -> Cell {
        self.cells[pos.y() * self.width + pos.x()]
    }

    fn beam_step(&self, beam: Beam) -> Option<Beam> {
        beam.pos
            .step(beam.direction)
            .and_then(|pos| self.contains(pos).then(|| beam.with_pos(pos)))
    }

    fn beam_ray(&self, beam: Beam) -> impl Iterator<Item = Beam> + '_ {
        std::iter::successors(Some(beam), |current| self.beam_step(*current))
    }

    fn follow_beams(&self, mut beams: Vec<Beam>, energized_grid: &mut EnergizedGrid) {
        let mut visited = HashSet::new();
        while let Some(beam) = beams.pop() {
            // println!("considering {:?}", beam);
            if !visited.insert(beam) {
                debug_assert!(energized_grid.get(beam.pos) == EnergizedState::Energized);
                continue;
            }
            let Some((beam, cell)) = self.beam_ray(beam).find_map(|successor| {
                // println!("            {successor:?}",);
                energized_grid.set_energized(successor.pos);
                let cell = self.get(successor.pos);
                if cell.passes_through(successor.direction) {
                    None
                } else {
                    Some((successor, cell))
                }
            }) else {
                continue;
            };
            // println!("         -> {beam:?} {cell:?}");
            if cell.is_mirror() {
                beams.extend(self.beam_step(cell.mirror_turn_beam(beam)));
            } else {
                debug_assert!(cell.is_splitter());
                if let Some((beam1, beam2)) = cell.splitter_split_beam(beam) {
                    beams.extend(self.beam_step(beam1));
                    beams.extend(self.beam_step(beam2));
                } else {
                    beams.extend(self.beam_step(beam));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnergizedState {
    NotEnergized = 0,
    Energized = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnergizedGrid {
    cells: Vec<EnergizedState>,
    width: usize,
    height: usize,
    energized_count: usize,
}

impl EnergizedGrid {
    fn new(grid: &Grid) -> Self {
        let cells = vec![EnergizedState::NotEnergized; grid.cells.len()];
        Self {
            cells,
            width: grid.width,
            height: grid.height,
            energized_count: 0,
        }
    }

    fn get(&self, pos: Pos) -> EnergizedState {
        self.cells[pos.y() * self.width + pos.x()]
    }

    fn get_mut(&mut self, pos: Pos) -> &mut EnergizedState {
        &mut self.cells[pos.y() * self.width + pos.x()]
    }

    fn set_energized(&mut self, pos: Pos) {
        let cell = self.get_mut(pos);
        if *cell == EnergizedState::NotEnergized {
            *cell = EnergizedState::Energized;
            self.energized_count += 1;
        }
    }
}

struct DisplayGrid<'a>(&'a Grid, &'a EnergizedGrid);

impl std::fmt::Display for DisplayGrid<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: fancier display of both energized grid and original grid
        use EnergizedState::*;
        for y in 0..self.1.height {
            for x in 0..self.1.width {
                let pos = Pos {
                    x: x as u32,
                    y: y as u32,
                };
                let cell = self.0.get(pos);
                let energized = self.1.get(pos);
                let c = match (cell, energized) {
                    (_, NotEnergized) => "⬛️",
                    (_, Energized) => "⬜️",
                };
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[aoc_generator(day16)]
fn parse(input: &[u8]) -> Grid {
    input.grid_like().unwrap().into_grid()
}

#[aoc(day16, part1)]
fn part1(input: &Grid) -> usize {
    let mut energized_grid = EnergizedGrid::new(input);
    let beam = Beam::new(Pos { x: 0, y: 0 }, BeamDirection::East);
    input.follow_beams(vec![beam], &mut energized_grid);
    // println!("{}", DisplayGrid(input, &energized_grid));
    energized_grid.energized_count
}

#[aoc(day16, part2)]
fn part2(input: &Grid) -> usize {
    (0..input.width)
        .map(|x| (x, 0, BeamDirection::South))
        .chain((0..input.width).map(|x| (x, input.height - 1, BeamDirection::North)))
        .chain((0..input.height).map(|y| (0, y, BeamDirection::East)))
        .chain((0..input.height).map(|y| (input.width - 1, y, BeamDirection::West)))
        .map(|(x, y, direction)| {
            let mut energized_grid = EnergizedGrid::new(input);
            let beam = Beam::new(
                Pos {
                    x: x as u32,
                    y: y as u32,
                },
                direction,
            );
            input.follow_beams(vec![beam], &mut energized_grid);
            energized_grid.energized_count
        })
        .max()
        .unwrap()
}

#[aoc(day16, part2, threaded)]
fn part2_threaded(input: &Grid) -> usize {
    // spawn an inordinate amount of threads because I decided to only use
    // stdlib and I don't want to implement a thread pool

    use std::thread;

    thread::scope(|s| {
        let threads: Vec<_> = (0..input.width)
            .map(|x| (x, 0, BeamDirection::South))
            .chain((0..input.width).map(|x| (x, input.height - 1, BeamDirection::North)))
            .chain((0..input.height).map(|y| (0, y, BeamDirection::East)))
            .chain((0..input.height).map(|y| (input.width - 1, y, BeamDirection::West)))
            .map(|(x, y, direction)| {
                let beam = Beam::new(
                    Pos {
                        x: x as u32,
                        y: y as u32,
                    },
                    direction,
                );
                s.spawn(move || {
                    let mut energized_grid = EnergizedGrid::new(input);
                    input.follow_beams(vec![beam], &mut energized_grid);
                    energized_grid.energized_count
                })
            })
            .collect();

        threads
            .into_iter()
            .map(|t| t.join().unwrap())
            .max()
            .unwrap()
    })
}

example_tests! {
    br"
    .|...\....
    |.-.\.....
    .....|-...
    ........|.
    ..........
    .........\
    ..../.\\..
    .-.-/..|..
    .|....-|.\
    ..//.|....
    ",
    part1 => 46,
}

known_input_tests! {
    input: include_bytes!("../input/2023/day16.txt"),
    part1 => 8098,
    part2 => 8335,
}
