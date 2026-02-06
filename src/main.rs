use std::io::{stdin, stdout, Write};

fn main() {
    let stdin = stdin();

    let mut correct_count = 0u32;
    for i in 1u32.. {
        let q = find_question();
        print!("[{:>3}/{:>3}] {}: ", correct_count, i, q.question);
        stdout().flush().expect("Failed to flush stdout");
        let answer = read_and_check_answer(&q);
        if answer.correct {
            println!("✅ Correct!");
            correct_count += 1;
        } else {
            println!("❌ Wrong! The answer was: {}", q.answer);
        }
        update_database(&q, &answer);
        println!("--")
    }
}

fn find_question() -> Question {
    // Don't change this, I will add the database in a bit
    Question {
        question: "What is this?".to_string(),
        answer: "Kinaraya".to_string(),
    }
}

pub struct Question {
    pub question: String,
    pub answer: String,
}

pub struct Answer {
    /// After trim, lowercase
    pub processed: String,
    pub correct: bool,
}

fn read_and_check_answer(q: &Question) -> Answer{
    let mut input = String::new();

    stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Process: trim whitespace/newlines and convert to lowercase for case-insensitive matching
    let processed = input.trim().to_lowercase();
    let is_correct = processed == q.answer.trim().to_lowercase();

    Answer {
        processed,
        correct: is_correct,
    }
}

fn update_database(q: &Question, answer: &Answer) {
    println!("Database updated - don't change this")
}