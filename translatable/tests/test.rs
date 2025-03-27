use translatable::translation;

#[test]
fn both_static() {
    let name = "john";
    let result = translation!("es", static common::greeting);

    assert!(result == "¡Hola john!")
}

#[test]
fn language_static_path_dynamic() {
    let name = "john";
    let result = translation!("es", "common.greeting");

    assert!(result.unwrap() == "¡Hola john!".to_string())
}

#[test]
fn language_dynamic_path_static() {
    let name = "john";
    let language = "es";
    let result = translation!(language, static common::greeting);

    assert!(result.unwrap() == "¡Hola john!".to_string())
}

#[test]
fn both_dynamic() {
    let name = "john";
    let language = "es";
    let result = translation!(language, "common.greeting");

    assert!(result.unwrap() == "¡Hola john!".to_string())
}
