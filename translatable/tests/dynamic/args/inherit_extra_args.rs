use translatable::translation;

fn main() {
    let lang = "en";
    let path = "common.greeting";
    let name = "Juan";
    let surname = "Doe";

    let res = translation!(lang, path, name, surname).unwrap();
    assert_eq!(res, "Hello Juan!");
}
