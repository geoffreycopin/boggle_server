use super::*;

use rand::{self, Rng};

use std::io::Write;

pub struct Board {
    grid: [char; 16],
    player_words: HashMap<String, Vec<String>>,
    played: HashSet<String>,
    turn: u64
}

impl Board {
    pub fn new() -> Board {
        Board {
            grid: Board::generate_grid(),
            player_words: HashMap::new(),
            played: HashSet::new(),
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

    pub fn reset(&mut self) {
        self.grid = Board::generate_grid();
        self.player_words.clear();
        self.played.clear();
        self.turn = 1;
    }

    pub fn welcome_str(&self, users: &[String]) -> String {
        let grid = String::from_iter(self.grid.iter());
        let scores = self.scores_str(users);
        format!("BIENVENUE/{}/{}*{}/\n", grid, self.turn, scores)
    }

    pub fn grid_str(&self) -> String {
        String::from_iter(self.grid.iter())
    }

    pub fn scores_str(&self, users: &[String]) -> String  {
        users.iter()
            .map(|u| format!("{}*{}", u, self.user_score(u)))
            .collect::<Vec<String>>()
            .join("*")
    }

    pub fn is_already_played(&self, word: &str) -> bool {
        self.played.contains(word)
    }

    pub fn submit_word(&mut self, user: &str, word: &str, trajectory: &str)
        -> Result<(), ServerError>
    {
        let t = trajectory_of_string(trajectory)
            .map_err(|_| ServerError::bad_trajectory(trajectory))?;

        if self.word_of_trajectory(&t) != word {
            return Err(ServerError::no_match(trajectory, word))
        }

        if self.played.contains(word) {
            return Err(ServerError::already_played(word))
        }

        self.player_words.entry(user.to_string()).or_insert(vec![])
            .push(word.to_string());
        self.played.insert(word.to_string());

        Ok(())
    }

    fn word_of_trajectory(&self, trajectory: &[(char, usize)]) -> String {
        trajectory.iter() .map(|&(line, col)| {
                let idx = (4 * index_of_letter(line)) + (col - 1);
                self.grid[idx]
            })
            .collect::<String>().to_lowercase()
    }

    pub fn new_turn(&mut self) {
        self.grid = Board::generate_grid();
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
pub mod test {
    use super::*;

    #[test]
    fn new() {
        let board = Board::new();
        board.grid.iter().enumerate()
            .for_each(|(idx, c)| assert!(DICES[idx].contains(c)));
    }

    #[test]
    fn update_users() {
        let mut board = Board::new();
        board.player_words.insert("user1".to_string(), vec!["word1".to_string()]);
        board.player_words.insert("user2".to_string(), vec!["word2".to_string()]);
        board.player_words.insert("user3".to_string(), vec!["word3".to_string()]);

        let users = vec!["user1", "user3"];
        board.update_users(HashSet::from_iter(users));

        let expected: HashSet<&str> = HashSet::from_iter(vec!["user1", "user3"]);
        let actual: HashSet<&str> = board.player_words.keys()
            .map(|p| p.as_str())
            .collect();

        assert_eq!(expected, actual);
    }

    #[test]
    fn word_of_trajectory() {
        let board = create_test_board();
        let trajectory = vec![('C', 2), ('B', 1), ('A', 2), ('A', 3), ('B', 2), ('C', 3), ('D', 2)];
        assert_eq!(board.word_of_trajectory(&trajectory), "trident");
    }

    #[test]
    fn submit_word() {
        let mut board = create_test_board();
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2");
        let expected = vec!["trident".to_string()];
        assert_eq!(board.player_words.get("user1").unwrap(), &expected);
    }

    #[test]
    fn submit_word_adds_word_to_played() {
        let mut board = create_test_board();
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2");
        assert!(board.played.contains("trident"));
    }

    #[test]
    fn submit_words_error_no_match() {
        let mut board = create_test_board();
        match board.submit_word("user1", "ile", "C2B1A2A3B2C3D2") {
            Err(ServerError::NoMatch {..}) => (),
            _ => panic!("This call should return an error !")
        }
    }

    #[test]
    fn update_users_updates_played() {
        let mut board = create_test_board();
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2");
        board.submit_word("user2", "ile", "A2A1B2");
        board.update_users(HashSet::from_iter(vec!["user2"]));
        assert_eq!(board.played, HashSet::from_iter(vec!["ile".to_string()]));
    }

    #[test]
    fn welcome_str() {
        let mut board = create_test_board();
        board.submit_word("user1", "trident", "C2B1A2A3B2C3D2");
        assert_eq!(board.welcome_str(&vec!["user1".to_string()]),
                   "BIENVENUE/LIDAREJULTNEATNG/1*user1*5/\n")
    }

    #[test]
    fn new_turn() {
        let mut board = create_test_board();
        let mut old_grid = board.grid;
        let old_turn = board.turn;

        board.new_turn();

        assert_eq!(board.turn, old_turn + 1);
        assert_ne!(board.grid, old_grid);
    }

    #[test]
    fn reset() {
        let mut board = create_test_board();
        let mut old_grid = board.grid;

        board.reset();

        assert_eq!(board.turn, 1);
        assert_eq!(board.player_words, HashMap::new());
        assert_eq!(board.played, HashSet::new());
        assert_ne!(board.grid, old_grid);
    }

    pub fn create_test_board() -> Board {
        let mut board = Board::new();
        board.grid = ['L', 'I', 'D', 'A',
                     'R', 'E', 'J', 'U',
                     'L', 'T', 'N', 'E',
                     'A', 'T', 'N', 'G' ];
        board
    }
}
