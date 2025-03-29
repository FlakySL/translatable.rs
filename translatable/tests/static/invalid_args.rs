use translatable::translation;

fn main() {
    // Missing required arguments
    translation!("es");

    // Invalid argument syntax
    translation!("es", "path", 42 = "value");
}
