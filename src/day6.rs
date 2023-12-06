use aoc_runner_derive::{aoc, aoc_generator};

use crate::{
    range::HasExtent,
    testing::{example_tests, known_input_tests},
};

#[derive(Debug)]
struct Race {
    time: u64,
    record_distance: u64,
}

impl Race {
    fn press_time_to_beat_record(&self) -> std::ops::Range<u64> {
        // solutions to inequality x * (time - x) > record_distance
        //
        // T/2 -+ sqrt(T^2 - 4D)/2
        debug_assert!(self.time.pow(2) >= self.record_distance * 4);
        let time_half = 0.5 * self.time as f64;
        let delta_squared = self.time.pow(2) - self.record_distance * 4;
        let delta_half = 0.5 * (delta_squared as f64).sqrt();
        let lo = time_half - delta_half;
        let hi = time_half + delta_half;

        debug_assert!(lo > 0.0 && hi > 0.0 && hi > lo);
        // lo and hi give a distance equal to the record. We need the closest integer
        // that beats it.
        let start = int_larger_than_float(lo);
        let end = int_smaller_than_float(hi) + 1;
        start..end
    }
}

fn int_larger_than_float(n: f64) -> u64 {
    let n_int = n.ceil() as u64;
    if n.fract() == 0.0 {
        n_int + 1
    } else {
        n_int
    }
}

fn int_smaller_than_float(n: f64) -> u64 {
    let n_int = n.floor() as u64;
    if n.fract() == 0.0 {
        n_int - 1
    } else {
        n_int
    }
}

#[aoc_generator(day6)]
fn parse(input: &str) -> Vec<Race> {
    let mut lines = input.lines();
    let time_line = lines
        .next()
        .and_then(|line| line.strip_prefix("Time:"))
        .expect("should have a Time line");
    let times = time_line
        .split_ascii_whitespace()
        .map(|x| x.parse().expect("should be a number"));
    let distance_line = lines
        .next()
        .and_then(|line| line.strip_prefix("Distance:"))
        .expect("should have a Distance line");
    let record_distances = distance_line
        .split_ascii_whitespace()
        .map(|x| x.parse().expect("should be a number"));
    times
        .zip(record_distances)
        .map(|(time, record_distance)| Race {
            time,
            record_distance,
        })
        .collect()
}

#[aoc(day6, part1)]
fn part1(input: &[Race]) -> u64 {
    #[cfg(debug_assertions)]
    for race in input {
        dbg!(race);
        dbg!(race.press_time_to_beat_record());
    }
    input
        .iter()
        .map(|race| race.press_time_to_beat_record().extent())
        .fold(1, std::ops::Mul::mul)
}

fn join_times(races: &[Race]) -> Race {
    // we could have a parser specific for part 2 but it's not a fun exercise so
    // I won't even bother and just take the already parsed result.
    let time: u64 = races
        .iter()
        .map(|race| format!("{}", race.time))
        .collect::<Vec<_>>()
        .join("")
        .parse()
        .unwrap();
    let record_distance: u64 = races
        .iter()
        .map(|race| format!("{}", race.record_distance))
        .collect::<Vec<_>>()
        .join("")
        .parse()
        .unwrap();
    Race {
        time,
        record_distance,
    }
}
#[aoc(day6, part2)]
fn part2(input: &[Race]) -> u64 {
    let race = join_times(input);
    race.press_time_to_beat_record().extent()
}

#[cfg(test)]
mod tests {}

example_tests! {
    "
    Time:      7  15   30
    Distance:  9  40  200
    ",

    part1 => 288,
    part2 => 71503,
}

known_input_tests! {
    input: include_str!("../input/2023/day6.txt"),
    part1 => 608902,
    part2 => 46173809,
}
