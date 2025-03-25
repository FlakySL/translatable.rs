use translatable::translation;

#[test]
fn both_static() {
    let result = translation!("es", static salutation::test);

    assert!(result == "Hola")
}

#[test]
fn language_static_path_dynamic() {
    let result = translation!("es", "salutation.test");

    assert!(result.unwrap() == "Hola".to_string())
}

#[test]
fn language_dynamic_path_static() {
    let language = "es";
    let result = translation!(language, static salutation::test);

    assert!(result.unwrap() == "Hola".to_string())
}

#[test]
fn both_dynamic() {
    let language = "es";
    let result = translation!(language, "salutation.test");

    assert!(result.unwrap() == "Hola".to_string())
}
