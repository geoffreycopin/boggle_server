use super::{
    errors::{ServerError, ServerError::*},
};

use std::{
    collections::HashMap,
};

use rand::{self, Rng};

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

pub struct Game {
    grid: [char; 16],
    played_words: HashMap<String, Vec<String>>
}

impl Game {
    pub fn new() -> Game {
        Game {
            grid: Game::generate_grid(),
            played_words: HashMap::new(),
        }
    }

    fn generate_grid() -> [char; 16] {
        let mut rng = rand::thread_rng();
        let mut result = ['A'; 16];

        DICES.iter()
            .map(|dice| rng.choose(dice).unwrap())
            .enumerate()
            .for_each(|(idx, &letter)| result[idx] = letter);

        result
    }

    fn letter_at(&self, line: char, column: usize) -> Result<char, ServerError> {
        let idx = Game::index_of_coordinates(line, column)?;
        Ok(self.grid[idx])
    }

    fn index_of_coordinates(line: char, column: usize) -> Result<usize, ServerError> {
        if line < 'A' || line > 'D' || column < 1 || column > 4 {
            Err(InvalidCoordinates { line, column })
        } else {
            Ok((column - 1) + (4 * Game::index_of_letter(line)))
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
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let game = Game::new();
        game.grid.iter().enumerate()
            .for_each(|(idx, c)| assert!(DICES[idx].contains(c)));
    }

    #[test]
    fn index_of_coordinates() {
        assert_eq!(Game::index_of_coordinates('C', 2).unwrap(), 9);
        assert_eq!(Game::index_of_coordinates('A', 1).unwrap(), 0);
        assert_eq!(Game::index_of_coordinates('D', 4).unwrap(), 15);
    }

    #[test]
    fn invalid_coordinates_returns_error() {
        match Game::index_of_coordinates('E', 1) {
            Err(InvalidCoordinates {..}) => (),
            _ => panic!("This call sould return an error !")
        }

        match Game::index_of_coordinates('E', 0) {
            Err(InvalidCoordinates {..}) => (),
            _ => panic!("This call sould return an error !")
        }

        match Game::index_of_coordinates('E', 5) {
            Err(InvalidCoordinates {..}) => (),
            _ => panic!("This call sould return an error !")
        }
    }
}

