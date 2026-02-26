use pest_parser::parser::{parse_query, Query};

fn main() {
    println!("Pest DB Query Parser");
    println!("---------------------\n");

    let queries = vec![
        "SELECT id, question FROM questions",
        "SELECT * FROM questions WHERE id = 1",
        "SELECT * FROM questions WHERE question LIKE '%test%'",
        "INSERT INTO questions VALUES ('Question', 'Answer')",
        "UPDATE questions SET attempts = ?",
        "DELETE FROM questions WHERE id = 1",
    ];

    let mut valid_count = 0;
    let mut total_count = queries.len();

    for query_str in queries {
        println!("Parsing: {}", query_str);

        match parse_query(query_str) {
            Ok(parsed) => {
                valid_count += 1;
                match parsed {
                    Query::Select(stmt) => {
                        println!("  ✅ Table: {}", stmt.table.name);
                        println!("  Columns: {:?}", stmt.columns);
                        if let Some(cs) = stmt.conditions {
                            println!("  Conditions: {:?}", cs);
                        }
                    }
                    Query::Insert(stmt) => {
                        println!("  ✅ Table: {}", stmt.table.name);
                        println!("  Values: {:?}", stmt.values);
                    }
                    Query::Update(stmt) => {
                        println!("  ✅ Table: {}", stmt.table.name);
                        println!("  Updates: {:?}", stmt.updates);
                    }
                    Query::Delete(stmt) => {
                        println!("  ✅ Table: {}", stmt.table.name);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Failed: {}", e);
            }
        }
    }

    println!("\nSummary: {}/{} valid queries", valid_count, total_count);
}
