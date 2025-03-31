use translatable::translation;

fn main() {
    let res = translation!("en", static common::greeting);
    assert_eq!(res, "Hello {name}!");
}
