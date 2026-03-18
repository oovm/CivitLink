use wae_tools::migration::*;

#[test]
fn test_simple_migration_creation() {
    let migration = SimpleMigration::new(1, "create_users_table", "CREATE TABLE users (id INTEGER PRIMARY KEY)")
        .with_description("Create users table")
        .with_down_sql("DROP TABLE users");

    assert_eq!(migration.version(), 1);
    assert_eq!(migration.name(), "create_users_table");
    assert_eq!(migration.description(), Some("Create users table"));
    assert!(migration.reversible());
}

#[test]
fn test_migrator_registration() {
    let mut migrator = Migrator::new();

    migrator.register(SimpleMigration::new(1, "first", "SQL1")).register(SimpleMigration::new(2, "second", "SQL2"));

    assert_eq!(migrator.migrations().len(), 2);
    assert_eq!(migrator.latest_version(), Some(2));
}
