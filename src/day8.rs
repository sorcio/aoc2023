use std::collections::HashMap;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::{
    testing::{example_tests, known_input_tests},
    utils::NumberIteratorExt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeId(usize);

#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
}

impl From<char> for Direction {
    fn from(c: char) -> Self {
        match c {
            'L' => Direction::Left,
            'R' => Direction::Right,
            _ => panic!("invalid direction: {}", c),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NodeType {
    Start,
    End,
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    left: NodeId,
    right: NodeId,
    node_type: NodeType,
}

impl Node {
    fn next(&self, direction: Direction) -> NodeId {
        match direction {
            Direction::Left => self.left,
            Direction::Right => self.right,
        }
    }
}

struct Network {
    nodes: Vec<Node>,
}

impl Network {
    fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    fn next(&self, id: NodeId, direction: Direction) -> NodeId {
        self.node(id).next(direction)
    }

    fn sequence(&self, start: NodeId, sequence: &[Direction]) -> NodeId {
        sequence
            .iter()
            .fold(start, |current, direction| self.next(current, *direction))
    }

    fn nodes(&self) -> impl Iterator<Item = NodeId> {
        (0..self.nodes.len()).map(NodeId)
    }
}

struct GraphBuilder<'s> {
    nodes: Vec<(&'s str, &'s str, &'s str)>,
    node_map: HashMap<String, NodeId>,
}

impl<'s> GraphBuilder<'s> {
    fn add_node(&mut self, node: &'s str, left: &'s str, right: &'s str) {
        let node_id = NodeId(self.nodes.len());
        let previous = self.node_map.insert(node.to_string(), node_id);
        assert!(previous.is_none(), "duplicate node: {}", node);
        self.nodes.push((node, left, right));
    }

    fn build(self) -> (Network, HashMap<String, NodeId>) {
        let nodes = self
            .nodes
            .into_iter()
            .enumerate()
            .map(|(i, (node, left, right))| {
                debug_assert!(self.node_map[node] == NodeId(i));
                let left_id = self.node_map[left];
                let right_id = self.node_map[right];

                match node.chars().last().unwrap() {
                    'A' => Node {
                        left: left_id,
                        right: right_id,
                        node_type: NodeType::Start,
                    },
                    'Z' => Node {
                        left: left_id,
                        right: right_id,
                        node_type: NodeType::End,
                    },
                    _ => Node {
                        left: left_id,
                        right: right_id,
                        node_type: NodeType::Normal,
                    },
                }
            })
            .collect();
        (Network { nodes }, self.node_map)
    }
}

fn parse_network(input: &str) -> (Network, HashMap<String, NodeId>) {
    let mut builder = GraphBuilder {
        nodes: Vec::new(),
        node_map: HashMap::new(),
    };

    for line in input.lines() {
        // a line looks like "XXX = (YYY, ZZZ)" and we know all labels are 3
        // characters so let's forget about validation
        debug_assert_eq!(&line[3..7], " = (");
        let node = &line[0..3];
        let left = &line[7..10];
        let right = &line[12..15];
        builder.add_node(node, left, right);
    }

    builder.build()
}

struct Day8Map {
    network: Network,
    node_map: HashMap<String, NodeId>,
    sequence: Vec<Direction>,
}

#[aoc_generator(day8)]
fn parse(input: &str) -> Day8Map {
    let mut split_input = input.split("\n\n");
    let sequence_line = split_input
        .next()
        .expect("should have at least one line")
        .trim_end();
    let sequence = sequence_line.chars().map(Direction::from).collect();

    let (network, node_map) =
        parse_network(split_input.next().expect("should have a network map part"));

    Day8Map {
        network,
        node_map,
        sequence,
    }
}

#[aoc(day8, part1)]
fn part1(input: &Day8Map) -> usize {
    let start = input.node_map["AAA"];
    let end = input.node_map["ZZZ"];
    let sequence = &input.sequence;
    let mut total_steps = 0;
    let mut current = start;
    while current != end {
        current = input.network.sequence(current, sequence);
        total_steps += sequence.len();
    }
    total_steps
}

#[aoc(day8, part2)]
fn part2(input: &Day8Map) -> usize {
    let sequence = &input.sequence;

    // precompute the application of the sequence to each node
    let destinations: Vec<_> = input
        .network
        .nodes()
        .map(|node_id| input.network.sequence(node_id, sequence))
        .collect();

    // compute the lcm of the number of steps for each start node
    input
        .network
        .nodes()
        .filter(|&node_id| input.network.node(node_id).node_type == NodeType::Start)
        .map(|start| {
            let mut total_steps = 0;
            let mut current = start;
            while input.network.node(current).node_type != NodeType::End {
                current = destinations[current.0];
                total_steps += sequence.len();
            }
            total_steps
        })
        .least_common_multiple()
}

#[aoc(day8, part2, brute_force)]
fn part2_brute_force(input: &Day8Map) -> usize {
    // including the brute force solution because it's the first one I wrote and
    // it actually found the result in reasonable time
    let sequence = &input.sequence;

    let destinations: Vec<_> = input
        .network
        .nodes()
        .map(|node_id| input.network.sequence(node_id, sequence))
        .collect();

    let start_nodes = input
        .network
        .nodes()
        .filter(|&node_id| input.network.node(node_id).node_type == NodeType::Start);

    let mut total_steps = 0;
    let mut current_nodes: Vec<_> = start_nodes.collect();
    while current_nodes
        .iter()
        .any(|&node_id| input.network.node(node_id).node_type != NodeType::End)
    {
        for node_id in &mut current_nodes {
            *node_id = destinations[node_id.0];
        }
        total_steps += sequence.len();
    }
    total_steps
}

example_tests! {
    "
    LLR

    AAA = (BBB, BBB)
    BBB = (AAA, ZZZ)
    ZZZ = (ZZZ, ZZZ)
    ",

    part1 => 6,
    part2 => 6,
    part2_brute_force => 6,
}

known_input_tests! {
    input: include_str!("../input/2023/day8.txt"),
    part1 => 20569,
    part2 => 21366921060721,
}
