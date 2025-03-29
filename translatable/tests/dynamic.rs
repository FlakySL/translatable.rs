use trybuild::TestCases;

#[test]
fn dynamic_tests() {
    let t = TestCases::new();

    println!("{:?}", std::process::Command::new("pwd").output().unwrap());

    // Compile-time tests for 
    t.pass("tests/dynamic/valid_syntax.rs");

}
