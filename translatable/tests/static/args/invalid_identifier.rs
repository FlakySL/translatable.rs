use translatable::translation;

fn main() {
    // Invalid argument syntax
    translation!("es", static common::greeting, 42 = "value");
}
