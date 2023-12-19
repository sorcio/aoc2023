use std::collections::HashMap;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::{example_tests, known_input_tests};

#[derive(Debug, Clone, Copy)]
enum Condition {
    Unconditional,
    GreaterThan(char, u32),
    LessThan(char, u32),
}

impl Condition {
    fn parse(s: &str) -> Self {
        let mut chars = s.chars();
        let variable = chars.next().unwrap();
        let operator = chars.next().unwrap();
        let value = s[2..].parse().unwrap();
        debug_assert!(variable == 'x' || variable == 'm' || variable == 'a' || variable == 's');
        match operator {
            '>' => Condition::GreaterThan(variable, value),
            '<' => Condition::LessThan(variable, value),
            _ => panic!("Invalid operator: {}", operator),
        }
    }

    fn check(&self, item: &Item) -> bool {
        match self {
            Condition::Unconditional => true,
            Condition::GreaterThan(variable, value) => item.get(*variable) > *value,
            Condition::LessThan(variable, value) => item.get(*variable) < *value,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Target {
    Accept,
    Reject,
    Workflow(u32),
}

#[derive(Debug, Clone)]
struct Rule {
    condition: Condition,
    target: Target,
}

#[derive(Debug, Clone)]

struct Workflow {
    #[allow(unused)]
    label: String,
    rules: Vec<Rule>,
}

impl Workflow {
    fn run(&self, item: &Item) -> Target {
        self.rules
            .iter()
            .find_map(|rule| rule.condition.check(item).then_some(rule.target))
            .unwrap_or_else(|| panic!("item {item:?} does not match any rule in {self:?}"))
    }
}

struct Program {
    workflows: Vec<Workflow>,
    entry_point: u32,
}

impl Program {
    fn accept_item(&self, item: &Item) -> bool {
        let mut current_workflow = self.entry_point;
        loop {
            let workflow = &self.workflows[current_workflow as usize];
            let target = workflow.run(item);
            match target {
                Target::Accept => return true,
                Target::Reject => return false,
                Target::Workflow(next_workflow) => current_workflow = next_workflow,
            }
        }
    }
}

fn compile_program<'a, I>(lines: I) -> Program
where
    I: Iterator<Item = &'a str>,
{
    let mut raw_workflows = Vec::new();
    let mut workflow_map = HashMap::new();
    let mut entry_point = None;
    for (i, line) in lines.enumerate() {
        let mut parts = line.split("{");
        let label = parts.next().unwrap();
        let index = i as u32;
        if label == "in" {
            entry_point = Some(index);
        }
        let rules = parts.next().and_then(|s| s.strip_suffix('}')).unwrap();
        // workflows.insert(label, rules);
        workflow_map.insert(label, index);
        raw_workflows.push((label, rules));
    }

    let workflows = raw_workflows
        .iter()
        .map(|(label, raw_rules)| {
            let rules = raw_rules
                .split(",")
                .map(|raw_rule| {
                    let mut parts = raw_rule.split(":");
                    let first_part = parts.next().unwrap();
                    let (condition, raw_target) = if let Some(target) = parts.next() {
                        (Condition::parse(first_part), target)
                    } else {
                        (Condition::Unconditional, first_part)
                    };
                    let target = match raw_target {
                        "A" => Target::Accept,
                        "R" => Target::Reject,
                        target_label => Target::Workflow(workflow_map[target_label]),
                    };
                    Rule { condition, target }
                })
                .collect();
            Workflow {
                label: label.to_string(),
                rules,
            }
        })
        .collect();

    Program {
        workflows,
        entry_point: entry_point.unwrap(),
    }
}

#[derive(Debug, Clone, Copy)]
struct Item {
    x: u32,
    m: u32,
    a: u32,
    s: u32,
}

impl Item {
    fn parse(s: &str) -> Self {
        let mut parts = s
            .strip_prefix('{')
            .unwrap()
            .strip_suffix('}')
            .unwrap()
            .split(',');
        let x = parts.next().unwrap()[2..].parse().unwrap();
        let m = parts.next().unwrap()[2..].parse().unwrap();
        let a = parts.next().unwrap()[2..].parse().unwrap();
        let s = parts.next().unwrap()[2..].parse().unwrap();
        Self { x, m, a, s }
    }

    fn get(&self, variable: char) -> u32 {
        match variable {
            'x' => self.x,
            'm' => self.m,
            'a' => self.a,
            's' => self.s,
            _ => panic!("Invalid variable: {}", variable),
        }
    }

    fn value(&self) -> u32 {
        self.x + self.m + self.a + self.s
    }
}

struct Input {
    program: Program,
    items: Vec<Item>,
}

#[aoc_generator(day19)]
fn parse(input: &str) -> Input {
    let mut lines = input.lines();
    let program = compile_program((&mut lines).take_while(|line| !line.is_empty()));
    let items = lines.map(Item::parse).collect();
    Input { program, items }
}

#[aoc(day19, part1)]
fn part1(input: &Input) -> u32 {
    input
        .items
        .iter()
        .filter_map(|item| input.program.accept_item(item).then(|| item.value()))
        .sum()
}

#[aoc(day19, part2)]
fn part2(_input: &Input) -> String {
    todo!()
}

#[cfg(test)]
mod tests {}

example_tests! {
    "
    px{a<2006:qkq,m>2090:A,rfg}
    pv{a>1716:R,A}
    lnx{m>1548:A,A}
    rfg{s<537:gd,x>2440:R,A}
    qs{s>3448:A,lnx}
    qkq{x<1416:A,crn}
    crn{x>2662:A,R}
    in{s<1351:px,qqz}
    qqz{s>2770:qs,m<1801:hdj,R}
    gd{a>3333:R,R}
    hdj{m>838:A,pv}

    {x=787,m=2655,a=1222,s=2876}
    {x=1679,m=44,a=2067,s=496}
    {x=2036,m=264,a=79,s=2244}
    {x=2461,m=1339,a=466,s=291}
    {x=2127,m=1623,a=2188,s=1013}
    ",
    part1 => 19114,
}

known_input_tests! {
    input: include_str!("../input/2023/day19.txt"),
    part1 => 456651,
}
