use std::{
    collections::{HashMap, VecDeque},
    iter::Sum,
    ops::{Add, Mul},
};

use aoc_runner_derive::{aoc, aoc_generator};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct PulseCounter {
    low: u32,
    high: u32,
}

impl PulseCounter {
    fn total_for_solution(&self) -> u64 {
        self.low as u64 * self.high as u64
    }

    fn add_pulse(&mut self, pulse: Pulse) {
        match pulse {
            Pulse::Low => self.low += 1,
            Pulse::High => self.high += 1,
        }
    }
}

impl Add for PulseCounter {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            low: self.low + rhs.low,
            high: self.high + rhs.high,
        }
    }
}

impl Mul<usize> for PulseCounter {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self {
            low: (self.low as usize)
                .checked_mul(rhs)
                .unwrap()
                .try_into()
                .unwrap(),
            high: (self.high as usize)
                .checked_mul(rhs)
                .unwrap()
                .try_into()
                .unwrap(),
        }
    }
}

impl Sum for PulseCounter {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Add::add)
    }
}

impl std::fmt::Display for PulseCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.total_for_solution().fmt(f)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Pulse {
    #[default]
    Low = 0,
    High = 1,
}

impl std::hash::Hash for Pulse {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state)
    }
}

impl Pulse {
    fn invert(self) -> Self {
        match self {
            Pulse::Low => Pulse::High,
            Pulse::High => Pulse::Low,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Label([u8; 4]);

impl Label {
    const BROADCASTER: Self = Self([0; 4]);

    fn new(label: &str) -> Self {
        let mut bytes = [0; 4];
        let label_bytes = label.as_bytes();
        // panics if label is too long
        bytes[..label_bytes.len()].copy_from_slice(label_bytes);
        Self(bytes)
    }
}

impl std::fmt::Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Label({self})")
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0[0] == 0 {
            return write!(f, "broadcaster");
        }
        for &b in &self.0 {
            if b == 0 {
                break;
            }
            write!(f, "{}", b as char)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModuleType {
    Broadcaster,
    FlipFlop,
    Conjunction,
    Sink,
}

use ModuleType::*;

use crate::{
    testing::{example_tests, known_input_tests},
    utils::NumberIteratorExt,
};

use self::parsing::Line;

#[derive(Debug, Clone)]
struct Module {
    module_type: ModuleType,
    incoming: Vec<usize>,
    outgoing: Vec<usize>,
}

#[derive(Debug, Clone)]
struct WiringConfiguration {
    modules: Vec<Module>,
    broadcaster_index: usize,
    mixer_index: Option<usize>,
}

impl<'a> FromIterator<&'a Line> for WiringConfiguration {
    fn from_iter<T: IntoIterator<Item = &'a Line>>(iter: T) -> Self {
        let mut map = HashMap::new();
        let mut modules = Vec::new();
        let mut connections = Vec::new();

        for line in iter {
            // if line.connections == vec![Label::new("rx")] {
            //     // ok this is a hack let's say this is THE sink
            //     assert!(line.module_type == Conjunction);
            //     // println!("found sink {}", line.label);
            //     let module = Module {
            //         module_type: ModuleType::Sink,
            //         incoming: Vec::new(),
            //         outgoing: Vec::new(),
            //     };
            //     map.insert(line.label, modules.len());
            //     modules.push(module);
            //     continue;
            // }
            for &outgoing_label in &line.connections {
                connections.push((line.label, outgoing_label));
            }
            let module = Module {
                module_type: line.module_type,
                incoming: Vec::new(),
                outgoing: Vec::new(),
            };
            map.insert(line.label, modules.len());
            modules.push(module);
        }

        // there could be one sink (if it's not the trivial example) but not
        // more than one otherwise we don't know how to solve part 2
        let mut sink_index = None;
        for (_, outgoing_label) in &connections {
            if !map.contains_key(outgoing_label) {
                let sink = Module {
                    module_type: Sink,
                    incoming: Vec::new(),
                    outgoing: Vec::new(),
                };
                map.insert(*outgoing_label, modules.len());
                debug_assert!(sink_index.is_none());
                sink_index = Some(modules.len());
                modules.push(sink);
            }
        }

        for (incoming_label, outgoing_label) in &connections {
            let incoming_index = map[incoming_label];
            let outgoing_index = map[outgoing_label];
            modules[incoming_index].outgoing.push(outgoing_index);
            modules[outgoing_index].incoming.push(incoming_index);
        }

        let broadcaster_index = map[&Label::BROADCASTER];
        // The "mixer" is the conjunction module that is connected to the sink
        // (in inputs compliant with part 2)
        let mixer_index = sink_index.map(|i| modules[i].incoming[0]);

        Self {
            modules,
            broadcaster_index,
            mixer_index,
        }
    }
}

mod parsing {
    use std::str::FromStr;

    use super::*;

    #[derive(Debug, Clone)]
    pub(super) struct Line {
        pub(super) module_type: ModuleType,
        pub(super) label: Label,
        pub(super) connections: Vec<Label>,
    }

    impl FromStr for Line {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut parts = s.split("->").map(str::trim);
            let label_part = parts
                .next()
                .ok_or_else(|| format!("Line must be separated by -> ({s:?})"))?;
            let connections_part = parts
                .next()
                .ok_or_else(|| format!("Missing connections after -> ({s:?})"))?;

            let (module_type, label) = if label_part == "broadcaster" {
                (Broadcaster, Label::BROADCASTER)
            } else {
                let type_char = label_part
                    .chars()
                    .next()
                    .ok_or_else(|| "Label should have at least two chars".to_string())?;
                let raw_label = label_part.get(1..).ok_or_else(|| {
                    format!("Label should have at least two chars: {label_part:?}")
                })?;
                let label = Label::new(raw_label);
                let module_type = match type_char {
                    '%' => super::ModuleType::FlipFlop,
                    '&' => super::ModuleType::Conjunction,
                    _ => return Err(format!("Unknown module type: {type_char}")),
                };
                (module_type, label)
            };

            let connections = connections_part
                .split(',')
                .map(str::trim)
                .map(Label::new)
                .collect();

            Ok(Self {
                module_type,
                label,
                connections,
            })
        }
    }
}

struct Simulator<'a> {
    wiring: &'a WiringConfiguration,
    memory: Vec<Pulse>,
    memory_map: HashMap<usize, usize>,
}

impl<'a> Simulator<'a> {
    fn new(wiring: &'a WiringConfiguration) -> Self {
        let mut memory_map = HashMap::new();
        let mut next_address = 0;
        for (index, module) in wiring.modules.iter().enumerate() {
            match module.module_type {
                Broadcaster | Sink => {}
                FlipFlop => {
                    memory_map.insert(index, next_address);
                    next_address += 1;
                }
                Conjunction => {
                    memory_map.insert(index, next_address);
                    next_address += module.incoming.len();
                }
            }
        }
        let memory = vec![Pulse::Low; next_address];
        Self {
            wiring,
            memory,
            memory_map,
        }
    }

    fn pulse_button(&mut self) -> (PulseCounter, Option<usize>) {
        self.simulate_one_branch(self.wiring.broadcaster_index, Pulse::Low)
    }

    fn simulate_one_branch(
        &mut self,
        input_index: usize,
        input: Pulse,
    ) -> (PulseCounter, Option<usize>) {
        let mut counter = PulseCounter::default();

        let mut queue: VecDeque<_> = [(
            // initial source is the "button" in theory but we don't care
            self.wiring.broadcaster_index,
            input,
            input_index,
        )]
        .into();

        let mut pulsed_mixer = None;

        while let Some((source, pulse, label)) = queue.pop_front() {
            counter.add_pulse(pulse);
            let module = &self.wiring.modules[label];
            let new_pulse = match module.module_type {
                // When it receives a pulse, [the broadcast module] sends the
                // same pulse to all of its destination modules.
                Broadcaster => Some(pulse),

                // if a flip-flop module receives a low pulse, it flips between
                // on and off. If it was off, it turns on and sends a high
                // pulse. If it was on, it turns off and sends a low pulse
                FlipFlop if pulse == Pulse::Low => {
                    // we decide that the flip-flop internal state is
                    // Low => off, High => on
                    let address = self.memory_map[&label];
                    let old_pulse = self.memory[address];
                    let new_pulse = old_pulse.invert();
                    self.memory[address] = new_pulse;
                    Some(new_pulse)
                }

                // If a flip-flop module receives a high pulse, it is ignored
                // and nothing happens
                FlipFlop => None,

                // When a pulse is received, the conjunction module first
                // updates its memory for that input. Then, if it remembers high
                // pulses for all inputs, it sends a low pulse; otherwise, it
                // sends a high pulse.
                Conjunction => {
                    let start_address = self.memory_map[&label];
                    let end_address = start_address + module.incoming.len();
                    let memory = &mut self.memory[start_address..end_address];
                    let pos = module.incoming.iter().position(|&l| l == source).unwrap();
                    memory[pos] = pulse;
                    if Some(label) == self.wiring.mixer_index && pulse == Pulse::High {
                        pulsed_mixer = Some(pos);
                        // println!("mixer: {memory:?}");
                    }
                    let new_pulse = if memory.iter().all(|&p| p == Pulse::High) {
                        Pulse::Low
                    } else {
                        Pulse::High
                    };
                    Some(new_pulse)
                }

                Sink => None,
            };

            // propagate the pulse to all outgoing connections
            if let Some(new_pulse) = new_pulse {
                for &outgoing_label in &module.outgoing {
                    queue.push_back((label, new_pulse, outgoing_label));
                }
            }
        }

        (counter, pulsed_mixer)
    }
}

#[allow(dead_code)]
fn find_cycle(wiring: &WiringConfiguration, max: usize) -> (usize, PulseCounter) {
    let mut simulator = Simulator::new(wiring);

    let mut counter = PulseCounter::default();
    for i in 0..max {
        let (new_counter, _) = simulator.pulse_button();
        counter = counter + new_counter;
        if simulator.memory.iter().all(|&p| p == Pulse::Low) {
            return (i + 1, counter);
        }
    }
    (0, counter)
}

#[aoc_generator(day20)]
fn parse(input: &str) -> Vec<Line> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.parse().unwrap())
        .collect()
}

#[aoc(day20, part1)]
fn part1(input: &[Line]) -> PulseCounter {
    let config = WiringConfiguration::from_iter(input);
    let mut simulator = Simulator::new(&config);
    (0..1000).map(|_| simulator.pulse_button().0).sum()
}

fn completely_ad_hoc_solution_to_part_2(lines: &[Line]) -> usize {
    // Instead of a generic solution, we learned that the input network always
    // has the same structure: the broadcaster is connected to N (apparently
    // always 4?) subnetworks, which all output to the same "sink" node which is
    // used to "turn on the machine" when they are all pulsed high. The
    // subnetworks, in turn, follow a simple pattern: a cascade of flip-flops,
    // acting as a counter (apparently always 12 bits?) and a conjunction module
    // that is pulsed when the counter reaches a certain value. The conjunction
    // module has two functions: 1) it resets the counter to 0 and 2) it sends a
    // pulse to the final stage, which is an inverter, and then to the sink. In
    // practice, each subnetwork structure encodes a binary number, which is the
    // number of clicks before it resets. As an ad hoc solution, we can simply
    // read the network and extract the encoded numbers, then compute the least
    // common multiple. This way we don't have to run any simulation.

    let map = lines
        .iter()
        .map(|line| (line.label, line))
        .collect::<HashMap<_, _>>();

    // identify subnetworks entry points
    let entry_points = &map[&Label::BROADCASTER].connections;

    entry_points
        .iter()
        .map(|entry_point| {
            let mut next = Some(entry_point);
            let mut number = 0;
            let mut shift = 0;
            while let Some(current) = next.take() {
                let line = &map[&current];
                for connection in line.connections.iter().map(|label| map[&label]) {
                    if connection.module_type == FlipFlop {
                        next.replace(&connection.label);
                    } else if connection.module_type == Conjunction {
                        number |= 1 << shift;
                    }
                }
                shift += 1;
            }
            number
        })
        .least_common_multiple()
}

#[aoc(day20, part2)]
fn part2(input: &[Line]) -> usize {
    let config = WiringConfiguration::from_iter(input);
    // This is less "ad hoc" than part2_ad_hoc because we actually simulate the
    // network, but we are still making a lot of assumptions. In particular, we
    // assume that when the "mixer" is pulsed high, we are at the end of a
    // cycle, and that all subnetwork cycles have different lengths. This is
    // true for the input, but it's not a general solution. Funnily enough, this
    // is not true for the example, which actually turns the machine on after
    // the first click, before any cycle is completed; and funnily enough, the
    // ad hoc solution instead works for the example almost by coincidence.
    let mut simulator = Simulator::new(&config);
    let inputs_to_sink = config.modules[config.mixer_index.unwrap()].incoming.len();
    let mut cycle_numbers = Vec::new();
    let mut mask = vec![false; inputs_to_sink];
    let mut click_count = 0;
    while mask.iter().any(|&p| !p) {
        let (_, pulsed) = simulator.pulse_button();
        click_count += 1;

        if let Some(pulsed) = pulsed {
            mask[pulsed] = true;
            cycle_numbers.push(click_count);
        }
        debug_assert!(click_count <= (1 << 12), "{click_count} {mask:?}");
    }
    cycle_numbers.into_iter().least_common_multiple()
}

#[aoc(day20, part2, ad_hoc)]
fn part2_ad_hoc(input: &[Line]) -> usize {
    completely_ad_hoc_solution_to_part_2(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_simple() {
        let lines = parse(
            "broadcaster -> a, b, c
            %a -> b
            %b -> c
            %c -> inv
            &inv -> a
            ",
        );
        let result = part1(&lines);
        assert_eq!(
            result,
            PulseCounter {
                low: 8000,
                high: 4000
            }
        );
    }

    #[test]
    fn part1_steps() {
        let lines = parse(
            "
            broadcaster -> a
            %a -> inv, con
            &inv -> b
            %b -> con
            &con -> out
            ",
        );
        let wiring = WiringConfiguration::from_iter(&lines);
        let mut simulator = Simulator::new(&wiring);
        // this is a cycle, so repeat an arbitrary number of times and expect the same result
        for _ in 0..10 {
            assert_eq!(
                simulator.pulse_button(),
                (PulseCounter { low: 4, high: 4 }, Some(1))
            );
            assert_eq!(
                simulator.pulse_button(),
                (PulseCounter { low: 4, high: 2 }, None)
            );
            assert_eq!(
                simulator.pulse_button(),
                (PulseCounter { low: 5, high: 3 }, Some(0))
            );
            assert_eq!(
                simulator.pulse_button(),
                (PulseCounter { low: 4, high: 2 }, None)
            );
            assert!(simulator.memory.iter().all(|&p| p == Pulse::Low));
        }
    }

    #[test]
    fn part1_cycle() {
        let lines = parse(
            "
            broadcaster -> a
            %a -> inv, con
            &inv -> b
            %b -> con
            &con -> out
            ",
        );
        let config = WiringConfiguration::from_iter(&lines);
        let (n, counter) = find_cycle(&config, 1000);
        assert_eq!(n, 4);
        assert_eq!(counter, PulseCounter { low: 17, high: 11 });
    }

    #[test]
    fn part1_display() {
        let lines = parse(
            "broadcaster -> a, b, c
            %a -> b
            %b -> c
            %c -> inv
            &inv -> a
            ",
        );
        let result = part1(&lines);
        assert_eq!(result.to_string(), "32000000");
    }
}

example_tests! {
    "
    broadcaster -> a
    %a -> inv, con
    &inv -> b
    %b -> con
    &con -> out
    ",
    part1 => super::PulseCounter { low: 4250, high: 2750 },

    // There is no part 2 example, because the example "turns the machine on"
    // after one click. Our part2 solution is broken for this case. The ad hoc
    // solution works by coincidence.

    // part2 => 1, // bad boy part 2

    part2_ad_hoc => 1,
}

known_input_tests! {
    input: include_str!("../input/2023/day20.txt"),
    part1 => super::PulseCounter { low: 16656, high: 42780 },
    part2 => 238920142622879,
    part2_ad_hoc => 238920142622879,
}
