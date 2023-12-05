#[derive(Debug)]
struct MappedRange {
    destination: u32,
    source: u32,
    length: u32,
}

impl MappedRange {
    fn map(&self, source: u32) -> Option<u32> {
        if source >= self.source && source - self.source < self.length {
            Some(self.destination + source - self.source)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Map {
    from: String,
    to: String,
    ranges: Vec<MappedRange>,
}

impl Map {
    fn map(&self, source: u32) -> u32 {
        // assuming no two ranges are overlapping - might need to check later
        self.ranges
            .iter()
            .find_map(|range| range.map(source))
            .unwrap_or(source)
    }
}

#[derive(Debug)]
struct Almanac {
    seeds: Vec<u32>,
    maps: Vec<Map>,
}

impl Almanac {
    fn seed_ranges(&self) -> impl Iterator<Item = Range<u32>> + '_ {
        self.seeds.chunks(2).map(|chunk| {
            let start = chunk[0];
            let length = chunk[1];
            start..start + length
        })
    }

    /// Map through all the maps in order
    fn map_seed(&self, seed: u32) -> u32 {
        self.maps.iter().fold(seed, |source, map| map.map(source))
    }
}

fn expect_empty_line<'a, I: Iterator<Item = &'a str>>(mut lines: I) -> Option<()> {
    match lines.next() {
        Some("") => Some(()),
        Some(line) => {
            eprintln!("expected blank line; found: {line}");
            None
        }
        _ => None,
    }
}

use std::ops::Range;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::example_tests;
#[aoc_generator(day5)]
fn parse(input: &str) -> Almanac {
    // let's just parse verbatim because we have no idea what part2 might ask
    let mut lines = input.lines();

    let seeds_line = lines.next().expect("should have at least one line");
    let seeds = seeds_line
        .strip_prefix("seeds: ")
        .expect("should have a 'seeds: ' line")
        .split_ascii_whitespace()
        .map(|n| n.parse().expect("seeds should be u32 numbers"))
        .collect();
    expect_empty_line(&mut lines).expect("should have an empty line after seeds");

    let mut maps = Vec::new();
    while let Some(header_line) = lines.next() {
        let map_name = header_line
            .strip_suffix(" map:")
            .expect("should have a map name");
        let (from, to) = {
            let mut split_name = map_name.split("-to-");
            let from = split_name
                .next()
                .expect("name should have a from part")
                .to_string();
            let to = split_name
                .next()
                .expect("name should have a to part")
                .to_string();
            assert!(split_name.next().is_none());
            (from, to)
        };

        let ranges =
            lines
                .by_ref()
                .take_while(|line| !line.is_empty())
                .map(|line| {
                    let mut split_range = line.split_ascii_whitespace();
                    let destination = split_range
                        .next()
                        .and_then(|n| n.parse().ok())
                        .expect("should have a destination range start");
                    let source = split_range
                        .next()
                        .and_then(|n| n.parse().ok())
                        .expect("should have a source range start");
                    let length = split_range
                        .next()
                        .and_then(|n| n.parse().ok())
                        .expect("should have a range length");
                    assert!(split_range.next().is_none());
                    MappedRange {
                        destination,
                        source,
                        length,
                    }
                })
                .collect();
        maps.push(Map { from, to, ranges })
    }
    for window in maps.windows(2) {
        debug_assert_eq!(window[0].to, window[1].from);
    }

    Almanac { seeds, maps }
}

#[aoc(day5, part1)]
fn part1(almanac: &Almanac) -> u32 {
    let locations: Vec<_> = almanac
        .seeds
        .iter()
        .map(|&seed| almanac.map_seed(seed))
        .collect();
    locations.into_iter().min().unwrap()
}

#[aoc(day5, part2)]
fn part2(almanac: &Almanac) -> u32 {
    // brute force because computers are fast - but I want something
    // better later
    almanac
        .seed_ranges()
        .flatten()
        .map(|seed| almanac.map_seed(seed))
        .min()
        .unwrap()
}

#[cfg(test)]
mod tests {}

example_tests! {
    "
    seeds: 79 14 55 13

    seed-to-soil map:
    50 98 2
    52 50 48
    
    soil-to-fertilizer map:
    0 15 37
    37 52 2
    39 0 15
    
    fertilizer-to-water map:
    49 53 8
    0 11 42
    42 0 7
    57 7 4
    
    water-to-light map:
    88 18 7
    18 25 70
    
    light-to-temperature map:
    45 77 23
    81 45 19
    68 64 13
    
    temperature-to-humidity map:
    0 69 1
    1 0 69
    
    humidity-to-location map:
    60 56 37
    56 93 4
    ",

    part1 => 35,
    part2 => 46,
}
