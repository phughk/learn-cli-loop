use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct QueryParser;

pub struct Column {
    pub name: String,
}

pub struct Table {
    pub name: String,
}

pub struct Value {
    pub inner: ValueInner,
}

#[derive(Clone, Debug)]
pub enum ValueInner {
    Integer(i64),
    String(String),
    QuestionMark,
}

pub struct SelectStatement {
    pub columns: Vec<Column>,
    pub table: Table,
    pub conditions: Option<Vec<String>>,
    pub limit: Option<i64>,
}

pub struct InsertStatement {
    pub table: Table,
    pub values: Vec<Value>,
    pub conditions: Option<Vec<String>>,
}

pub struct UpdateStatement {
    pub table: Table,
    pub updates: Vec<(String, Value)>,
    pub conditions: Option<Vec<String>>,
}

pub struct DeleteStatement {
    pub table: Table,
    pub conditions: Option<Vec<String>>,
}

pub enum Query {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
}

pub fn parse_query(input: &str) -> Result<Query, pest::error::Error<Rule>> {
    let pairs = QueryParser::parse(Rule::QUERY, input)?;
    parse_statement(pairs.into_iter().next().unwrap())
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<Query, pest::error::Error<Rule>> {
    let rule = pair.as_rule();

    match rule {
        Rule::select_statement => parse_select(pair),
        Rule::insert_statement => parse_insert(pair),
        Rule::update_statement => parse_update(pair),
        Rule::delete_statement => parse_delete(pair),
        _ => Err(pest::error::Error::from_position(
            Rule::QUERY,
            pest::Position::new(pair.as_str(), 0),
        )),
    }
}

fn parse_select(pair: pest::iterators::Pair<Rule>) -> Result<Query, pest::error::Error<Rule>> {
    let mut inside = pair.into_inner();
    let mut columns = Vec::new();
    let mut conditions = None;
    let mut limit = None;

    inside.next().unwrap(); // SELECT
    inside.next().unwrap(); // FROM

    // Get table name
    let table_name = inside.next().unwrap().as_str().to_string();

    // Parse columns
    let columns_part = inside.next().unwrap();
    for col in columns_part.into_inner() {
        if col.as_rule() == Rule::col_spec {
            columns.push(Column {
                name: col.into_inner().next().unwrap().as_str().to_string(),
            });
        }
    }

    // Parse WHERE clause if present
    if let Some(where_pair) = inside.next() {
        if where_pair.as_rule() == Rule::WHERE {
            conditions = Some(parse_conditions(where_pair.into_inner()));
        }
    }

    // Parse LIMIT clause if present
    if let Some(limit_pair) = inside.next() {
        if limit_pair.as_rule() == Rule::LIMIT {
            limit = Some(
                limit_pair
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .parse::<i64>()?,
            );
        }
    }

    Ok(Query::Select(SelectStatement {
        columns,
        table: Table { name: table_name },
        conditions,
        limit,
    }))
}

fn parse_insert(pair: pest::iterators::Pair<Rule>) -> Result<Query, pest::error::Error<Rule>> {
    let mut inside = pair.into_inner();
    let mut values = Vec::new();
    let mut conditions = None;

    inside.next().unwrap(); // INSERT
    inside.next().unwrap(); // INTO
    inside.next().unwrap(); // VALUES

    // Skip table in INSERT
    inside.next();

    // Parse values
    let values_part = inside.next().unwrap();
    for val in values_part.into_inner() {
        if val.as_rule() == Rule::value {
            values.push(Value {
                inner: parse_value_inner(val.into_inner().next().unwrap()),
            });
        }
    }

    // Parse WHERE clause if present
    if let Some(where_pair) = inside.next() {
        if where_pair.as_rule() == Rule::WHERE {
            conditions = Some(parse_conditions(where_pair.into_inner()));
        }
    }

    Ok(Query::Insert(InsertStatement {
        table: Table {
            name: "questions".to_string(),
        },
        values,
        conditions,
    }))
}

fn parse_update(pair: pest::iterators::Pair<Rule>) -> Result<Query, pest::error::Error<Rule>> {
    let mut inside = pair.into_inner();
    let mut updates = Vec::new();
    let mut conditions = None;

    inside.next().unwrap(); // UPDATE
    inside.next().unwrap(); // SET

    // Skip table
    inside.next();

    // Parse updates
    let updates_part = inside.next().unwrap();
    for up in updates_part.into_inner() {
        if up.as_rule() == Rule::update_item {
            let mut parts = up.into_inner();
            let col_name = parts.next().unwrap().as_str().to_string();
            parts.next().unwrap(); // =
            let value = Value {
                inner: parse_value_inner(parts.next().unwrap()),
            };
            updates.push((col_name, value));
        }
    }

    // Parse WHERE clause if present
    if let Some(where_pair) = inside.next() {
        if where_pair.as_rule() == Rule::WHERE {
            conditions = Some(parse_conditions(where_pair.into_inner()));
        }
    }

    Ok(Query::Update(UpdateStatement {
        table: Table {
            name: "questions".to_string(),
        },
        updates,
        conditions,
    }))
}

fn parse_delete(pair: pest::iterators::Pair<Rule>) -> Result<Query, pest::error::Error<Rule>> {
    let mut inside = pair.into_inner();
    let mut conditions = None;

    inside.next().unwrap(); // DELETE
    inside.next().unwrap(); // FROM

    // Skip table
    inside.next();

    // Parse WHERE clause if present
    if let Some(where_pair) = inside.next() {
        if where_pair.as_rule() == Rule::WHERE {
            conditions = Some(parse_conditions(where_pair.into_inner()));
        }
    }

    Ok(Query::Delete(DeleteStatement {
        table: Table {
            name: "questions".to_string(),
        },
        conditions,
    }))
}

fn parse_value_inner(pair: pest::iterators::Pair<Rule>) -> ValueInner {
    match pair.as_rule() {
        Rule::INTEGER => ValueInner::Integer(pair.as_str().parse().unwrap()),
        Rule::STRING_LITERAL | Rule::QUOTED_STRING => ValueInner::String(pair.as_str().to_string()),
        Rule::QUESTION_MARK => ValueInner::QuestionMark,
        _ => ValueInner::Integer(0),
    }
}

fn parse_conditions(inner: pest::iterators::Pair<Rule>) -> Vec<String> {
    let mut conditions = Vec::new();
    for clause in inner {
        if clause.as_rule() == Rule::logical_and {
            conditions.push(clause.as_str().to_string());
        }
    }
    conditions
}

fn main() {
    println!("Pest DB Query Parser");
    println!("---------------------\n");

    let queries = vec![
        "SELECT id, question FROM questions",
        "SELECT * FROM questions WHERE id = 1",
        "SELECT * FROM questions WHERE question LIKE '%test%'",
        "INSERT INTO questions VALUES ('Question', 'Answer')",
        "UPDATE questions SET attempts = ? WHERE id = ?",
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
                        if let Some(l) = stmt.limit {
                            println!("  Limit: {}", l);
                        }
                    }
                    Query::Insert(stmt) => {
                        println!("  ✅ Table: {}", stmt.table.name);
                        println!(
                            "  Values: {:?}",
                            stmt.values
                                .iter()
                                .map(|v| match &v.inner {
                                    ValueInner::Integer(i) => format!("{}", i),
                                    ValueInner::String(s) => format!("'{}'", s),
                                    ValueInner::QuestionMark => "?".to_string(),
                                })
                                .collect::<Vec<_>>()
                        );
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
