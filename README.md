# Translatable

[![Crates.io](https://img.shields.io/crates/v/translatable)](https://crates.io/crates/translatable)
[![Docs.rs](https://docs.rs/translatable/badge.svg)](https://docs.rs/translatable)

A robust internationalization solution for Rust featuring compile-time validation, ISO 639-1 compliance, and TOML-based translation management.

## Table of Contents

- [Features](#features-🚀)
- [Installation](#installation-📦)
- [Usage](#usage-🛠️)
- [Configuration](#configuration-⚙️)
- [Error Handling](#error-handling-🚨)
- [Example Structure](#example-structure-📂)
- [Integration Guide](#integration-guide-🔗)

## Features 🚀

- **ISO 639-1 Standard**: Full support for 180+ language codes/names
- **Compile-Time Safety**: Macro-based translation validation
- **TOML Structure**: Hierarchical translation files with nesting
- **Smart Error Messages**: Context-aware suggestions
- **Template Validation**: Balanced bracket checking
- **Flexible Loading**: Configurable file processing order
- **Conflict Resolution**: Choose between first or last match priority

## Installation 📦

Run the following command in your project directory:

```sh
cargo add translatable
```

## Usage 🛠️

### Macro Behavior Matrix

| Parameters                        | Compile-Time Checks                                              | Return Type                              |
| --------------------------------- | ---------------------------------------------------------------- | ---------------------------------------- |
| `static path` + `static language` | - Path existence<br>- Language validity<br>- Template validation | `&'static str`                           |
| `static path` + dynamic language  | - Path existence<br>- Template structure                         | `Result<&'static str, TranslationError>` |
| dynamic path + `static language`  | - Language validity                                              | `Result<&'static str, TranslationError>` |
| dynamic path + dynamic language   | None (runtime checks only)                                       | `Result<&'static str, TranslationError>` |

### Key Implications

- **Static Path**  
  ✅ Verifies translation path exists in TOML files  
  ❌ Requires path literal (e.g., `static common::greeting`)

- **Static Language**  
  ✅ Validates ISO 639-1 compliance  
  ❌ Requires language literal (e.g., `"en"`)

- **Mixed Modes**

  ```rust
  // Compile-time path + runtime language
  translation!(user_lang, static user::profile::title)

  // Compile-time language + runtime path
  translation!("fr", dynamic_path)
  ```

- **Full Dynamic**

  ```rust
  // Runtime checks only
  translation!(lang_var, path_var) // Returns Result
  ```

- **Full Static**

  ```rust
  // Compile-time checks only
  translation!("en", static common::greeting) // Returns &'static str
  ```

Optimization Guide

```rust
// Maximum safety - fails compile if any issues
let text = translation!("es", static home::welcome_message);

// Balanced approach - compile-time path validation
let result = translation!(user_lang, static user::profile::title);

// Flexible runtime - handles dynamic inputs
let result = translation!(lang_var, path_var)?;
```

## Example Structure 📂

```txt
project-root/
├── Cargo.toml
├── translatable.toml
└── translations/
    ├── app.toml
    ├── errors.toml
    └── user/
        ├── profile.toml
```

### Example Translation File (translations/app.toml)

```toml
[home]
welcome_message = {
    en = "Welcome to our app!",
    es = "¡Bienvenido a nuestra aplicación!"
}

[common]
greeting = {
    en = "Hello {name}!",
    es = "¡Hola {name}!"
}
```

### Translation File Organization

The `translations/` folder can be structured flexibly. You can organize translations based on features, modules, or locales.
Here are some best practices:

- Keep related translations in subdirectories (`user/profile.toml`, `errors.toml`)
- Use consistent naming conventions (`common.toml`, `app.toml`)
- Keep files small and manageable to avoid conflicts

## Configuration ⚙️

Create `translatable.toml` in your project root:

```toml
path = "./translations"
seek_mode = "alphabetical"
overlap = "overwrite"
```

| Option    | Default        | Description                                 |
| --------- | -------------- | ------------------------------------------- |
| path      | ./translations | Translation directory location              |
| seek_mode | alphabetical   | Order in which translation files are loaded |
| overlap   | overwrite      | Defines conflict resolution strategy        |

### Configuration Options Explained

- **`seek_mode`**: Controls the order of file processing (e.g., `alphabetical`, `manual`).
- **`overlap`**: Determines priority when duplicate keys exist (`overwrite` replaces existing keys, `first` keeps the first occurrence).

## Error Handling 🚨

### Invalid Language Code

```sh
Error: 'e' is not valid ISO 639-1. These are some valid languages including 'e':
          ae (Avestan),
          eu (Basque),
          be (Belarusian),
          ce (Chechen),
          en (English),
          ... (12 more)
    --> tests/static.rs:5:5
     |
   5 |     translation!("e", static salutation::test);
     |
     = note: this error originates in the macro `translation` (in Nightly builds, run with -Z macro-backtrace for more info)
```

### Structural Validation

```sh
Error: Invalid TOML structure in file ./translations/test.toml: Translation files must contain either nested tables or language translations, but not both at the same level.
```

### Template Validation

```sh
Error: Toml parse error 'invalid inline table
          expected `}`' in ./translations/test.toml:49:50
    --> tests/static.rs:5:5
     |
   5 |     translation!("es", static salutation::test);
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
     = note: this error originates in the macro `translation` (in Nightly builds, run with -Z macro-backtrace for more info)
```

## Integration Guide 🔗

If you're using `translatable` in a web application, here’s how to integrate it:

### Actix-Web Example

```rust
use actix_web::{get, web, App, HttpServer, Responder};
use translatable::translation;

#[get("/")]
async fn home() -> impl Responder {
    let text = translation!("en", static home::welcome_message);
    text.to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(home))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
```
