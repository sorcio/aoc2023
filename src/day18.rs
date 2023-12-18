use std::str::FromStr;

use aoc_runner_derive::aoc;

use crate::testing::{example_tests, known_input_tests};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn as_unit_step(self) -> (isize, isize) {
        match self {
            Direction::Up => (-1, 0),
            Direction::Down => (1, 0),
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
        }
    }

    fn clockwise(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl TryFrom<char> for Direction {
    type Error = char;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'U' => Ok(Self::Up),
            'D' => Ok(Self::Down),
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            c => Err(c),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = match s.chars().next() {
            Some('#') => &s[1..],
            Some('0'..='9' | 'a'..='f' | 'A'..='F') => s,
            _ => return Err(()),
        };
        if s.len() != 6 {
            return Err(());
        }
        let r = u8::from_str_radix(&s[0..2], 16).unwrap();
        let g = u8::from_str_radix(&s[2..4], 16).unwrap();
        let b = u8::from_str_radix(&s[4..6], 16).unwrap();
        Ok(Color { r, g, b })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Step {
    direction: Direction,
    distance: isize,

    /// Color of the segment. This is never used in the solution but it's cute
    /// in the SVG rendering.
    color: Color,
}

impl Step {
    /// Parsing as defined in part 1 (ignoring color if not present).
    fn parse_regular(s: &str) -> Self {
        let mut parts = s.split_whitespace();
        let direction = parts
            .next()
            .unwrap()
            .chars()
            .next()
            .unwrap()
            .try_into()
            .unwrap();
        let distance = parts.next().unwrap().parse().unwrap();
        let color = parts
            .next()
            .map(|s| s.parse().unwrap_or_default())
            .unwrap_or_default();
        Step {
            direction,
            distance,
            color,
        }
    }

    /// Parsing as defined in part 2. Color is always 0, 0, 0.
    fn parse_alternate(s: &str) -> Self {
        let s = s
            .split_whitespace()
            .last()
            .unwrap()
            .trim_matches(|c| c == '(' || c == ')' || c == '#');
        debug_assert!(s.len() == 6);
        let distance = u32::from_str_radix(&s[0..5], 16).unwrap();
        let direction_code = s.chars().last().unwrap();
        // 0 means R, 1 means D, 2 means L, and 3 means U.
        let direction = match direction_code {
            '0' => Direction::Right,
            '1' => Direction::Down,
            '2' => Direction::Left,
            '3' => Direction::Up,
            _ => panic!("invalid direction code"),
        };
        let color = Color::default();
        Self {
            direction,
            distance: distance.try_into().unwrap(),
            color,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn step(self, direction: Direction, distance: isize) -> Self {
        let (dx, dy) = direction.as_unit_step();
        Self {
            x: self.x + dx * distance,
            y: self.y + dy * distance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SegmentLoop {
    steps: Vec<Step>,
}

impl SegmentLoop {
    fn new(steps: Vec<Step>) -> Self {
        Self { steps }
    }

    /// Iterate through all the positions in the loop, starting at `start`.
    /// Since the loop is closed, the final position is the same as start.
    fn positions(&self, start: Position) -> impl Iterator<Item = Position> + '_ {
        std::iter::once(start).chain(self.steps.iter().scan(start, |pos, step| {
            *pos = pos.step(step.direction, step.distance);
            Some(*pos)
        }))
    }
}

#[cfg(feature = "draw-visuals")]
fn draw_loop_as_svg_path(segments: &SegmentLoop, file_name: &str) {
    let start = Position::default();
    let positions: Vec<_> = segments.positions(start).collect();
    dbg!(positions.len(), segments.steps.len());
    let mut path = "M 0,0".to_string();
    for pos in &positions[1..] {
        path.push_str(&format!(" L {},{}", pos.x, pos.y));
    }
    path.push_str(" Z");

    let max_x = positions.iter().map(|p| p.x).max().unwrap();
    let min_x = positions.iter().map(|p| p.x).min().unwrap();
    let max_y = positions.iter().map(|p| p.y).max().unwrap();
    let min_y = positions.iter().map(|p| p.y).min().unwrap();
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"{} {} {} {}\" xmlns=\"http://www.w3.org/2000/svg\" style=\"background: #000000\">",
        min_x - 4, min_y - 4, width + 8, height + 8
    ));
    svg.push_str(&format!(
        "<path d=\"{}\" fill=\"white\" stroke=\"transparent\"/>",
        path
    ));

    svg.push_str(
        r#"<circle cx="0" cy="0" r="2.0" stroke="rgba(255, 0, 0, 127)" stroke-width="0.5" fill="transparent" />"#,
    );

    for (segment, step) in positions.windows(2).zip(&segments.steps) {
        let (p1, p2) = (segment[0], segment[1]);
        let color = step.color;
        svg.push_str(&format!(
            r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="rgb({r},{g},{b})" stroke-width="1.0"
            style="filter: hue-rotate(10deg) saturate(2) drop-shadow(0.5px 0.5px 0.2px rgb({r},{g},{b}))" />"#,
            x1 = p1.x, y1 = p1.y, x2 = p2.x, y2 = p2.y, r = color.r, g = color.g, b = color.b
        ));
    }

    svg.push_str("</svg>");

    use std::path::*;
    let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(file_name);
    std::fs::write(file_path, svg).unwrap();
}

fn parse_regular(input: &str) -> SegmentLoop {
    SegmentLoop::new(input.lines().map(Step::parse_regular).collect())
}

fn parse_alternate(input: &str) -> SegmentLoop {
    SegmentLoop::new(input.lines().map(Step::parse_alternate).collect())
}

/// Compute area of a polygon given its vertices.
fn shoelace_formula(vertices: &[Position]) -> isize {
    let mut area = 0;
    for pp in vertices.windows(2) {
        let p1 = pp[0];
        let p2 = pp[1];
        area += -p1.y * p2.x + p1.x * p2.y;
    }
    let last = vertices.last().unwrap();
    area += -last.y * vertices[0].x + last.x * vertices[0].y;
    area.abs() / 2
}

fn exterior_area_of_loop(segment_loop: &SegmentLoop) -> isize {
    // We figure out which side is inside or outside using the same method as in
    // day 10, by counting the turns
    let mut cw_turn_count = 0isize;
    let mut current_direction = segment_loop.steps[0].direction;
    for step in &segment_loop.steps[1..] {
        let turn = match (current_direction, step.direction) {
            (Direction::Up, Direction::Right) => 1,
            (Direction::Right, Direction::Down) => 1,
            (Direction::Down, Direction::Left) => 1,
            (Direction::Left, Direction::Up) => 1,
            (Direction::Up, Direction::Left) => -1,
            (Direction::Left, Direction::Down) => -1,
            (Direction::Down, Direction::Right) => -1,
            (Direction::Right, Direction::Up) => -1,
            (a, b) if a == b => 0,
            _ => panic!(
                "only 90 degree turns expected {:?} -> {:?}",
                current_direction, step.direction
            ),
        };
        cw_turn_count += turn;
        current_direction = step.direction;
    }

    let outside = if cw_turn_count > 0 {
        // clockwise
        |dir: Direction| dir.opposite().clockwise()
    } else {
        // counter-clockwise
        |dir: Direction| dir.clockwise()
    };

    // the vertices coordinates correspond to the start of each step inflated on
    // the outside by 1 to account for the grid cell occupied by the border
    let vertices: Vec<_> = segment_loop
        .positions(Position::default())
        .zip(&segment_loop.steps)
        .map(|(step_start, step)| step_start.step(outside(step.direction), 1))
        .collect();
    let area = shoelace_formula(&vertices);
    area - 1
}

#[aoc(day18, part1)]
fn part1(input: &str) -> usize {
    let segment_loop = parse_regular(input);

    #[cfg(feature = "draw-visuals")]
    draw_loop_as_svg_path(&segment_loop, "day18-p1.svg");

    exterior_area_of_loop(&segment_loop) as usize
}

#[aoc(day18, part2)]
fn part2(input: &str) -> usize {
    let segment_loop = parse_alternate(input);

    #[cfg(feature = "draw-visuals")]
    draw_loop_as_svg_path(&segment_loop, "day18-p2.svg");

    exterior_area_of_loop(&segment_loop) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part2_parser() {
        let input = unindent::unindent(
            "
            R 6 (#70c710)
            D 5 (#0dc571)
            L 2 (#5713f0)
            D 2 (#d2c081)
            R 2 (#59c680)
            D 2 (#411b91)
            L 5 (#8ceee2)
            U 2 (#caa173)
            L 1 (#1b58a2)
            U 2 (#caa171)
            R 2 (#7807d2)
            U 3 (#a77fa3)
            L 2 (#015232)
            U 2 (#7a21e3)
            ",
        );

        let equivalent_input = unindent::unindent(
            "
            R 461937 (#000000)
            D 56407 (#000000)
            R 356671 (#000000)
            D 863240 (#000000)
            R 367720 (#000000)
            D 266681 (#000000)
            L 577262 (#000000)
            U 829975 (#000000)
            L 112010 (#000000)
            D 829975 (#000000)
            L 491645 (#000000)
            U 686074 (#000000)
            L 5411 (#000000)
            U 500254 (#000000)
            ",
        );

        dbg!(parse_alternate(&input));
        assert_eq!(
            parse_alternate(&input).steps,
            parse_regular(&equivalent_input).steps
        );
    }

    #[test]
    fn part2_simple() {
        // rectangle of interior size 3x3 (4x4 including the border)
        let segment_loop = parse_regular("R 3\nD 3\nL 3\nU 3");
        assert_eq!(exterior_area_of_loop(&segment_loop), 16);
    }
}

example_tests! {
    parser: None,

    // TODO: find out why unindent is broken here
    "
    R 6 (#70c710)
    D 5 (#0dc571)
    L 2 (#5713f0)
    D 2 (#d2c081)
    R 2 (#59c680)
    D 2 (#411b91)
    L 5 (#8ceee2)
    U 2 (#caa173)
    L 1 (#1b58a2)
    U 2 (#caa171)
    R 2 (#7807d2)
    U 3 (#a77fa3)
    L 2 (#015232)
    U 2 (#7a21e3)
    ",
    part1 => 62,
    part2 => 952408144115,
}

known_input_tests! {
    parser: None,
    input: include_str!("../input/2023/day18.txt"),
    part1 => 40714,
    part2 => 129849166997110,
}
