use translatable::translation;

fn main() {
    let a_lang = "es";
    let a_path = "common.greeting";
    let a_name = "john";

    let _ = translation!(a_lang, a_path, name = a_name);

    let b_lang = "en";
    let b_path = "common.greeting";
    let b_name = "Marie";

    let _ = translation!(b_lang, b_path, name = b_name);
}
