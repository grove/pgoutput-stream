use anyhow::{anyhow, Result};
use crate::decoder::Change;
use serde_json;

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Json,
    JsonPretty,
    Text,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "json-pretty" => Ok(OutputFormat::JsonPretty),
            "text" => Ok(OutputFormat::Text),
            _ => Err(anyhow!("Unknown output format: {}. Valid options: json, json-pretty, text", s)),
        }
    }
}

pub fn print_change(change: &Change, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(change)?);
        }
        OutputFormat::JsonPretty => {
            println!("{}", serde_json::to_string_pretty(change)?);
        }
        OutputFormat::Text => {
            print_text_format(change);
        }
    }
    Ok(())
}

fn print_text_format(change: &Change) {
    match change {
        Change::Begin { lsn, timestamp, xid } => {
            println!("BEGIN [LSN: {}, XID: {}, Time: {}]", lsn, xid, timestamp);
        }
        Change::Commit { lsn, timestamp } => {
            println!("COMMIT [LSN: {}, Time: {}]", lsn, timestamp);
        }
        Change::Relation { relation_id, schema, table, columns } => {
            println!("RELATION [{}.{} (ID: {})]", schema, table, relation_id);
            println!("  Columns:");
            for col in columns {
                println!("    - {} (type_id: {}, flags: {})", col.name, col.type_id, col.flags);
            }
        }
        Change::Insert { relation_id, schema, table, new_tuple } => {
            println!("INSERT into {}.{} (ID: {})", schema, table, relation_id);
            println!("  New values:");
            for (key, value) in new_tuple {
                match value {
                    Some(v) => println!("    {}: {}", key, v),
                    None => println!("    {}: NULL", key),
                }
            }
        }
        Change::Update { relation_id, schema, table, old_tuple, new_tuple } => {
            println!("UPDATE {}.{} (ID: {})", schema, table, relation_id);
            if let Some(old) = old_tuple {
                println!("  Old values:");
                for (key, value) in old {
                    match value {
                        Some(v) => println!("    {}: {}", key, v),
                        None => println!("    {}: NULL", key),
                    }
                }
            }
            println!("  New values:");
            for (key, value) in new_tuple {
                match value {
                    Some(v) => println!("    {}: {}", key, v),
                    None => println!("    {}: NULL", key),
                }
            }
        }
        Change::Delete { relation_id, schema, table, old_tuple } => {
            println!("DELETE from {}.{} (ID: {})", schema, table, relation_id);
            println!("  Old values:");
            for (key, value) in old_tuple {
                match value {
                    Some(v) => println!("    {}: {}", key, v),
                    None => println!("    {}: NULL", key),
                }
            }
        }
    }
}
