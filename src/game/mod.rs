mod game;

pub use self::game::Game;

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

pub fn check_trajectory(trajectory: &str) -> Result<(), ServerError> {
    unimplemented!()
}

fn trajectory_of_string(t: &str) -> Result<Vec<(char, usize)>, ()> {
    let chars: Vec<char> = t.chars().collect();
    chars.chunks(2)
}

fn coordinates_of_chars(line: char, column: char) -> Result<(char, usize), ()> {
    let c = column.to_digit(10);
    if c.is_none() { return Err(()) }
    match line {
        'a' | 'A' | 'b' | 'B' | 'c' | 'C' | 'd' | 'D'  => (),
        _ => return Err(())
    };
    Ok((line, c.unwrap() as usize))
}

pub fn check_distances(t: &[(char, usize)]) -> bool {
    for i in 0..t.len() - 1 {
        if ! distance_is_valid(t[i], t[i + 1]) {
            return false
        }
    }
    true
}

fn distance_is_valid(square1: (char, usize), square2: (char, usize)) -> bool {
    let (v_dist, h_dist) = distance(square1, square2);
    v_dist + h_dist <= 2
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
        'a' | 'A' => 0,
        'b' | 'B' => 1,
        'c' | 'C' => 2,
        'd' | 'D' => 3,
        _ => panic!("Invalid character index !")
    }
}