

#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "L'utilisateur {} existe déja.", username)]
    ExistingUser { username: String },

    #[fail(display = "L'utilisateur {} n'existe pas.", username)]
    NonExistingUser { username: String },

    #[fail(display = "Coordinates ({}, {}) are invalid.", line, column)]
    InvalidCoordinates { line: char, column: usize},
}