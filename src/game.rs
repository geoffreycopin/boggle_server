use super::{
    errors::{ServerError, ServerError::*},
};

use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
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
    played_words: HashMap<String, Vec<String>>,
    turn: u64,
}

impl Game {
    pub fn new() -> Game {
        Game {
            grid: Game::generate_grid(),
            played_words: HashMap::new(),
            turn: 0,
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

    pub fn grid_str(&self) -> String {
        String::from_iter(self.grid.iter())
    }

    pub fn new_turn(&mut self) {
        self.grid = Game::generate_grid();
        self.turn += 1;
    }

    pub fn update_users(&mut self, users: HashSet<&str>) {
        for players in self.played_words.values_mut() {
            players.retain(|p| users.contains(&p.as_str()))
        }
    }

    pub fn users_scores(&self, users: &[String]) -> Vec<(String, u32)> {
        users.iter()
            .map(|u| (u.to_owned(), self.user_score(u)))
            .collect()
    }

    fn user_score(&self, user: &str) -> u32 {
        self.words_played_by(user).iter()
            .map(|w| Game::word_score(w))
            .sum()
    }

    fn words_played_by(&self, user: &str) -> Vec<&str> {
        self.played_words.keys()
            .filter(|&word| self.played_words[word].contains(&user.to_string()))
            .map(|u| u.as_str())
            .collect()
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
            'a' | 'A' => 0,
            'b' | 'B' => 1,
            'c' | 'C' => 2,
            'd' | 'D' => 3,
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

    #[test]
    fn update_users() {
        let mut game = Game::new();
        game.played_words.insert("word".to_string(),
                                 vec!["user1".to_string(), "user2".to_string()]);
        game.played_words.insert("word2".to_string(), vec!["user3".to_string()]);

        let users = vec!["user1", "user3"];
        game.update_users(HashSet::from_iter(users));

        let expected: HashSet<&str> = HashSet::from_iter(vec!["user1", "user3"]);
        let actual: HashSet<&str> = game.played_words.values()
            .flat_map(|p| p)
            .map(|p| p.as_str())
            .collect();

        assert_eq!(expected, actual);
    }
}

