use translatable::translation;

fn main() {
    let name = "Juan";
    let res = translation!("en", static common::greeting, name);
    assert_eq!(res, "Hello Juan!");
}
