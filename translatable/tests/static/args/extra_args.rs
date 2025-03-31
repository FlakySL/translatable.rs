use translatable::translation;

fn main() {
    let res = translation!("en", static common::greeting, name = "Juan", surname = "Doe");
    assert_eq!(res, "Hello Juan!");
}
