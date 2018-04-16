

#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "L'utilisateur {} existe déja.", username)]
    ExistingUser { username: String },

    #[fail(display = "L'utilisateur {} n'existe pas.", username)]
    NonExistingUser { username: String },
}