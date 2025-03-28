use translatable::translation;

#[test]
fn both_static() {
    let result = translation!("es", static common::greeting, name = "john");

    assert!(result == "¡Hola john!")
}

#[test]
fn language_static_path_dynamic() {
    let result = translation!("es", "common.greeting", name = "john");

    assert!(result.unwrap() == "¡Hola john!".to_string())
}

#[test]
fn language_dynamic_path_static() {
    let language = "es";
    let name = "john";
    let result = translation!(language, static common::greeting, name = name);

    assert!(result.unwrap() == "¡Hola john!".to_string())
}

#[test]
fn both_dynamic() {
    let language = "es";
    let result = translation!(language, "common.greeting", lol = 10, name = "john");

    assert!(result.unwrap() == "¡Hola john!".to_string())
}
