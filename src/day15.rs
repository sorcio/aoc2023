use core::panic;
use std::ffi::CStr;

use aoc_runner_derive::aoc;

use crate::testing::{example_tests, known_input_tests};
use crate::utils::AsciiUtils;

/// Hash algorithm for day 15.
fn hash_d15<I: IntoIterator<Item = u8>>(iter: I) -> u8 {
    iter.into_iter()
        .fold(0u8, |acc, b| b.wrapping_add(acc).wrapping_mul(17))
}

trait HashableD15 {
    fn hashed_d15(&self) -> u8;
}

impl HashableD15 for &[u8] {
    fn hashed_d15(&self) -> u8 {
        hash_d15(self.iter().copied())
    }
}

impl<const N: usize> HashableD15 for &[u8; N] {
    fn hashed_d15(&self) -> u8 {
        hash_d15(self.iter().copied())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Label([u8; 8]);

impl core::fmt::Debug for Label {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let str = CStr::from_bytes_until_nul(&self.0)
            .unwrap()
            .to_str()
            .unwrap();

        f.pad(str)
    }
}

impl From<[u8; 8]> for Label {
    fn from(value: [u8; 8]) -> Self {
        Self(value)
    }
}

impl HashableD15 for Label {
    fn hashed_d15(&self) -> u8 {
        hash_d15(self.0.iter().copied().take_while(|&b| b > 0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Minus,
    Eq(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Step {
    label: Label,
    box_: usize,
    op: Op,
}

impl Step {
    fn from_ascii(s: &[u8]) -> Step {
        let mut label = [0; 8];
        let mut hash: u8 = 0;
        let mut op = Op::Minus;
        for (i, &byte) in s.iter().enumerate() {
            match byte {
                b'a'..=b'z' => {
                    label[i] = byte;
                    hash = hash.wrapping_add(byte).wrapping_mul(17);
                }
                b'-' => {
                    op = Op::Minus;
                    break;
                }
                b'=' => {
                    op = Op::Eq(s[i + 1] - b'0');
                    break;
                }
                _ => panic!("Invalid byte in step: {}", byte),
            }
        }
        let label: Label = label.into();
        let box_ = label.hashed_d15() as _;
        Step { label, box_, op }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Lens {
    label: Label,
    focal_length: u8,
}

#[derive(Debug)]
struct Registry {
    boxes: [LensesBox; 256],
}

#[repr(align(32))]
#[derive(Debug, Default)]
struct LensesBox {
    lenses: Vec<Lens>,
}

impl IntoIterator for LensesBox {
    type Item = Lens;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.lenses.into_iter()
    }
}

impl LensesBox {
    fn remove(&mut self, label: Label) {
        self.lenses.retain(|lens| lens.label != label);
    }

    fn add(&mut self, label: Label, focal_length: u8) {
        if let Some(i) = self.lenses.iter().position(|lens| lens.label == label) {
            self.lenses[i].focal_length = focal_length;
        } else {
            self.lenses.push(Lens {
                label,
                focal_length,
            });
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        let boxes = if cfg!(any(miri, nfeature = "no_dark_magic")) {
            // A safe version for Miri, because we already know that the version below
            // is unsound.
            [(); 256].map(|_| LensesBox::default())
        } else {
            // SAFETY: nope, this is just for fun. This is unsound af, for
            // multiple reasons. It depends on the internal representation of
            // Vec, and on the assumption that the pointer never gets
            // dereferenced when capacity=0. Miri and rust-analyzer get very
            // angry, and rightly so, because we are violating the non-null
            // pointer invariant. And I don't even want to think about platforms
            // where 0 is not the null pointer. But tests pass and this gets us
            // a negligible, absolutely not worth it, speedup. Let me just have
            // fun, ok?
            unsafe {
                #[allow(invalid_value)]
                core::mem::zeroed()
            }
        };
        Self { boxes }
    }
}

impl Registry {
    fn apply_step(&mut self, step: Step) {
        let box_ = &mut self.boxes[step.box_];
        match step.op {
            Op::Minus => {
                box_.remove(step.label);
            }
            Op::Eq(focal_length) => {
                box_.add(step.label, focal_length);
            }
        };
    }

    fn into_boxes(self) -> impl Iterator<Item = LensesBox> {
        self.boxes.into_iter()
    }
}

fn parse_steps(input: &[u8]) -> impl Iterator<Item = Step> + '_ {
    input
        .ascii_trim_end()
        .split(|&b| b == b',')
        .map(Step::from_ascii)
}

#[aoc(day15, part1)]
fn part1(input: &[u8]) -> u64 {
    input
        .ascii_trim_end()
        .split(|&b| b == b',')
        .map(|step| step.hashed_d15() as u64)
        .sum()
}

#[aoc(day15, part2)]
fn part2(input: &[u8]) -> u64 {
    let mut reg = Registry::default();
    for step in parse_steps(input) {
        reg.apply_step(step);
    }
    reg.into_boxes()
        .enumerate()
        .map(|(i, box_)| {
            box_.into_iter()
                .enumerate()
                .map(|(n, lens)| lens.focal_length as u64 * (n as u64 + 1) * (i as u64 + 1))
                .sum::<u64>()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_examples() {
        assert_eq!(b"rn=1".hashed_d15(), 30);
        assert_eq!(b"cm-".hashed_d15(), 253);
        assert_eq!(b"qp=3".hashed_d15(), 97);
        assert_eq!(b"cm=2".hashed_d15(), 47);
        assert_eq!(b"qp-".hashed_d15(), 14);
        assert_eq!(b"pc=4".hashed_d15(), 180);
        assert_eq!(b"ot=9".hashed_d15(), 9);
        assert_eq!(b"ab=5".hashed_d15(), 197);
        assert_eq!(b"pc-".hashed_d15(), 48);
        assert_eq!(b"pc=6".hashed_d15(), 214);
        assert_eq!(b"ot=7".hashed_d15(), 231);
    }

    #[test]
    fn hash_example_labels() {
        assert_eq!(b"rn".hashed_d15(), 0);
        assert_eq!(b"cm".hashed_d15(), 0);
        assert_eq!(b"qp".hashed_d15(), 1);
        assert_eq!(b"pc".hashed_d15(), 3);
        assert_eq!(b"ot".hashed_d15(), 3);
        assert_eq!(b"ab".hashed_d15(), 3);
    }
}

example_tests! {
    parser: None,

    &b"rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7"[..],
    part1 => 1320,
    part2 => 145,
}

known_input_tests! {
    parser: None,
    input: &include_bytes!("../input/2023/day15.txt")[..],

    part1 => 507291,
    part2 => 296921,
}
