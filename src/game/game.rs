use super::*;

use rand::{self, Rng};

pub struct Game {
    grid: [char; 16],
    player_words: HashMap<String, Vec<String>>,
    played: HashSet<String>,
    turn: u64,
    dict: Box<Dict>,
}

impl Game {
    pub fn new() -> Game {
        Game::with_dict(Box::new(LocalDict::new()))
    }

    pub fn with_dict(dict: Box<Dict>) -> Game {
        Game {
            grid: Game::generate_grid(),
            player_words: HashMap::new(),
            played: HashSet::new(),
            turn: 0,
            dict
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

    pub fn submit_trajectory(&mut self, user: &str, t: &str) -> Result<(), ServerError> {
        let trajectory = trajectory_of_string(t)
            .map_err(|_| ServerError::bad_trajectory(t.to_string()))?;

        let word = self.word_of_trajectory(&trajectory);
        if self.played.contains(&word) {
            return Err(ServerError::already_played(word));
        }

        self.player_words.entry(user.to_string()).or_insert(vec![]).push(word.clone());
        self.played.insert(word);
        Ok(())
    }

    fn word_of_trajectory(&self, trajectory: &[(char, usize)]) -> String {
        trajectory.iter()
            .map(|&(line, col)| index_of_coordinates(line, col).unwrap())
            .map(|idx| self.grid[idx])
            .collect()
    }

    pub fn new_turn(&mut self) {
        self.grid = Game::generate_grid();
        self.turn += 1;
    }

    pub fn update_users(&mut self, users: HashSet<&str>) {
        self.player_words.retain(|key, _| users.contains(&key.as_str()));
        self.played = self.compute_played()
    }

    fn compute_played(&self) -> HashSet<String> {
        HashSet::from_iter(
            self.player_words.values()
                .flat_map(|v| v)
                .map(|v| v.to_string())
        )
    }

    pub fn users_scores(&self, users: &[String]) -> Vec<(String, u32)> {
        users.iter()
            .map(|u| (u.to_owned(), self.user_score(u)))
            .collect()
    }

    fn user_score(&self, user: &str) -> u32 {
        self.player_words.get(user).map_or(0, |words| {
            words.iter()
                .map(|w| word_score(w))
                .sum()
        })
    }

    fn letter_at(&self, line: char, column: usize) -> Result<char, ServerError> {
        let idx = index_of_coordinates(line, column)?;
        Ok(self.grid[idx])
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
    fn update_users() {
        let mut game = Game::new();
        game.player_words.insert("user1".to_string(), vec!["word1".to_string()]);
        game.player_words.insert("user2".to_string(), vec!["word2".to_string()]);
        game.player_words.insert("user3".to_string(), vec!["word3".to_string()]);

        let users = vec!["user1", "user3"];
        game.update_users(HashSet::from_iter(users));

        let expected: HashSet<&str> = HashSet::from_iter(vec!["user1", "user3"]);
        let actual: HashSet<&str> = game.player_words.keys()
            .map(|p| p.as_str())
            .collect();

        assert_eq!(expected, actual);
    }

    #[test]
    fn word_of_trajectory() {
        let game = create_test_game();
        let trajectory = vec![('C', 2), ('B', 1), ('A', 2), ('A', 3), ('B', 2), ('C', 3), ('D', 2)];
        assert_eq!(game.word_of_trajectory(&trajectory), "TRIDENT");
    }

    #[test]
    fn submit_word() {
        let mut game = create_test_game();
        game.submit_trajectory("user1", "C2B1A2A3B2C3D2");
        let expected = vec!["TRIDENT".to_string()];
        assert_eq!(game.player_words.get("user1").unwrap(), &expected);
    }

    #[test]
    fn already_played_word_is_refused() {
        let mut game = create_test_game();
        game.submit_trajectory("user1", "C2B1A2A3B2C3D2");
        match game.submit_trajectory("user2", "C2B1A2A3B2C3D2") {
            Err(ServerError::AlreadyPlayed {..}) => (),
            _ => panic!("{} has already been played !")
        }
    }

    #[test]
    fn submit_word_adds_word_to_played() {
        let mut game = create_test_game();
        game.submit_trajectory("user1", "C2B1A2A3B2C3D2");
        assert!(game.played.contains("TRIDENT"));
    }

    #[test]
    fn update_users_updates_played() {
        let mut game = create_test_game();
        game.submit_trajectory("user1", "C2B1A2A3B2C3D2");
        game.submit_trajectory("user2", "A2A1B2");
        game.update_users(HashSet::from_iter(vec!["user2"]));
        assert_eq!(game.played, HashSet::from_iter(vec!["ILE".to_string()]));
    }

    fn create_test_game() -> Game {
        let mut game = Game::new();
        game.grid = ['L', 'I', 'D', 'A',
                     'R', 'E', 'J', 'U',
                     'L', 'T', 'N', 'E',
                     'A', 'T', 'N', 'G' ];
        game
    }
}
