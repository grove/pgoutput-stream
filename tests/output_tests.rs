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
