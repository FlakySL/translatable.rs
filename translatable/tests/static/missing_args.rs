use translatable::translation;

fn main() {
    // Test completely empty invocation
    let _ = translation!();

    // Missing path argument
    let _ = translation!("es");

    // Missing language argument
    let _ = translation!(static common::greeting);

    // Missing both language and path
    let _ = translation!();

    // Missing interpolation arguments
    let _ = translation!("es", static common::greeting);

    // Partial arguments with named params
    let _ = translation!("es", static common::greeting, name);
}
