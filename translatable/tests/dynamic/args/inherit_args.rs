use translatable::translation;

fn main() {
    let lang = "en";
    let path = "common.greeting";
    let name = "Juan";

    let res = translation!(lang, path, name).unwrap();
    assert_eq!(res, "Hello Juan!");
}
