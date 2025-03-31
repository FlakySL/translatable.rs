use translatable::translation;

fn main() {
    let lang = "en";
    let path = "common.greeting";

    // Invalid argument syntax
    translation!(lang, path, 42 = "value");
}
