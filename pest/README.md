# Pest Database Query Parser

A powerful PEG (Parsing Expression Grammar) parser for database query operations using the `pest` crate, enabled specifically for REPL interaction (not just CLI/Clap arguments).

## Overview

This parser enables database operations through the REPL interface using natural SQL-like syntax for SELECT, INSERT, UPDATE, and DELETE queries.

## Grammar Features

### Supported Operations

1. **SELECT Queries**
   - Simple column selection: `SELECT id, question FROM questions`
   - Wildcard selection: `SELECT * FROM questions`
   - Conditional selection with WHERE clause
   - Optional LIMIT clause

2. **INSERT Queries**
   - Simple value insertion: `INSERT INTO questions VALUES ('Question', 'Answer')`
   - Parameterized queries with placeholders: `INSERT INTO questions VALUES (?, ?)`
   - Optional WHERE clause

3. **UPDATE Queries**
   - Simple updates: `UPDATE questions SET attempts = ?`
   - Conditional updates: `UPDATE questions SET attempts = ? WHERE id = ?`
   - Multiple column updates supported

4. **DELETE Queries**
   - Simple deletion: `DELETE FROM questions`
   - Conditional deletion: `DELETE FROM questions WHERE id = ?`

## Grammar Rules

### Primary Components

- **WS**: Whitespace handling
- **COMMENT**: Comment support 
- **INT**: Integer literals
- **STRING_LITERAL**: Double-quoted strings
- **QUOTED_STRING**: Single-quoted strings
- **KEYWORD**: SQL keywords
- **identifier**: Column and table names

### Statement Types

- **SELECT**: Column selection and WHERE conditions
- **INSERT**: Value insertion with optional WHERE
- **UPDATE**: Column updates with optional WHERE
- **DELETE**: Row deletion with optional WHERE

### Expression Grammar

Supports complex conditional expressions:
- Logical AND/OR operations
- Comparison operators (=, ≠, <, >, ≤, ≥, LIKE)
- Mathematical expressions (+, -, *, /)
- Unary NOT operator
- Nested conditions

### Value Types

- **INTEGER**: Numeric values
- **STRING**: Text values
- **QUESTION_MARK**: Parameter placeholders (?, ?, ?)

## Usage

### Basic Parsing

```rust
use pest_query_parser::parser;

let result = parser::parse_query("SELECT id FROM questions")?;
```

### Integration with REPL

The parser can be integrated into the REPL to process database queries typed directly by users:

```rust
if let Ok(query) = parser::parse_query(input) {
    match query {
        parser::Query::Select(stmt) => {
            // Execute SELECT with conditions
            execute_select(&stmt);
        }
        parser::Query::Insert(stmt) => {
            // Execute INSERT
            execute_insert(&stmt);
        }
        // Handle Update and Delete similarly
    }
}
```

## Parser Functions

1. **parse_query** - Main entry point for parsing query strings
2. **parse_select** - Parse SELECT statements
3. **parse_insert** - Parse INSERT statements
4. **parse_update** - Parse UPDATE statements
5. **parse_delete** - Parse DELETE statements
6. **parse_value_inner** - Parse single value types
7. **parse_conditions** - Extract WHERE conditions

## Example Queries

### Select Examples
```rust
"SELECT id, question FROM questions"
"SELECT * FROM questions WHERE id = 1"
"SELECT * FROM questions WHERE question LIKE '%test%' AND attempts < 5 LIMIT 5"
```

### Insert Examples
```rust
"INSERT INTO questions VALUES ('Question', 'Answer')"
"INSERT INTO questions VALUES (?, ?)"
"INSERT INTO questions VALUES (?, ?) WHERE id = 1"
```

### Update Examples
```rust
"UPDATE questions SET attempts = ?"
"UPDATE questions SET attempts = ? WHERE id = 1"
"UPDATE questions SET correct = 1, attempts = ? WHERE id = ?"
```

### Delete Examples
```rust
"DELETE FROM questions WHERE id = 1"
"DELETE FROM questions WHERE attempts = 0"
```

## Error Handling

Returns `pest::error::Error<Rule>` for parsing errors, which provides:

- Error position information
- Expected rules
- Current rule mismatch details

## Limitations

- Uses `questions` as fixed table name for INSERT/UPDATE/DELETE (easily configurable)
- Basic SQL-like syntax without full SQL compliance
- Supports essential database operations but not advanced SQL features

## Future Enhancements

- Dynamic table name support
- Additional SQL operator support (BETWEEN, IN, OR, NOT, etc.)
- Query execution integration with actual database
- Result set parsing and formatting
- Transaction support
- JOIN queries
- Complex WHERE clause parsing
- Transaction rollback/commit