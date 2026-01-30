use pgoutput_cmdline::output::*;
use pgoutput_cmdline::decoder::*;
use std::collections::HashMap;

#[test]
fn test_output_format_from_str_json() {
    let format = OutputFormat::from_str("json").unwrap();
    assert!(matches!(format, OutputFormat::Json));
}

#[test]
fn test_output_format_from_str_json_pretty() {
    let format = OutputFormat::from_str("json-pretty").unwrap();
    assert!(matches!(format, OutputFormat::JsonPretty));
}

#[test]
fn test_output_format_from_str_text() {
    let format = OutputFormat::from_str("text").unwrap();
    assert!(matches!(format, OutputFormat::Text));
}

#[test]
fn test_output_format_from_str_case_insensitive() {
    assert!(matches!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json));
    assert!(matches!(OutputFormat::from_str("Json").unwrap(), OutputFormat::Json));
    assert!(matches!(OutputFormat::from_str("TEXT").unwrap(), OutputFormat::Text));
}

#[test]
fn test_output_format_from_str_invalid() {
    assert!(OutputFormat::from_str("invalid").is_err());
    assert!(OutputFormat::from_str("xml").is_err());
    assert!(OutputFormat::from_str("").is_err());
}

#[test]
fn test_json_serialization_begin() {
    let change = Change::Begin {
        lsn: "0/123456".to_string(),
        timestamp: 123456789,
        xid: 999,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("Begin"));
    assert!(json.contains("0/123456"));
    assert!(json.contains("123456789"));
    assert!(json.contains("999"));
    
    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_json_serialization_commit() {
    let change = Change::Commit {
        lsn: "0/789ABC".to_string(),
        timestamp: 987654321,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("Commit"));
    assert!(json.contains("0/789ABC"));
    assert!(json.contains("987654321"));
}

#[test]
fn test_json_serialization_insert() {
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    new_tuple.insert("name".to_string(), Some("Alice".to_string()));
    
    let change = Change::Insert {
        relation_id: 100,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("Insert"));
    assert!(json.contains("public"));
    assert!(json.contains("users"));
    assert!(json.contains("Alice"));
    
    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_json_serialization_insert_with_null() {
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    new_tuple.insert("email".to_string(), None);
    
    let change = Change::Insert {
        relation_id: 100,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("null"));
    
    // Verify proper JSON null handling
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let insert = &value["Insert"];
    assert!(insert["new_tuple"]["email"].is_null());
}

#[test]
fn test_json_serialization_update_with_old_tuple() {
    let mut old_tuple = HashMap::new();
    old_tuple.insert("name".to_string(), Some("Bob".to_string()));
    
    let mut new_tuple = HashMap::new();
    new_tuple.insert("name".to_string(), Some("Robert".to_string()));
    
    let change = Change::Update {
        relation_id: 200,
        schema: "public".to_string(),
        table: "users".to_string(),
        old_tuple: Some(old_tuple),
        new_tuple,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("Update"));
    assert!(json.contains("Bob"));
    assert!(json.contains("Robert"));
}

#[test]
fn test_json_serialization_update_without_old_tuple() {
    let mut new_tuple = HashMap::new();
    new_tuple.insert("name".to_string(), Some("Carol".to_string()));
    
    let change = Change::Update {
        relation_id: 200,
        schema: "public".to_string(),
        table: "users".to_string(),
        old_tuple: None,
        new_tuple,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("Update"));
    assert!(json.contains("Carol"));
    
    // Verify old_tuple is null
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(value["Update"]["old_tuple"].is_null());
}

#[test]
fn test_json_serialization_delete() {
    let mut old_tuple = HashMap::new();
    old_tuple.insert("id".to_string(), Some("42".to_string()));
    
    let change = Change::Delete {
        relation_id: 300,
        schema: "public".to_string(),
        table: "users".to_string(),
        old_tuple,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("Delete"));
    assert!(json.contains("42"));
}

#[test]
fn test_json_serialization_relation() {
    let columns = vec![
        ColumnInfo {
            name: "id".to_string(),
            type_id: 23,
            flags: 1,
        },
        ColumnInfo {
            name: "name".to_string(),
            type_id: 1043,
            flags: 0,
        },
    ];
    
    let change = Change::Relation {
        relation_id: 12345,
        schema: "public".to_string(),
        table: "users".to_string(),
        columns,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    assert!(json.contains("Relation"));
    assert!(json.contains("12345"));
    assert!(json.contains("public"));
    assert!(json.contains("users"));
    assert!(json.contains("\"name\":\"id\""));
    assert!(json.contains("\"type_id\":23"));
}

#[test]
fn test_json_pretty_format() {
    let change = Change::Begin {
        lsn: "0/123456".to_string(),
        timestamp: 123456789,
        xid: 999,
    };
    
    let json_pretty = serde_json::to_string_pretty(&change).unwrap();
    
    // Pretty format should have newlines and indentation
    assert!(json_pretty.contains("\n"));
    assert!(json_pretty.contains("  ")); // Indentation
}

#[test]
fn test_json_special_characters() {
    let mut new_tuple = HashMap::new();
    new_tuple.insert("description".to_string(), Some("Test \"quotes\" and \\backslash".to_string()));
    
    let change = Change::Insert {
        relation_id: 100,
        schema: "public".to_string(),
        table: "items".to_string(),
        new_tuple,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    
    // Verify special characters are properly escaped
    assert!(json.contains("\\\""));
    assert!(json.contains("\\\\"));
    
    // Verify it can be parsed back
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();
}

#[test]
fn test_json_unicode() {
    let mut new_tuple = HashMap::new();
    new_tuple.insert("name".to_string(), Some("Håkon Müller 李明".to_string()));
    
    let change = Change::Insert {
        relation_id: 100,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    
    // Verify Unicode is preserved
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let name = value["Insert"]["new_tuple"]["name"].as_str().unwrap();
    assert_eq!(name, "Håkon Müller 李明");
}

#[test]
fn test_json_empty_string() {
    let mut new_tuple = HashMap::new();
    new_tuple.insert("description".to_string(), Some("".to_string()));
    
    let change = Change::Insert {
        relation_id: 100,
        schema: "public".to_string(),
        table: "items".to_string(),
        new_tuple,
    };
    
    let json = serde_json::to_string(&change).unwrap();
    
    // Verify empty string is properly serialized
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let desc = value["Insert"]["new_tuple"]["description"].as_str().unwrap();
    assert_eq!(desc, "");
}

// Tests for new OutputTarget trait and implementations

#[tokio::test]
async fn test_stdout_output_insert() {
    let output = StdoutOutput::new(OutputFormat::Json);
    
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    new_tuple.insert("name".to_string(), Some("Alice".to_string()));
    
    let change = Change::Insert {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple,
    };
    
    // Should not panic
    output.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_stdout_output_update() {
    let output = StdoutOutput::new(OutputFormat::Json);
    
    let mut old_tuple = HashMap::new();
    old_tuple.insert("id".to_string(), Some("1".to_string()));
    old_tuple.insert("name".to_string(), Some("Alice".to_string()));
    
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    new_tuple.insert("name".to_string(), Some("Alice Updated".to_string()));
    
    let change = Change::Update {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        old_tuple: Some(old_tuple),
        new_tuple,
    };
    
    output.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_stdout_output_delete() {
    let output = StdoutOutput::new(OutputFormat::JsonPretty);
    
    let mut old_tuple = HashMap::new();
    old_tuple.insert("id".to_string(), Some("1".to_string()));
    old_tuple.insert("name".to_string(), Some("Alice".to_string()));
    
    let change = Change::Delete {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        old_tuple,
    };
    
    output.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_stdout_output_transaction_events() {
    let output = StdoutOutput::new(OutputFormat::Text);
    
    let begin = Change::Begin {
        lsn: "0/16B2D50".to_string(),
        timestamp: 730826470123456,
        xid: 1000,
    };
    output.write_change(&begin).await.unwrap();
    
    let commit = Change::Commit {
        lsn: "0/16B2E20".to_string(),
        timestamp: 730826470123457,
    };
    output.write_change(&commit).await.unwrap();
}

#[tokio::test]
async fn test_stdout_output_relation() {
    let output = StdoutOutput::new(OutputFormat::Json);
    
    let columns = vec![
        ColumnInfo {
            name: "id".to_string(),
            type_id: 23,
            flags: 1,
        },
        ColumnInfo {
            name: "name".to_string(),
            type_id: 1043,
            flags: 0,
        },
    ];
    
    let change = Change::Relation {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        columns,
    };
    
    output.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_composite_output_with_single_target() {
    use std::sync::Arc;
    
    let stdout = StdoutOutput::new(OutputFormat::Json);
    let composite = CompositeOutput::new(vec![Arc::new(stdout)]);
    
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    
    let change = Change::Insert {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple,
    };
    
    composite.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_composite_output_with_multiple_targets() {
    use std::sync::Arc;
    
    let stdout1 = StdoutOutput::new(OutputFormat::Json);
    let stdout2 = StdoutOutput::new(OutputFormat::Text);
    let composite = CompositeOutput::new(vec![
        Arc::new(stdout1),
        Arc::new(stdout2),
    ]);
    
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    
    let change = Change::Insert {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple,
    };
    
    composite.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_composite_output_empty_targets() {
    let composite = CompositeOutput::new(vec![]);
    
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    
    let change = Change::Insert {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple,
    };
    
    // Should not panic with no targets
    composite.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_full_transaction_flow_through_output() {
    let output = StdoutOutput::new(OutputFormat::Json);
    
    // Begin transaction
    let begin = Change::Begin {
        lsn: "0/100".to_string(),
        timestamp: 1234567890,
        xid: 500,
    };
    output.write_change(&begin).await.unwrap();
    
    // Relation metadata
    let columns = vec![
        ColumnInfo {
            name: "id".to_string(),
            type_id: 23,
            flags: 1,
        },
    ];
    let relation = Change::Relation {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "test_table".to_string(),
        columns,
    };
    output.write_change(&relation).await.unwrap();
    
    // Insert
    let mut insert_tuple = HashMap::new();
    insert_tuple.insert("id".to_string(), Some("1".to_string()));
    let insert = Change::Insert {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "test_table".to_string(),
        new_tuple: insert_tuple,
    };
    output.write_change(&insert).await.unwrap();
    
    // Update
    let mut old_tuple = HashMap::new();
    old_tuple.insert("id".to_string(), Some("1".to_string()));
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("2".to_string()));
    let update = Change::Update {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "test_table".to_string(),
        old_tuple: Some(old_tuple),
        new_tuple,
    };
    output.write_change(&update).await.unwrap();
    
    // Delete
    let mut delete_tuple = HashMap::new();
    delete_tuple.insert("id".to_string(), Some("2".to_string()));
    let delete = Change::Delete {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "test_table".to_string(),
        old_tuple: delete_tuple,
    };
    output.write_change(&delete).await.unwrap();
    
    // Commit transaction
    let commit = Change::Commit {
        lsn: "0/200".to_string(),
        timestamp: 1234567900,
    };
    output.write_change(&commit).await.unwrap();
}

#[tokio::test]
async fn test_stdout_output_with_null_values() {
    let output = StdoutOutput::new(OutputFormat::Json);
    
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    new_tuple.insert("email".to_string(), None);
    new_tuple.insert("name".to_string(), Some("Alice".to_string()));
    
    let change = Change::Insert {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple,
    };
    
    output.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_stdout_output_with_special_schema_names() {
    let output = StdoutOutput::new(OutputFormat::Json);
    
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    
    let change = Change::Insert {
        relation_id: 16384,
        schema: "my-custom_schema".to_string(),
        table: "test_table".to_string(),
        new_tuple,
    };
    
    output.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_stdout_output_text_format() {
    let output = StdoutOutput::new(OutputFormat::Text);
    
    let mut new_tuple = HashMap::new();
    new_tuple.insert("id".to_string(), Some("1".to_string()));
    new_tuple.insert("name".to_string(), Some("Test User".to_string()));
    
    let change = Change::Insert {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple,
    };
    
    // Text format should not panic
    output.write_change(&change).await.unwrap();
}

#[tokio::test]
async fn test_multiple_inserts_through_composite() {
    use std::sync::Arc;
    
    let output1 = StdoutOutput::new(OutputFormat::Json);
    let output2 = StdoutOutput::new(OutputFormat::JsonPretty);
    let composite = CompositeOutput::new(vec![
        Arc::new(output1),
        Arc::new(output2),
    ]);
    
    // First insert
    let mut tuple1 = HashMap::new();
    tuple1.insert("id".to_string(), Some("1".to_string()));
    let change1 = Change::Insert {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "users".to_string(),
        new_tuple: tuple1,
    };
    composite.write_change(&change1).await.unwrap();
    
    // Second insert
    let mut tuple2 = HashMap::new();
    tuple2.insert("id".to_string(), Some("2".to_string()));
    let change2 = Change::Insert {
        relation_id: 16384,
        schema: "public".to_string(),
        table: "orders".to_string(),
        new_tuple: tuple2,
    };
    composite.write_change(&change2).await.unwrap();
}
