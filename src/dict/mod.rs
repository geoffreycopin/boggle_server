mod local_dict;

use std::marker::Send;

pub use self::local_dict::LocalDict;

pub trait Dict: Send {
    fn contains(&self, word: &str) -> bool;
}