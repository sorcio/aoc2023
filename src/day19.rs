use std::collections::HashMap;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::{example_tests, known_input_tests};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn invert(&self) -> Self {
        // x > 10 becomes x <= 10 which is equivalent to x < 11
        match self {
            Condition::Unconditional => Condition::Unconditional,
            Condition::GreaterThan(variable, value) => Condition::LessThan(*variable, *value + 1),
            Condition::LessThan(variable, value) => Condition::GreaterThan(*variable, *value - 1),
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
        let mut parts = line.split('{');
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
                .split(',')
                .map(|raw_rule| {
                    let mut parts = raw_rule.split(':');
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

    fn set(&mut self, variable: char, value: u32) {
        match variable {
            'x' => self.x = value,
            'm' => self.m = value,
            'a' => self.a = value,
            's' => self.s = value,
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
        .filter(|&item| input.program.accept_item(item))
        .map(|item| item.value())
        .sum()
}

#[derive(Debug, Clone)]
struct Bounds {
    lower: Item,
    upper: Item,
}

impl Bounds {
    fn new(lower: u32, upper: u32) -> Self {
        Self {
            lower: Item {
                x: lower,
                m: lower,
                a: lower,
                s: lower,
            },
            upper: Item {
                x: upper,
                m: upper,
                a: upper,
                s: upper,
            },
        }
    }

    fn update_lower_bound(&mut self, variable: char, value: u32) {
        let current = self.lower.get(variable);
        if value > current {
            self.lower.set(variable, value);
        }
    }

    fn update_upper_bound(&mut self, variable: char, value: u32) {
        let current = self.upper.get(variable);
        if value < current {
            self.upper.set(variable, value);
        }
    }

    fn update(&mut self, condition: Condition) {
        match condition {
            Condition::Unconditional => {}
            Condition::GreaterThan(variable, value) => {
                // a condition like x > 10 means that the lower bound for x is 11
                self.update_lower_bound(variable, value + 1);
            }
            Condition::LessThan(variable, value) => {
                // a condition like x < 10 means that the upper bound for x is 9
                self.update_upper_bound(variable, value - 1);
            }
        }
    }
}

impl std::fmt::Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{x=[{},{}],m=[{},{}],a=[{},{}],s=[{},{}]}}",
            self.lower.x,
            self.upper.x,
            self.lower.m,
            self.upper.m,
            self.lower.a,
            self.upper.a,
            self.lower.s,
            self.upper.s,
        )
    }
}

struct IteratePathsToAcceptance<'a> {
    program: &'a Program,
    stack: Vec<(u32, usize, Bounds)>,
}

impl<'a> IteratePathsToAcceptance<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            program,
            stack: vec![(program.entry_point, 0, Bounds::new(1, 4000))],
        }
    }
}

impl<'a> Iterator for IteratePathsToAcceptance<'a> {
    type Item = Bounds;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (_workflow_index, workflow, rule_index, bounds) = loop {
                let &(workflow_index, rule_index, ref bounds) = self.stack.last()?;
                let workflow = &self.program.workflows[workflow_index as usize];
                if rule_index == workflow.rules.len() {
                    self.stack.pop();
                } else {
                    break (workflow_index, workflow, rule_index, bounds.clone());
                }
            };
            self.stack.last_mut().unwrap().1 = rule_index + 1;
            let rule = &workflow.rules[rule_index];

            if rule.condition != Condition::Unconditional {
                self.stack
                    .last_mut()
                    .unwrap()
                    .2
                    .update(rule.condition.invert());
            }

            let push_next = match rule.target {
                Target::Accept => {
                    let mut bounds = bounds.clone();
                    bounds.update(rule.condition);
                    return Some(bounds);
                }
                Target::Reject => None,
                Target::Workflow(next_workflow) => {
                    let mut bounds = bounds.clone();
                    bounds.update(rule.condition);
                    Some((next_workflow, bounds))
                }
            };

            if let Some((next_workflow, bounds)) = push_next {
                self.stack.push((next_workflow, 0, bounds.clone()));
            }
        }
    }
}

fn find_paths_to_acceptance(
    program: &Program,
    workflow_index: u32,
    paths: &mut Vec<Vec<Condition>>,
    partial: Vec<Condition>,
) {
    let workflow = &program.workflows[workflow_index as usize];

    let mut local_partial = partial.clone();

    for rule in &workflow.rules {
        match rule.target {
            Target::Accept => {
                let mut path = local_partial.clone();
                path.push(rule.condition);
                paths.push(path);
            }
            Target::Reject => {}
            Target::Workflow(next_workflow) => {
                let mut path = local_partial.clone();
                path.push(rule.condition);
                find_paths_to_acceptance(program, next_workflow, paths, path);
            }
        }

        if rule.condition != Condition::Unconditional {
            local_partial.push(rule.condition.invert());
        }
    }
}

#[aoc(day19, part2)]
fn part2(input: &Input) -> u64 {
    let program = &input.program;
    let mut paths = Vec::new();
    find_paths_to_acceptance(program, program.entry_point, &mut paths, vec![]);

    let mut total = 0;
    for path in paths {
        let mut bounds = Bounds::new(1, 4000);

        for condition in path {
            bounds.update(condition);
        }
        let x_diff = bounds.upper.x - bounds.lower.x + 1;
        let m_diff = bounds.upper.m - bounds.lower.m + 1;
        let a_diff = bounds.upper.a - bounds.lower.a + 1;
        let s_diff = bounds.upper.s - bounds.lower.s + 1;
        let count = x_diff as u64 * m_diff as u64 * a_diff as u64 * s_diff as u64;
        total += count;
    }
    total
}

#[aoc(day19, part2, iterator)]
fn part2_iterator(input: &Input) -> u64 {
    let program = &input.program;
    let mut total = 0;
    for bounds in IteratePathsToAcceptance::new(program) {
        let x_diff = bounds.upper.x - bounds.lower.x + 1;
        let m_diff = bounds.upper.m - bounds.lower.m + 1;
        let a_diff = bounds.upper.a - bounds.lower.a + 1;
        let s_diff = bounds.upper.s - bounds.lower.s + 1;
        let count = x_diff as u64 * m_diff as u64 * a_diff as u64 * s_diff as u64;
        total += count;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part2() {
        let input = parse(&unindent::unindent(
            "
                in{a>1:A,R}

                {x=0,m=0,a=0,s=0}
                ",
        ));
        let expected = 3999 * 4000 * 4000 * 4000; // 255936000000000
        assert_eq!(part2(&input), expected);
        assert_eq!(part2_iterator(&input), expected);
    }

    #[test]
    fn test_part2_iter() {
        let input = parse(&unindent::unindent(
            "
                in{a>2000:wf1,x>1:A}
                wf1{m>1:A}

                {x=0,m=0,a=0,s=0}
                ",
        ));
        let expected = [
            // a > 2000, m > 1
            3999 * 2000 * 4000 * 4000,
            // a <= 2000, x > 1
            3999 * 2000 * 4000 * 4000,
        ]
        .iter()
        .sum();
        // let expected = 3999 * 4000 * 4000 * 4000 + 3999 * 2001 * 4000; // 255_999_984_000_000
        assert_eq!(part2(&input), expected);
        assert_eq!(part2_iterator(&input), expected);
    }
}

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
    part2 => 167409079868000,
    part2_iterator => 167409079868000,
}

known_input_tests! {
    input: include_str!("../input/2023/day19.txt"),
    part1 => 456651,
    part2 => 131899818301477,
    part2_iterator => 131899818301477,
}
