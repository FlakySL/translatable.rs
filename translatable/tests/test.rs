use translatable::{Language, translation};

#[test]
fn both_static() {
    let result = translation!("es", static common::greeting, name = "john");

    assert!(result == "¡Hola john!")
}

#[test]
fn language_static_path_dynamic() {
    let result = translation!("es", vec!["common", "greeting"], name = "john");

    assert!(result.unwrap() == "¡Hola john!".to_string())
}

#[test]
fn language_dynamic_path_static() {
    let name = "john";
    let result = translation!(Language::ES, static common::greeting, name);

    assert!(result.unwrap() == "¡Hola john!".to_string())
}

#[test]
fn both_dynamic() {
    let result = translation!(Language::ES, vec!["common", "greeting"], name = "john");
    assert!(result.unwrap() == "¡Hola john!".to_string())
}
