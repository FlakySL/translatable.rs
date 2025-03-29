use translatable::translation;

fn main() {

    println!("{:?}", std::process::Command::new("pwd").output().unwrap());

    let a_lang = "es";
    let a_path = "common.greeting";
    let a_name = "john";

    // translation!(a_lang, a_path, name = a_name);
    // !! https://github.com/dtolnay/trybuild/issues/202
    assert!(translation!(a_lang, a_path, name = a_name).unwrap() == "¡Hola john!".into());

    let b_lang = "en";
    let b_path = "common.greeting";
    let b_name = "Marie";

    translation!(b_lang, b_path, name = b_name);

}
