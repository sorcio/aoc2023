use crate::range::{Interval, Overlaps};

// forgive me but I renamed things later and I don't want to change the whole code
type Range = Interval<u32>;

#[derive(Debug, Clone)]
struct MappedRange {
    source: Range,
    destination: Range,
}

impl MappedRange {
    fn from_triplet(destination: u32, source: u32, length: u32) -> Self {
        Self {
            source: Range::new(source, length),
            destination: Range::new(destination, length),
        }
    }
    fn map(&self, source: u32) -> Option<u32> {
        self.source
            .distance_from_start(source)
            .map(|distance| self.destination.start().checked_add(distance).unwrap())
    }

    /// Return a destination range that overlaps with the given source range, or
    /// None if no overlap exists.
    fn map_range(&self, source_range: &Range) -> Option<Range> {
        self.source
            .intersection(source_range)
            .map(|source_overlap| {
                let destination_start = self.map(source_overlap.start()).unwrap();
                Range::new(destination_start, source_overlap.len())
            })
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

    fn map_range(&self, source_range: Range) -> impl Iterator<Item = Range> + '_ {
        let ranges = {
            let mut copy: Vec<_> = self
                .ranges
                .iter()
                .filter(|range| range.source.overlaps(&source_range))
                .collect();
            copy.sort_by_key(|range| range.source.start());
            copy
        };
        let mut result = Vec::new();
        let mut last = source_range.start() as u64;
        for range in ranges {
            if range.source.start() as u64 > last {
                result.push(Range::excl(last.try_into().unwrap(), range.source.start()));
            }
            result.push(range.map_range(&source_range).unwrap());
            last = range.source.end();
        }
        if source_range.end() > last {
            result.push(
                Range::new(last.try_into().unwrap(), u32::MAX - 1)
                    .intersection(&source_range)
                    .unwrap(),
            );
        }
        result.into_iter()
    }
}

#[derive(Debug)]
struct Almanac {
    seeds: Vec<u32>,
    maps: Vec<Map>,
}

impl Almanac {
    fn seed_ranges(&self) -> impl Iterator<Item = Range> + '_ {
        self.seeds.chunks(2).map(|chunk| {
            let start = chunk[0];
            let length = chunk[1];
            Range::new(start, length)
        })
    }

    /// Map through all the maps in order
    fn map_seed(&self, seed: u32) -> u32 {
        self.maps.iter().fold(seed, |source, map| map.map(source))
    }

    /// Map the whole range through all the maps in order
    fn map_seed_range(&self, seed_range: Range) -> impl Iterator<Item = Range> + '_ {
        self.maps
            .iter()
            .fold(vec![seed_range], move |source_ranges, map| {
                source_ranges
                    .into_iter()
                    .flat_map(|source| map.map_range(source))
                    .collect()
            })
            .into_iter()
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

        let ranges = lines
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
                MappedRange::from_triplet(destination, source, length)
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
    // rust-analyzer seems to be very confused by the aoc macro for some reason
    // so I wrote the implementation as a separate function :/
    part2_impl(almanac)
}

fn part2_impl(almanac: &Almanac) -> u32 {
    almanac
        .seed_ranges()
        .flat_map(|seed_range| almanac.map_seed_range(seed_range.clone()))
        .min_by_key(|location_range| location_range.start())
        .unwrap()
        .start()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapped_range_map_range() {
        let range = MappedRange {
            source: Range::new(10, 40),
            destination: Range::new(100, 40),
        };
        assert_eq!(range.map_range(&Range::excl(0, 10)), None);
        assert_eq!(
            range.map_range(&Range::excl(0, 11)),
            Some(Range::excl(100, 101))
        );
        assert_eq!(
            range.map_range(&Range::excl(0, 50)),
            Some(Range::excl(100, 140))
        );
        assert_eq!(
            range.map_range(&Range::excl(0, 60)),
            Some(Range::excl(100, 140))
        );
        assert_eq!(range.map_range(&Range::excl(10, 10)), None);
        assert_eq!(
            range.map_range(&Range::excl(10, 15)),
            Some(Range::excl(100, 105))
        );
        assert_eq!(
            range.map_range(&Range::excl(10, 50)),
            Some(Range::excl(100, 140))
        );
        assert_eq!(
            range.map_range(&Range::excl(10, 60)),
            Some(Range::excl(100, 140))
        );
        assert_eq!(range.map_range(&Range::excl(20, 20)), None);
        assert_eq!(
            range.map_range(&Range::excl(20, 25)),
            Some(Range::excl(110, 115))
        );
        assert_eq!(
            range.map_range(&Range::excl(20, 50)),
            Some(Range::excl(110, 140))
        );
        assert_eq!(
            range.map_range(&Range::excl(20, 60)),
            Some(Range::excl(110, 140))
        );
        assert_eq!(range.map_range(&Range::excl(50, 50)), None);
        assert_eq!(range.map_range(&Range::excl(50, 60)), None);
        assert_eq!(range.map_range(&Range::excl(60, 70)), None);
    }

    #[test]
    fn map_map_range() {
        let map = Map {
            from: "a".to_string(),
            to: "b".to_string(),
            ranges: vec![
                // 10..20 -> 25..35
                MappedRange::from_triplet(25, 10, 10),
                // 22..24 -> 2..4
                MappedRange::from_triplet(2, 22, 2),
            ],
        };
        assert_eq!(
            map.map_range(Range::excl(10, 20)).collect::<Vec<_>>(),
            vec![Range::excl(25, 35)]
        );
        assert_eq!(
            map.map_range(Range::excl(22, 24)).collect::<Vec<_>>(),
            vec![Range::excl(2, 4)]
        );
        assert_eq!(
            map.map_range(Range::excl(0, 10)).collect::<Vec<_>>(),
            vec![Range::excl(0, 10)]
        );
        assert_eq!(
            map.map_range(Range::excl(5, 10)).collect::<Vec<_>>(),
            vec![Range::excl(5, 10)]
        );
        assert_eq!(
            map.map_range(Range::excl(5, 15)).collect::<Vec<_>>(),
            vec![Range::excl(5, 10), Range::excl(25, 30)]
        );
        assert_eq!(
            map.map_range(Range::excl(5, 25)).collect::<Vec<_>>(),
            vec![
                Range::excl(5, 10),
                Range::excl(25, 35),
                Range::excl(20, 22),
                Range::excl(2, 4),
                Range::excl(24, 25)
            ]
        );
    }

    #[test]
    fn map_map_range_with_our_input() {
        let test_input = unindent::unindent(
            "
        seeds: 10 10

        a-to-b map:
        2955816171 2260659770 927037009
        1906648752 2188942242 71717528
        848878920 35928575 8026852
        4100692468 1994667414 194274828
        2066384942 3405536067 889431229
        559945395 1052613350 288933525
        3882853180 3187696779 217839288
        856905772 1341546875 164300625
        0 528596530 524016820
        1978366280 1723924810 88018662
        1044385850 67134880 400987760
        524016820 0 35928575
        1021206397 43955427 23179453
        1445373610 468122640 60473890
        1723924810 1811943472 182723942",
        );
        let almanac = parse(&test_input);
        let map = &almanac.maps[0];

        assert_eq!(
            map.map_range(Range::excl(0, 35928575)).collect::<Vec<_>>(),
            vec![Range::new(524016820, 35928575)]
        );
        // ...?
    }
}

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
