use std::fmt;
use std::borrow::Cow;

use rand::{self, Rng};

/// Table to retrieve base62 values from.
const BASE62: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn new_id(size: usize) -> String {
    let mut id = String::with_capacity(size);
    let mut rng = rand::thread_rng();
    for _ in 0..size {
        id.push(BASE62[rng.gen::<usize>() % 62] as char);
    }
    id
}

// pub struct MakeID<'a>(Cow<'a, str>);
//
// impl<'a> MakeID<'a> {
//     /// Generate a _probably_ unique ID with `size` characters. For readability,
//     /// the characters used are from the sets [0-9], [A-Z], [a-z]. The
//     /// probability of a collision depends on the value of `size`. In
//     /// particular, the probability of a collision is 1/62^(size).
//     pub fn new(size: usize) -> MakeID<'static> {
//         let mut id = String::with_capacity(size);
//         let mut rng = rand::thread_rng();
//         for _ in 0..size {
//             id.push(BASE62[rng.gen::<usize>() % 62] as char);
//         }
//
//         MakeID(Cow::Owned(id))
//     }
// }
//
// impl<'a> fmt::Display for MakeID<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }
//
//
// /// Returns `true` if `id` is a valid paste ID and `false` otherwise.
// fn valid_id(id: &str) -> bool {
//     id.chars().all(|c| {
//         (c >= 'a' && c <= 'z')
//             || (c >= 'A' && c <= 'Z')
//             || (c >= '0' && c <= '9')
//     })
// }
