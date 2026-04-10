use gg_factory::generator::{name_to_package_name, name_to_struct_name};
#[test]
fn test_name_to_struct_name_spaces() {
    assert_eq!(name_to_struct_name("My Galgame"), "MyGalgameEngine");
}
#[test]
fn test_name_to_struct_name_hyphens() {
    assert_eq!(name_to_struct_name("my-galgame"), "MyGalgameEngine");
}
#[test]
fn test_name_to_struct_name_underscores() {
    assert_eq!(name_to_struct_name("my_galgame"), "MyGalgameEngine");
}
#[test]
fn test_name_to_struct_name_single_word() {
    assert_eq!(name_to_struct_name("Galgame"), "GalgameEngine");
}
#[test]
fn test_name_to_package_name() {
    assert_eq!(name_to_package_name("My Galgame"), "my-galgame");
}
#[test]
fn test_name_to_package_name_single() {
    assert_eq!(name_to_package_name("Galgame"), "galgame");
}
