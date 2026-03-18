use wae_database::{TableSchema, col};
use wae_tools::auto_migrate::{AutoMigrator, MigrationPlan};

#[test]
fn test_auto_migrator_registration() {
    let mut migrator = AutoMigrator::new();

    let user_schema = TableSchema::new("users")
        .column(col::id("id"))
        .column(col::text("name").not_null())
        .column(col::text("email").unique());

    migrator.register(user_schema);

    assert_eq!(migrator.schemas().len(), 1);
    assert!(migrator.schemas().contains_key("users"));
}

#[test]
fn test_migration_plan() {
    let plan = MigrationPlan::empty();
    assert!(plan.is_empty());
    assert_eq!(plan.sql_count(), 0);
}
