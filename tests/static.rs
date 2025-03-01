use translatable::translation;

#[test]
fn get_salutation() {
    translation!("en", static salutation::test);
}
