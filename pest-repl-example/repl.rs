use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::io::{stdin, stdout, Read, Write as StdWrite};
use std::time::{SystemTime, UNIX_EPOCH};
use stoolap::{Database, Rows, Value};

pub struct Question {
    pub id: i64,
    pub question: String,
    pub answer: String,
}

pub struct Answer {
    pub processed: String,
    pub correct: bool,
}

fn prompt(message: &str) -> String {
    print!("{}", message);
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn parse_addq(query: &str) -> Option<(String, String)> {
    // Parse ADDQ "question" "answer" format
    let parts: Vec<&str> = query.split('"').collect();
    if parts.len() >= 4 {
        let question = parts[1].trim().to_string();
        let answer = parts[3].trim().to_string();
        if !question.is_empty() && !answer.is_empty() {
            return Some((question, answer));
        }
    }
    None
}

fn handle_addq(db: &Database, query: &str) {
    if let Some((question, answer)) = parse_addq(query) {
        let success = db.execute(
            "INSERT INTO questions (question, answer, correct, attempts, last_checked) VALUES (?, ?, 0, 0, 0)",
            (question, answer)
        ).is_ok();

        if success {
            println!("✅ Question added: {} -> {}", question, answer);
        } else {
            println!("❌ Failed to add question");
        }
    } else {
        println!("❌ ADDQ format incorrect. Use: ADDQ \"question\" \"answer\"");
    }
}

fn handle_quiz(db: &Database, query: &str) {
    let mut rng = ChaCha12Rng::from_os_rng();

    // Parse QUIZ command like QUIZ 10 LEAST RECENT
    if query.starts_with("QUIZ") {
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(num) = parts[1].parse::<usize>() {
                let order = parts
                    .get(2)
                    .map(|s| s.to_lowercase())
                    .unwrap_or(String::from("RANDOM"));

                println!("Starting quiz with {} questions ordered by {}", num, order);

                let query_str = match order.as_str() {
                    "least recent" | "recent" => "SELECT id, question, answer, correct, attempts FROM questions ORDER BY last_checked ASC",
                    "most recent" => "SELECT id, question, answer, correct, attempts FROM questions ORDER BY last_checked DESC",
                    "random" | "least" => "SELECT id, question, answer, correct, attempts FROM questions ORDER BY id ASC",
                    "most" | "least-recent" | "most-recent" => "SELECT id, question, answer, correct, attempts FROM questions ORDER BY last_checked ASC",
                    _ => "SELECT id, question, answer, correct, attempts FROM questions ORDER BY RANDOM()"
                };

                println!("Executing: {}", query_str);
                execute_quiz(db, &mut rng, query_str, num);
            } else {
                println!("❌ QUIZ number format incorrect");
            }
        } else {
            println!("❌ QUIZ requires number parameter. Use: QUIZ <number> <order>");
            println!("Order options: least, most, random");
            println!("Example: QUIZ 10 most-recent");
        }
    }
}

fn execute_quiz(db: &Database, rng: &mut ChaCha12Rng, query_str: &str, num: usize) {
    let query_result = db.query(query_str, ());

    let mut session_pool = Vec::new();

    if let Ok(rows_result) = query_result {
        for row in rows_result.iter() {
            if let Ok(row_obj) = row {
                if session_pool.len() >= num * 2 {
                    continue;
                }

                let id = match row_obj.get(0) {
                    Some(Ok(v)) => v.clone(),
                    _ => continue,
                };

                let question = match row_obj.get(1) {
                    Some(Ok(v)) => v.clone(),
                    _ => continue,
                };

                let answer = match row_obj.get(2) {
                    Some(Ok(v)) => v.clone(),
                    _ => continue,
                };

                let question_obj = Question {
                    id,
                    question,
                    answer,
                };

                if rng.random_bool(0.7) {
                    session_pool.push(question_obj);
                }
            }
        }
    }

    if session_pool.is_empty() {
        println!("❌ No questions met quiz criteria.");
        return;
    }

    // Shuffle the session pool
    for i in (0..session_pool.len()).rev() {
        let j = rng.random_range(0..(i + 1));
        session_pool.swap(i, j);
    }

    println!(
        "--- Starting quiz with {} questions ---",
        session_pool.len()
    );
    let mut correct_count = 0u32;

    for i in 1..=num {
        if session_pool.is_empty() {
            break;
        }

        let idx = 0; // Use first element since we shuffled
        let q = session_pool.remove(idx);

        print!("[{:>3}/{:>3}] {}: ", correct_count + 1, i, q.question);
        stdout().flush().unwrap();

        let answer = read_answer();
        if answer.correct {
            println!("✅ Correct!");
            correct_count += 1;
        } else {
            println!("❌ Wrong! The answer was: {}", q.answer);
        }

        // Update database
        update_database(db, &q, &answer);
        println!();
    }

    println!(
        "\n🎉 Quiz complete! You got {}/{} correct.",
        correct_count, num
    );
}

fn read_answer() -> Answer {
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read line");

    let processed = input.trim().to_lowercase();
    Answer {
        processed: processed.clone(),
        correct: !processed.is_empty(),
    }
}

fn update_database(db: &Database, q: &Question, answer: &Answer) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let increment_correct = if answer.correct { 1 } else { 0 };

    let _ = db.execute(
        "UPDATE questions \
         SET attempts = attempts + 1, \
             correct = correct + ?, \
             last_checked = ? \
         WHERE id = ?",
        vec![
            Value::Integer(increment_correct),
            Value::Integer(now),
            Value::Integer(q.id),
        ],
    );
}

pub struct REPL {
    pub db: Database,
    pub running: bool,
}

impl REPL {
    pub fn new() -> Result<Self, stoolap::Error> {
        let db = Database::open("file://my_database")?;
        Ok(REPL { db, running: true })
    }

    pub fn start(&mut self) {
        println!("📊 Quiz REPL");
        println!("------------");
        println!("Available commands:");
        println!("  ADDQ \"question\" \"answer\"  - Add a new question");
        println!("  QUIZ <number> <order>    - Start a quiz");
        println("  SELECT ...               - Query questions");
        println("  INSERT INTO ...         - Insert questions");
        println("  UPDATE questions SET ... - Update questions");
        println("  DELETE FROM ...         - Delete questions");
        println!("  HELP                    - Show this help");
        println!("  EXIT                    - Quit the REPL");
        println!("--------------------------");

        while self.running {
            print!("quiz> ");
            stdout().flush().unwrap();

            let mut input = String::new();
            if let Err(_) = stdin().read_line(&mut input) {
                break;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            let input_lower = input.to_lowercase();
            if input_lower == "exit" || input_lower == "quit" {
                self.running = false;
                println!("Goodbye!");
                continue;
            } else if input_lower == "help" {
                println!("\nAvailable commands:");
                println!("  ADDQ \"question\" \"answer\"  - Add a new question");
                println!("  QUIZ <number> <order>    - Start a quiz");
                println!("  SELECT ...               - Query questions");
                println!("  INSERT, UPDATE, DELETE   - Database operations");
                println!("  HELP                    - Show this help");
                println!("  EXIT                    - Quit the REPL");
                println!();
                continue;
            }

            if input_lower.starts_with("addq") {
                handle_addq(&self.db, input);
            } else if input_lower.starts_with("quiz") {
                handle_quiz(&self.db, input);
            } else {
                // Execute SQL query
                let query_result = self.db.query(input, ());

                if let Ok(rows_result) = query_result {
                    display_query_results(&rows_result);
                } else {
                    println!("❓ Command not recognized. Type HELP for options.");
                }
            }
        }
    }
}

fn display_query_results(rows_result: &[Option<Result<stoolap::Row, stoolap::Error>>]) {
    // Find column names from first row
    if let Some(first_row) = rows_result.first() {
        if let Ok(row) = first_row {
            let column_names: Vec<String> =
                row.iter().map(|(k, _)| normalize_column_name(k)).collect();

            // Display column headers
            for (i, name) in column_names.iter().enumerate() {
                if i > 0 {
                    print!("| ");
                }
                print!("{}", name);
            }
            println!();

            // Display separator
            for _ in 0..column_names.len() {
                print!("---|");
            }
            println!();

            // Display rows
            for (i, row) in rows_result.iter().enumerate() {
                if let Ok(row_obj) = row {
                    let values: Vec<String> = column_names
                        .iter()
                        .map(|col| {
                            let val = row_obj.get(col);
                            match val {
                                Some(Ok(v)) => format!("{}", v),
                                Some(Err(e)) => format!("ERROR: {}", e),
                                None => "NULL".to_string(),
                            }
                        })
                        .collect();

                    print!("{}: ", i + 1);
                    for (j, val) in values.iter().enumerate() {
                        if j > 0 {
                            print!("| ");
                        }
                        print!("{}", val);
                    }
                    println!();
                }
            }
        }
    }
}
