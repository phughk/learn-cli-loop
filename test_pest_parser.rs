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

    let table_name = inside.next().unwrap().as_str().to_string();

    let columns_part = inside.next().unwrap();
    for col in columns_part.into_inner() {
        if col.as_rule() == Rule::col_spec {
            columns.push(Column {
                name: col.into_inner().next().unwrap().as_str().to_string(),
            });
        }
    }

    if let Some(where_pair) = inside.next() {
        if let Rule::WHERE = where_pair.as_rule() {
            conditions = Some(parse_conditions(where_pair.into_inner()));
        }
    }

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

    inside.next();

    let values_part = inside.next().unwrap();
    for val in values_part.into_inner() {
        if let Rule::value = val.as_rule() {
            values.push(Value {
                inner: parse_value_inner(val.into_inner().next().unwrap()),
            });
        }
    }

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

    inside.next();

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

    inside.next();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select_simple() {
        let input = "SELECT id, question FROM questions";
        let result = parse_query(input);
        assert!(result.is_ok());

        match result.unwrap() {
            Query::Select(stmt) => {
                assert_eq!(stmt.table.name, "questions");
                assert_eq!(stmt.columns.len(), 2);
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_parse_select_with_conditions() {
        let input = "SELECT * FROM questions WHERE id = 1";
        let result = parse_query(input);
        assert!(result.is_ok());

        match result.unwrap() {
            Query::Select(stmt) => {
                assert!(stmt.conditions.is_some());
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_parse_select_with_limit() {
        let input = "SELECT * FROM questions WHERE attempts < 5 LIMIT 10";
        let result = parse_query(input);
        assert!(result.is_ok());

        match result.unwrap() {
            Query::Select(stmt) => {
                assert_eq!(stmt.limit, Some(10));
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_parse_insert() {
        let input = "INSERT INTO questions VALUES ('Hello', 'World')";
        let result = parse_query(input);
        assert!(result.is_ok());

        match result.unwrap() {
            Query::Insert(stmt) => {
                assert_eq!(stmt.table.name, "questions");
                assert_eq!(stmt.values.len(), 2);
            }
            _ => panic!("Expected Insert statement"),
        }
    }

    #[test]
    fn test_parse_update() {
        let input = "UPDATE questions SET attempts = ? WHERE id = 1";
        let result = parse_query(input);
        assert!(result.is_ok());

        match result.unwrap() {
            Query::Update(stmt) => {
                assert_eq!(stmt.table.name, "questions");
                assert_eq!(stmt.updates.len(), 1);
            }
            _ => panic!("Expected Update statement"),
        }
    }

    #[test]
    fn test_parse_delete() {
        let input = "DELETE FROM questions WHERE id = 1";
        let result = parse_query(input);
        assert!(result.is_ok());

        match result.unwrap() {
            Query::Delete(stmt) => {
                assert_eq!(stmt.table.name, "questions");
            }
            _ => panic!("Expected Delete statement"),
        }
    }

    #[test]
    fn test_addq_parsing() {
        let input = "ADDQ \"What is the capital of France?\" \"Paris\"";
        let result = parse_query(input);
        assert!(result.is_ok());
    }
}
