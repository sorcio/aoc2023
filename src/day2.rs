use aoc_runner_derive::{aoc, aoc_generator};

#[derive(Debug)]
struct Game {
    game_id: u32,
    sets: Vec<Colors>,
}

#[derive(Debug, Default)]
struct Colors {
    r: u32,
    g: u32,
    b: u32,
}

impl Colors {
    fn is_within_limit(&self, limit: &Colors) -> bool {
        self.r <= limit.r && self.g <= limit.g && self.b <= limit.b
    }
}

#[aoc_generator(day2)]
fn parse(input: &str) -> Vec<Game> {
    input
        .lines()
        .map(|line| {
            let (part1, part2) = line
                .trim() // trimming just to make it friendly for tests
                .split_once(':')
                .expect("should be a colon-separated line");
            let game_id: u32 = part1
                .strip_prefix("Game ")
                .expect("should start with 'Game '")
                .parse()
                .expect("Game id should be a number");
            let sets = part2
                .split(';')
                .map(|set| {
                    let color_strings = set.trim().split(',');
                    let mut colors = Colors::default();
                    for color_string in color_strings {
                        let (num_string, color) = color_string
                            .trim()
                            .split_once(' ')
                            .expect("color should be separated by a space");
                        let num = num_string.parse().expect("should be a number");
                        match color {
                            "red" => {
                                assert!(colors.r == 0);
                                colors.r = num;
                            }
                            "green" => {
                                assert!(colors.g == 0);
                                colors.g = num;
                            }
                            "blue" => {
                                assert!(colors.b == 0);
                                colors.b = num;
                            }
                            _ => panic!("expected only red|green|blue"),
                        }
                    }
                    colors
                })
                .collect();
            Game { game_id, sets }
        })
        .collect()
}

#[aoc(day2, part1)]
fn part1(input: &[Game]) -> u32 {
    let limit =
        Colors {
            r: 12,
            g: 13,
            b: 14,
        };
    input
        .iter()
        .map(|game| {
            if game.sets.iter().all(|set| set.is_within_limit(&limit)) {
                game.game_id
            } else {
                0
            }
        })
        .sum()
}

#[aoc(day2, part2)]
fn part2(input: &[Game]) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_example() {
        assert_eq!(
            part1(&parse(
                "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
        Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
        Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
        Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
        Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green"
            )),
            8
        );
    }

    // #[test]
    // fn part2_example() {
    //     assert_eq!(part2(&parse("<EXAMPLE>")), "<RESULT>");
    // }
}
