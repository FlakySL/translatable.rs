use trybuild::TestCases;

#[test]
fn static_tests() {
    let t = TestCases::new();

    // Compile-time tests for invalid translations
    t.compile_fail("tests/static/invalid_args.rs");
    t.compile_fail("tests/static/invalid_syntax.rs");
    t.compile_fail("tests/static/missing_args.rs");
}
