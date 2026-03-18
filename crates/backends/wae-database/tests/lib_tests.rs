use wae_database::{ColumnDef, ColumnType, TableSchema};

#[test]
fn test_column_def() {
    let col = ColumnDef::new("id", ColumnType::Integer);
    assert_eq!(col.name, "id");
}

#[test]
fn test_table_schema() {
    let table = TableSchema::new("users");
    assert_eq!(table.name, "users");
}
