use std::cmp::Ordering;
use std::str::FromStr;

use aoc_runner_derive::{aoc, aoc_generator};

use crate::testing::{example_tests, known_input_tests};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Card(u8);

impl From<char> for Card {
    fn from(c: char) -> Self {
        match c {
            '2'..='9' => Card(c.to_digit(10).unwrap() as u8),
            'T' => Card(10),
            'J' => Card(11),
            'Q' => Card(12),
            'K' => Card(13),
            'A' => Card(14),
            _ => panic!("card should be one of 2..9, T, J, Q, K, A; not {c}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CardWithJoker(u8);

impl From<Card> for CardWithJoker {
    fn from(card: Card) -> Self {
        match card.0 {
            11 => CardWithJoker(0),
            n => CardWithJoker(n),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Hand {
    cards: [Card; 5],
}

impl FromIterator<char> for Hand {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let cards = iter
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<_>>()
            .try_into()
            .expect("should have 5 cards");
        Hand { cards }
    }
}

impl FromStr for Hand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.chars().collect())
    }
}

impl Hand {
    fn strength(&self) -> HandStrength {
        // count cards of a kind
        let mut kinds = [0u8; 15];
        for card in self.cards {
            kinds[card.0 as usize] += 1;
        }
        kinds.sort_unstable();
        match kinds {
            [.., 5] => HandStrength::FiveOfAKind,
            [.., 4] => HandStrength::FourOfAKind,
            [.., 2, 3] => HandStrength::FullHouse,
            [.., 3] => HandStrength::ThreeOfAKind,
            [.., 2, 2] => HandStrength::TwoPair,
            [.., 2] => HandStrength::Pair,
            [.., 1] => HandStrength::HighCard,
            _ => unreachable!(),
        }
    }

    fn with_joker(self) -> HandWithJoker {
        self.into()
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.strength().cmp(&other.strength()) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self
                .cards
                .iter()
                .zip(&other.cards)
                .map(|(sc, oc)| sc.cmp(oc))
                .find(|&ord| ord != Ordering::Equal)
                .unwrap_or(Ordering::Equal),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HandStrength {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HandWithJoker {
    cards: [CardWithJoker; 5],
}

impl From<Hand> for HandWithJoker {
    fn from(value: Hand) -> Self {
        let cards = value.cards.map(|card| card.into());
        HandWithJoker { cards }
    }
}

impl HandWithJoker {
    fn strength(&self) -> HandStrength {
        // count cards of a kind
        let mut kinds = [0u8; 15];
        for card in self.cards {
            kinds[card.0 as usize] += 1;
        }
        let jokers = kinds[0];
        kinds[0] = 0;
        kinds.sort_unstable();
        *kinds.last_mut().unwrap() += jokers;
        match kinds {
            [.., 5] => HandStrength::FiveOfAKind,
            [.., 4] => HandStrength::FourOfAKind,
            [.., 2, 3] => HandStrength::FullHouse,
            [.., 3] => HandStrength::ThreeOfAKind,
            [.., 2, 2] => HandStrength::TwoPair,
            [.., 2] => HandStrength::Pair,
            [.., 1] => HandStrength::HighCard,
            _ => unreachable!(),
        }
    }
}

impl PartialOrd for HandWithJoker {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HandWithJoker {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.strength().cmp(&other.strength()) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self
                .cards
                .iter()
                .zip(&other.cards)
                .map(|(sc, oc)| sc.cmp(oc))
                .find(|&ord| ord != Ordering::Equal)
                .unwrap_or(Ordering::Equal),
        }
    }
}

#[derive(Debug, Clone)]
struct Bid {
    hand: Hand,
    bid_value: u32,
}

#[aoc_generator(day7)]
fn parse(input: &str) -> Vec<Bid> {
    input
        .lines()
        .map(|line| {
            let hand = line[..5].parse().unwrap();
            let bid_value = line[6..].parse().unwrap();
            Bid { hand, bid_value }
        })
        .collect()
}

fn part1_impl(input: &[Bid]) -> u32 {
    let mut bids = input.to_vec();
    bids.sort_by(|a, b| a.hand.cmp(&b.hand));
    (1..1 + bids.len() as u32)
        .zip(&bids)
        .map(|(rank, bid)| rank * bid.bid_value)
        .sum()
}

fn part2_impl(input: &[Bid]) -> u32 {
    let mut bids: Vec<_> = input
        .into_iter()
        .map(|bid| (bid.clone().hand.with_joker(), bid.bid_value))
        .collect();
    bids.sort_by(|a, b| a.0.cmp(&b.0));
    (1..1 + bids.len() as u32)
        .zip(&bids)
        .map(|(rank, bid)| rank * bid.1)
        .sum()
}

#[aoc(day7, part1)]
fn part1(input: &[Bid]) -> u32 {
    part1_impl(input)
}

#[aoc(day7, part2)]
fn part2(input: &[Bid]) -> u32 {
    part2_impl(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand() {
        assert_eq!(
            "32T3K".parse(),
            Ok(Hand {
                cards: [Card(3), Card(2), Card(10), Card(3), Card(13)]
            })
        );
    }

    #[test]
    fn hand_strength() {
        let strength = |s: &str| s.parse::<Hand>().unwrap().strength();
        assert_eq!(strength("A2345"), HandStrength::HighCard);
        assert_eq!(strength("AA234"), HandStrength::Pair);
        assert_eq!(strength("AA233"), HandStrength::TwoPair);
        assert_eq!(strength("AAA24"), HandStrength::ThreeOfAKind);
        assert_eq!(strength("AAA22"), HandStrength::FullHouse);
        assert_eq!(strength("AAAA2"), HandStrength::FourOfAKind);
        assert_eq!(strength("AAAAA"), HandStrength::FiveOfAKind);
    }

    #[test]
    fn hand_cmp() {
        let hand = |s: &str| s.parse::<Hand>().unwrap();
        assert!(hand("A2345") < hand("AA234"));
        assert!(hand("A2345") > hand("23456"));
        // ...
    }

    #[test]
    fn hand_with_joker_strength() {
        let strength = |s: &str| s.parse::<Hand>().unwrap().with_joker().strength();
        // with joker:
        assert_eq!(strength("AAAAJ"), HandStrength::FiveOfAKind);
        assert_eq!(strength("AAAJJ"), HandStrength::FiveOfAKind);
        assert_eq!(strength("QJJQ2"), HandStrength::FourOfAKind);
        assert_eq!(strength("QJJQ2"), HandStrength::FourOfAKind);

        // WITHOUT joker:
        assert_eq!(strength("A2345"), HandStrength::HighCard);
        assert_eq!(strength("AA234"), HandStrength::Pair);
        assert_eq!(strength("AA233"), HandStrength::TwoPair);
        assert_eq!(strength("AAA24"), HandStrength::ThreeOfAKind);
        assert_eq!(strength("AAA22"), HandStrength::FullHouse);
        assert_eq!(strength("AAAA2"), HandStrength::FourOfAKind);
        assert_eq!(strength("AAAAA"), HandStrength::FiveOfAKind);
    }
}

example_tests! {
    "
    32T3K 765
    T55J5 684
    KK677 28
    KTJJT 220
    QQQJA 483
    ",

    part1 => 6440,
    part2 => 5905,
}

known_input_tests! {
    input: include_str!("../input/2023/day7.txt"),
    part1 => 248179786,
    part2 => 247885995,
}
