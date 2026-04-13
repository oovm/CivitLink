use gg_runtime_ui::parse_vx;

#[test]
fn test_parse_template_only() {
    let doc = parse_vx("<template><div>Hello</div></template>").unwrap();
    assert!(doc.template.is_some());
    assert!(doc.script.is_none());
    assert!(doc.style.is_none());
}

#[test]
fn test_parse_full_vx_file() {
    let source = "\
<template>
  <Layout style=\"flex-1\">
    <Text class=\"title\">Hello GG Editor</Text>
  </Layout>
</template>

<script>
  using gg_editor::ui::widgets::{Layout, Text};
</script>

<style>
  .title {
    color: #fff;
  }
</style>";
    let doc = parse_vx(source).unwrap();
    assert!(doc.template.is_some());
    assert!(doc.script.is_some());
    assert!(doc.style.is_some());
}
