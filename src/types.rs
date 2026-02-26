use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::io::{stdin, stdout, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use stoolap::{Database, Value};

// Import the pest parser types
use pest_parser::{parse_query, Query};

pub struct Question {
    pub id: i64,
    pub question: String,
    pub answer: String,
}

pub struct Answer {
    pub processed: String,
    pub correct: bool,
}

pub struct Answer {
    pub processed: String,
    pub correct: bool,
}
