#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/correct-usage.rs");
    t.compile_fail("tests/ui/*.rs");
}
