use translatable::translation;

fn main() {
    let lang = "en";
    let path = "common.greeting";

    let res = translation!(lang, path, name = "Juan", surname = "Doe").unwrap();
    assert_eq!(res, "Hello Juan!");
}
