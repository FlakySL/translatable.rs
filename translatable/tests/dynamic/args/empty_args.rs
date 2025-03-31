use translatable::translation;

fn main() {
    let lang = "en";
    let path = "common.greeting";

    let res = translation!(lang, path).unwrap();
    assert_eq!(res, "Hello {name}!");
}
