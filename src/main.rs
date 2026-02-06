use clap::{Parser, Subcommand};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::io::{stdin, stdout, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use stoolap::{Database, Value};

#[derive(Parser)]
#[command(name = "QuizApp")]
#[command(about = "A database-backed quiz tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Mode 1: Create the database and schema
    Init,
    /// Mode 2: Add or remove entries from the database
    Manage,
    /// Mode 3: Start the quiz
    Quiz,
}

fn main() {
    let cli = Cli::parse();
    let db = Database::open("file://my_database").expect("Failed to open database");

    match &cli.command {
        Commands::Init => {
            println!("Initializing database schema...");
            db.execute("CREATE TABLE questions (\
                id INTEGER PRIMARY KEY AUTO_INCREMENT,\
                question TEXT,\
                answer TEXT, \
                correct INTEGER, \
                attempts INTEGER, \
                last_checked INTEGER)", ())
                .expect("Failed to create table");
            println!("✅ Database initialized successfully.");
        }
        Commands::Manage => {
            manage_entries(&db);
        }
        Commands::Quiz => {
            run_quiz(&db);
        }
    }
}

fn run_quiz(db: &Database) {
    let total_questions = {
        let mut count_rows = db.query("SELECT COUNT(*) FROM questions", ()).unwrap();
        let total: i64 = count_rows.next().unwrap().ok().unwrap().get(0).ok().unwrap();
        total
    };
    let mut correct_count = 0u32;
    let mut rng = ChaCha12Rng::seed_from_u64(123);
    for i in 1u32.. {
        let q = find_question(&db, &mut rng, &total_questions);
        print!("[{:>3}/{:>3}] {}: ", correct_count, i, q.question);
        stdout().flush().expect("Failed to flush stdout");
        let answer = read_and_check_answer(&q);
        if answer.correct {
            println!("✅ Correct!");
            correct_count += 1;
        } else {
            println!("❌ Wrong! The answer was: {}", q.answer);
        }
        update_database(db, &q, &answer);
        println!("--")
    }
}

fn manage_entries(db: &Database) {
    println!("--- Database Management ---");
    println!("1. Add entry");
    println!("2. Remove entry");
    print!("Select an option: ");
    stdout().flush().unwrap();

    let mut choice = String::new();
    stdin().read_line(&mut choice).unwrap();

    match choice.trim() {
        "1" => {
            let question_text = prompt("Enter question: ");
            let answer_text = prompt("Enter answer: ");

            db.execute(
                "INSERT INTO questions (question, answer, correct, attempts, last_checked) VALUES (?, ?, 0, 0, 0)",
                (question_text, answer_text),
            ).expect("Failed to insert question");

            println!("✅ Question added.");
        }
        "2" => {
            // List questions first so user knows what to delete
            println!("\nExisting Questions:");
            let rows = db.query("SELECT id, question FROM questions", ()).unwrap();
            for row in rows {
                let row = row.unwrap();
                let id: i64 = row.get(0).unwrap();
                let text: String = row.get(1).unwrap();
                println!("{}: {}", id, text);
            }

            let id = prompt("\nEnter ID to remove: ");
            if let Ok(id) = id.parse::<i64>() {
                db.execute("DELETE FROM questions WHERE id = $1", vec![Value::Integer(id)])
                    .expect("Failed to delete question");
                println!("✅ Removed ID {}.", id);
            } else {
                println!("Done goofed - nan")
            }
        }
        _ => println!("Invalid selection."),
    }
}

fn prompt(message: &str) -> String {
    print!("{}", message);
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn find_question(db: &Database, rng: &mut ChaCha12Rng, total: &i64) -> Question {
    // Find the total number of questions from database and pick a random question
    let choice = rng.random_range(0..*total);
    let mut rows = db.query("SELECT id, question, answer FROM questions LIMIT 1 OFFSET $1", vec![Value::Integer(choice)]).unwrap();

    let row = rows.next().unwrap().unwrap();
    let id: i64 = row.get(0).unwrap();
    let question: String = row.get(1).unwrap();
    let answer: String = row.get(2).unwrap();


    // Placeholder as requested
    Question {
        id,
        question,
        answer,
    }
}

pub struct Question {
    pub id: i64,
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

fn update_database(db: &Database, q: &Question, answer: &Answer) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let increment_correct = if answer.correct { 1 } else { 0 };

    // Update the row: increment attempts, optionally increment correct, and set timestamp
    db.execute(
        "UPDATE questions \
         SET attempts = attempts + 1, \
             correct = correct + $1, \
             last_checked = $2 \
         WHERE id = $3",
        vec![
            Value::Integer(increment_correct),
            Value::Integer(now),
            Value::Integer(q.id),
        ],
    ).expect("Failed to update database metrics");
}
