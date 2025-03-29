use translatable::translation;

fn main() {
    // Missing arguments
    let _ = translation!("es");

    // Invalid literal type
    let _ = translation!(42, static common::greeting);

    // Malformed static path
    let _ = translation!("es", static invalid::path);
}
