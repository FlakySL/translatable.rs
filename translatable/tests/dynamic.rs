use trybuild::TestCases;

#[test]
fn dynamic_tests() {
    let t = TestCases::new();

    // Compile-time tests for
    t.pass("tests/dynamic/valid_syntax.rs");
}
