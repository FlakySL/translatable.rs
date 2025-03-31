use trybuild::TestCases;

#[test]
fn static_tests() {
    let t = TestCases::new();

    // implementation
    t.compile_fail("tests/static/invalid_syntax.rs");

    // args
    t.pass("tests/static/args/empty_args.rs");
    t.pass("tests/static/args/extra_args.rs");
    t.pass("tests/static/args/inherit_args.rs");
    t.pass("tests/static/args/inherit_extra_args.rs");
    t.compile_fail("tests/static/args/invalid_identifier.rs");
    t.pass("tests/static/args/with_args.rs");
}

#[test]
fn dynamic_tests() {
    let t = TestCases::new();

    // implementation
    t.compile_fail("tests/dynamic/invalid_syntax.rs");

    // args
    t.pass("tests/dynamic/args/empty_args.rs");
    t.pass("tests/dynamic/args/extra_args.rs");
    t.pass("tests/dynamic/args/inherit_args.rs");
    t.pass("tests/dynamic/args/inherit_extra_args.rs");
    t.compile_fail("tests/dynamic/args/invalid_identifier.rs");
    t.pass("tests/dynamic/args/with_args.rs");
}
