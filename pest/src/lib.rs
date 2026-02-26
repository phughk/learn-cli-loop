use pest::Parser;
use pest::Rule;
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
        _ => Err(pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::CustomError {
                message: "Unexpected rule".to_string(),
            },
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
        if let Rule::WHERE = where_pair.as_rule() {
            conditions = Some(parse_conditions(where_pair.into_inner()));
        }
    }

    // Parse LIMIT clause if present
    if let Some(limit_pair) = inside.next() {
        if let Rule::LIMIT = limit_pair.as_rule() {
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
        if let Rule::value = val.as_rule() {
            values.push(Value {
                inner: parse_value_inner(val.into_inner().next().unwrap()),
            });
        }
    }

    // Parse WHERE clause if present
    if let Some(where_pair) = inside.next() {
        if let Rule::WHERE = where_pair.as_rule() {
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
        if let Rule::update_item = up.as_rule() {
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
        if let Rule::WHERE = where_pair.as_rule() {
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
        if let Rule::WHERE = where_pair.as_rule() {
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
        if let Rule::logical_and = clause.as_rule() {
            conditions.push(clause.as_str().to_string());
        }
    }
    conditions
}
