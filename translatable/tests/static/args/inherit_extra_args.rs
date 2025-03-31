use translatable::translation;

fn main() {
    let name = "Juan";
    let surname = "Doe";

    let res = translation!("en", static common::greeting, name, surname);
    assert_eq!(res, "Hello Juan!");
}
