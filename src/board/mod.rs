pub mod board;

pub use self::board::Board;

use super::{
    errors::{ServerError},
};

use std::{
    collections::{HashMap, HashSet}
};

use regex::Regex;
use rand::{self, Rng};

lazy_static! {
    static ref GRID_REGEX: Regex = Regex::new("[a-zA-Z]{16}").unwrap();
}

const DICES: [[char; 6]; 16] = [
    ['E', 'T', 'U', 'K', 'N', 'O'],
    ['E', 'V', 'G', 'T', 'I', 'N'],
    ['D', 'E', 'C', 'A', 'M', 'P'],
    ['I', 'E', 'L', 'R', 'U', 'W'],
    ['E', 'H', 'I', 'F', 'S', 'E'],
    ['R', 'E', 'C', 'A', 'L', 'S'],
    ['E', 'N', 'T', 'D', 'O', 'S'],
    ['O', 'F', 'X', 'R', 'I', 'A'],
    ['N', 'A', 'V', 'E', 'D', 'Z'],
    ['E', 'I', 'O', 'A', 'T', 'A'],
    ['G', 'L', 'E', 'N', 'Y', 'U'],
    ['B', 'M', 'A', 'Q', 'J', 'O'],
    ['T', 'L', 'I', 'B', 'R', 'A'],
    ['S', 'P', 'U', 'L', 'T', 'E'],
    ['A', 'I', 'M', 'S', 'O', 'R'],
    ['E', 'N', 'H', 'R', 'I', 'S'],
];

fn generate_random_grid() -> [char; 16] {
    let mut rng = rand::thread_rng();
    let mut result = ['A'; 16];

    DICES.iter()
        .map(|dice| rng.choose(dice).unwrap())
        .enumerate()
        .for_each(|(idx, &letter)| result[idx] = letter);

    result
}

fn grid_of_string(s: &str) -> Option<[char; 16]> {
    if GRID_REGEX.is_match(s) {
        let mut result = ['A'; 16];
        s.chars().enumerate().for_each(|(idx, c)| result[idx] = c);
        Some(result)
    } else {
        None
    }
}

fn trajectory_of_string(t: &str) -> Result<Vec<(char, usize)>, ServerError> {
    if t.len() % 2 == 1 || t.len() < 6 {
        return Err(ServerError::bad_trajectory(t))
    }

    let chars: Vec<char> = t.chars().collect();
    let trajectory = chars.chunks(2)
        .map(|chunk| coordinates_of_chars(chunk[0], chunk[1]))
        .collect::<Result<Vec<(char, usize)>, _>>()
        .map_err(|_| ServerError::bad_trajectory(t))?;

    if is_valid_trajectory(&trajectory) {
        Ok(trajectory)
    } else {
        Err(ServerError::bad_trajectory(t))
    }
}

fn coordinates_of_chars(line: char, column: char) -> Result<(char, usize), ()> {
    let c = column_of_char(column)?;
    let l = line_of_char(line)?;
    Ok((l, c))
}

fn column_of_char(col: char) -> Result<usize, ()> {
    match col.to_digit(10) {
        Some(n @ 1...4) => Ok(n as usize),
        _ => Err(())
    }
}

fn line_of_char(line: char) -> Result<char, ()> {
    match line {
        'a' | 'A' => Ok('A'),
        'b' | 'B' => Ok('B'),
        'c' | 'C' => Ok('C'),
        'd' | 'D' => Ok('D'),
        _ => Err(())
    }
}

pub fn is_valid_trajectory(t: &[(char, usize)]) -> bool {
    if contains_doubles(t) {
        return false
    }
    for i in 0..t.len() - 1 {
        if ! is_valid_distance(t[i], t[i + 1]) {
            return false
        }
    }
    true
}

fn contains_doubles(trajectory: &[(char, usize)]) -> bool {
    let mut seen = HashSet::new();
    for c in trajectory {
        if seen.contains(c) {
            return true
        }
        seen.insert(c);
    }
    false
}

fn is_valid_distance(square1: (char, usize), square2: (char, usize)) -> bool {
    let (v_dist, h_dist) = distance(square1, square2);
    let s = v_dist + h_dist;
    s > 0 && s <= 2
}

fn distance(square1: (char, usize), square2: (char, usize)) -> (i8, i8) {
    ((square1.0 as i8 - square2.0 as i8).abs(),
     (square1.1 as i8 - square2.1 as i8).abs())
}

fn word_score(word: &str) -> u32 {
    match word.len() {
        0...2 => 0,
        3...4 => 1,
        5 => 2,
        6 => 3,
        7 => 5,
        _ => 11
    }
}

fn index_of_letter(letter: char) -> usize {
    match letter {
        'A' => 0,
        'B' => 1,
        'C' => 2,
        'D' => 3,
        _ => panic!("Invalid character index !")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn random_grid() {
       generate_random_grid().iter()
           .enumerate()
           .for_each(|(idx, c)| assert!(DICES[idx].contains(c)))
    }

    #[test]
    fn line_of_char_ok() {
        let lines = vec!['A', 'B', 'C', 'D'];
        for c in lines {
            match line_of_char(c) {
                Ok(r) if r == c => (),
                _ => panic!("{} is a valid line !", c)
            }
        }
    }

    #[test]
    fn line_or_char_err() {
        match line_of_char('E') {
            Err(()) => (),
            _ => panic!("'E' is not a valid line !")
        }
    }

    #[test]
    fn column_of_char_ok() {
        let cols = vec!['1', '2', '3', '4'];
        for c in cols {
            match column_of_char(c) {
                Ok(r) if r == c.to_digit(10).unwrap() as usize => (),
                _ => panic!("{} is a valid column !", c),
            }
        }
    }

    #[test]
    fn column_of_char_err() {
        match column_of_char('5') {
            Err(()) => (),
            _ => panic!("'5' is not a valid column !")
        }
    }

    #[test]
    fn distance_test() {
        assert_eq!(distance(('B', 2), ('A', 1)), (1, 1));
        assert_eq!(distance(('B', 2), ('A', 2)), (1, 0));
        assert_eq!(distance(('B', 2), ('B', 3)), (0, 1));
        assert_eq!(distance(('B', 2), ('B', 2)), (0, 0));
    }

    #[test]
    fn is_valid_distance_test() {
        assert!(is_valid_distance(('B', 2), ('A', 1)));
        assert!(is_valid_distance(('B', 2), ('B', 1)));
        assert!(! is_valid_distance(('B', 2), ('B', 2)));
    }

    #[test]
    fn trajectory_of_string_ok() {
        let expected = vec![('C', 2), ('B', 1), ('A', 2), ('A', 3), ('B', 2), ('C', 3), ('D', 2)];
        let trajectory = "C2B1A2A3B2C3D2";
        match trajectory_of_string(trajectory) {
            Ok(ref t) if t == &expected => (),
            _ => panic!("Trajectory {} is invalid !", trajectory)
        }
    }

    #[test]
    fn contains_duplicates() {
        let trajectory = vec![('A', 2), ('B', 1), ('A', 2)];
        assert!(contains_doubles(&trajectory))
    }
}