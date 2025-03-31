use translatable::translation;

fn main() {
    // Missing arguments
    let lang = "es";
    let _ = translation!(lang);

    // Invalid literal type
    let lang = 42;
    let path = "common.greeting";
    let _ = translation!(lang, path);

    // Malformed dynamic path
    // TODO: I'm 100% sure that this is a bug.
    let lang = "es";
    let path = "invalid.path";
    assert!(translation!(lang, path).is_err());
}
