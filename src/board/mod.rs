pub mod board;

pub use self::board::Board;

use super::{
    dict::{Dict, LocalDict},
    errors::{ServerError, ServerError::*},
};

use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    i8::*,
};

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

fn trajectory_of_string(t: &str) -> Result<Vec<(char, usize)>, ()> {
    if t.len() % 2 == 1 || t.len() < 6 { return Err(()) }
    let chars: Vec<char> = t.chars().collect();
    let trajectory = chars.chunks(2)
        .map(|chunk| coordinates_of_chars(chunk[0], chunk[1]))
        .collect::<Result<Vec<(char, usize)>, _>>()?;

    if is_valid_trajectory(&trajectory) { Ok(trajectory) } else { Err(()) }
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
    for i in 0..t.len() - 1 {
        if ! is_valid_distance(t[i], t[i + 1]) {
            return false
        }
    }
    true
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

fn index_of_coordinates(line: char, column: usize) -> Result<usize, ServerError> {
    if line < 'A' || line > 'D' || column < 1 || column > 4 {
        Err(InvalidCoordinates { line, column })
    } else {
        Ok((column - 1) + (4 * index_of_letter(line)))
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
    fn index_of_coordinates_ok() {
        assert_eq!(index_of_coordinates('C', 2).unwrap(), 9);
        assert_eq!(index_of_coordinates('A', 1).unwrap(), 0);
        assert_eq!(index_of_coordinates('D', 4).unwrap(), 15);
    }

    #[test]
    fn invalid_coordinates_returns_error() {
        match index_of_coordinates('E', 1) {
            Err(InvalidCoordinates {..}) => (),
            _ => panic!("This call sould return an error !")
        }

        match index_of_coordinates('E', 0) {
            Err(InvalidCoordinates {..}) => (),
            _ => panic!("This call sould return an error !")
        }

        match index_of_coordinates('E', 5) {
            Err(InvalidCoordinates {..}) => (),
            _ => panic!("This call sould return an error !")
        }
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
}