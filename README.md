# Translatable

[![Crates.io](https://img.shields.io/crates/v/translatable)](https://crates.io/crates/translatable)
[![Docs.rs](https://docs.rs/translatable/badge.svg)](https://docs.rs/translatable)

A robust internationalization solution for Rust featuring compile-time validation, ISO 639-1 compliance, and TOML-based translation management.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Error Handling](#error-handling)
- [Example Structure](#example-structure)

## Features 🚀

- **ISO 639-1 Standard**: Full support for 180+ language codes/names
- **Compile-Time Safety**: Macro-based translation validation
- **TOML Structure**: Hierarchical translation files with nesting
- **Smart Error Messages**: Context-aware suggestions
- **Template Validation**: Balanced bracket checking
- **Flexible Loading**: Configurable file processing order
- **Conflict Resolution**: Choose between first or last match priority

## Installation 📦

Add to your `Cargo.toml`:

```toml
[dependencies]
translatable = "1"
```

## Usage

Basic Macro Usage

```rust
use translatable::translation;

fn main() {
    let greeting = translation!("en", static common.greeting);
    println!("{}", greeting);
}
```

Translation File Example (`translations/app.toml`)

```toml
[home]
welcome_message = {
    en = "Welcome to our app!",
    es = "¡Bienvenido a nuestra aplicación!"
}

[user]
greeting = {
    en = "Hello {name}!",
    es = "¡Hola {name}!"
}
```

## Configuration ⚙️

Create translatable.toml in your project root:

```toml
path = "./translations"
seek_mode = "alphabetical"
overlap = "overwrite"
```

| Option    | Default        | Description                       |
| --------- | -------------- | --------------------------------- |
| path      | ./translations | Translation directory location    |
| seek_mode | alphabetical   | File processing order             |
| overlap   | overwrite      | Last file priority vs first found |

## Error Handling 🚨

Invalid Language Code

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
```

Structural Validation

```sh
Error: Invalid TOML structure in file ./translations/test.toml: Translation files must contain either nested tables or language translations, but not both at the same level.
```

Template Validation

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
        └── settings.toml
```
