# Quiz REPL - Pest Grammar Integration

This example shows how to integrate the Pest grammar system with a REPL to support database operations through natural language commands rather than just CLI arguments.

## Design Requirements

From the DESIGN.md:
> The user should be able to choose a dataset, add questions and answers, then quiz themselves.
> After which a quiz will begin with 10 of the least recent questions asked.

## Implementation

The Pest grammar enables REPL commands like:
- `ADDQ "Where do you live?" "At home"`
- `QUIZ 10 LEAST RECENT`

## Pest Grammar Usage

```rust
use pest_query_parser::{parse_query, Query};

// Parse a query
let result = parse_query("SELECT id, question FROM questions WHERE attempts < 5")?;

match result {
    Query::Select(stmt) => {
        // Execute SELECT with database operations
        execute_select(&db, stmt);
    }
    Query::Insert(stmt) => {
        // Parse INSERT statements
        execute_insert(&db, stmt);
    }
    Query::Update(stmt) => {
        // Parse UPDATE statements
        execute_update(&db, stmt);
    }
    Query::Delete(stmt) => {
        // Parse DELETE statements
        execute_delete(&db, stmt);
    }
}
```

## RECOMMENDED: Single REPL Command Integration

The pest grammar is most powerful when integrated into a REPL loop:

```rust
while repl_running {
    print!("quiz> ");
    stdin().read_line(&mut input)?;
    
    let lower_input = input.to_lowercase();
    
    // Handle REPL commands
    if lower_input.starts_with("addq") {
        handle_addq(&db, input);
    } else if lower_input.starts_with("quiz") {
        handle_quiz(&db, input);
    } else {
        // Try pest parser for SQL-like queries
        match parse_query(input) {
            Ok(parsed_query) => execute_parsed_query(&db, parsed_query),
            Err(_) => println!("❓ Unknown command"),
        }
    }
}
```

## Example REPL Session

```
📊 Quiz REPL
------------
Available commands:
  ADDQ "question" "answer"  - Add a new question
  QUIZ <number> <order>    - Start a quiz
  SELECT ...               - Query questions
  INSERT INTO ...         - Insert questions
  ...

quiz> ADDQ "Where do you live?" "At home"
✅ Question added: Where do you live? -> At home

quiz> ADDQ "What is the capital of France?" "Paris"
✅ Question added: What is the capital of France? -> Paris

quiz> QUIZ 10 LEAST RECENT
Starting quiz with 10 questions ordered by least recent
Executing: SELECT id, question, answer, correct, attempts FROM questions ORDER BY last_checked ASC
Found 2 total questions, selecting 10 for quiz...
--- Starting quiz with 2 questions ---

[  1/  1] Where do you live? The correct answer was: at home

🎉 Quiz complete! You got 1/1 correct.

quiz> SELECT * FROM questions
id    question                             | answer | correct | attempts | last_checked
---|---|---|---|---|---
1:    Where do you live?                          | at home | NULL    | NULL     | 1234567890
2:    What is the capital of France?               | paris   | NULL    | NULL     | 9876543210
```

## Advantages of Pest Grammar Integration

1. **Natural Commands**: Users can type `ADDQ "question" "answer"` instead of `manage` subcommand
2. **Flexible Queries**: `SELECT * FROM questions WHERE attempts < 5 LIMIT 10`
3. **Reactive**: Instant feedback without CLI flag parsing
4. **Extensible**: Add new commands through the grammar
5. **No CLI Arguments**: All operations from single REPL interface

## Key Features

- **ADDQ Command**: Handles quote-enclosed question and answer pairs
- **QUIZ Command**: Supports ordering by least/most recent, random, or specific criteria
- **SQL Integration**: Natural SQL queries for advanced filtering
- **Database Tracking**: Automatic tracking of attempts and correctness