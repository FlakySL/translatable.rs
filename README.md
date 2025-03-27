# Translatable 🌐🗣️💬🌍

[![Crates.io](https://img.shields.io/crates/v/translatable)](https://crates.io/crates/translatable)
[![Docs.rs](https://docs.rs/translatable/badge.svg)](https://docs.rs/translatable)

A robust internationalization solution for Rust featuring compile-time validation, ISO 639-1 compliance, and TOML-based translation management.

## Table of Contents 📖

- [Features](#features-)
- [Installation](#installation-)
- [Usage](#usage-)
- [Example implementation](#example-implementation-)
- [Licensing](#license-)

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

### Configuration

There are things you can configure on how translations are loaded from the folder, for this
you should make a `translatable.toml` in the root of the project, and abide by the following
configuration values.

| Key       | Value type                         | Description                                                                                                                    |
|-----------|------------------------------------|--------------------------------------------------------------------------------------------------------------------------------|
| `path`      | `String`                             | Where the translation files will be stored, non translation files in that folder will cause errors.                            |
| `seek_mode` | `"alphabetical"` \| `"unalphabetical"` | The found translations are ordered by file name, based on this field.                                                          |
| `overlap`   | `"overwrite"` \| `"ignore"`            | Orderly if a translation is found `"overwrite"` will keep searching for translations and `"ignore"` will preserve the current one. |

`seek_mode` and `overlap` only reverse the translations as convenient, this way the process
doesn't get repeated every time a translation is loaded.

### Translation file format

All the translation files are going to be loaded from the path specified in the configuration,
all the files inside the path must be TOML files and sub folders, a `walk_dir` algorithm is used
to load all the translations inside that folder.

The translation files have three rules
- Objects (including top level) can only contain objects and strings
- If an object contains another object, it can only contain other objects (known as nested object)
- If an object contains a string, it can only contain other strings (known as translation object)

### Loading translations

The load configuration such as `seek_mode` and `overlap` is not relevant here, as previously
specified, these configuration values only get applied once by reversing the translations conveniently.

To load translations you make use of the `translatable::translation` macro, that macro requires two
parameters to be passed.

The first parameter consists of the language which can be passed dynamically as a variable or an expression
that resolves to an `impl Into<String>`, or statically as a `&'static str` literal. Not mattering the way
it's passed, the translation must comply with the `ISO 639-1` standard.

The second parameter consists of the path, which can be passed dynamically as a variable or an expression
that resolves to an `impl Into<String>` with the format `path.to.translation`, or statically with the following
syntax `static path::to::translation`.

Depending on whether the parameters are static or dynamic the macro will act different, differing whether
the checks are compile-time or run-time, the following table is a macro behavior matrix.

| Parameters                                         | Compile-Time checks                                      | Return type                                                                       |
|----------------------------------------------------|----------------------------------------------------------|-----------------------------------------------------------------------------------|
| `static language` + `static path` (most optimized) | Path existence, Language validity, \*Template validation | `&'static str` (stack) if there are no templates or `String` (heap) if there are. |
| `dynamic language` + `dynamic path`                | None                                                     | `Result<String, TranslatableError>` (heap)                                        |
| `static language` + `dynamic path`                 | Language validity                                        | `Result<String, TranslatableError>` (heap)                                        |
| `dynamic language` + `static path` (commonly used) | Path existence, \*Template validation                    | `Result<String, TranslatableError>` (heap)                                        |

- For the error handling, if you want to integrate this with `thiserror` you can use a `#[from] translatable::TranslationError`,
as a nested error, all the errors implement display, for optimization purposes there are not the same amount of errors with
dynamic parameters than there are with static parameters.

- The runtime errors implement a `cause()` method that returns a heap allocated `String` with the error reason, essentially
the error display.

- Template validation in the static parameter handling means variable existence, since templates are generated as a `format!`
call which processes expressions found in scope. It's always recommended to use full paths in translation templates
to avoid needing to make variables in scope, unless the calls are contextual, in that case there is nothing that can
be done to avoid making variables.

## Example implementation 📂

The following examples are an example application structure for a possible
real project.

### Example application tree

```plain
project-root/
├── Cargo.toml
├── translatable.toml
├── translations/
│    └── app.toml
└── src/
     └── main.rs
```

### Example translation file (translations/app.toml)

Notice how `common.greeting` has a template named `name`.

```toml
[welcome_message]
en = "Welcome to our app!"
es = "¡Bienvenido a nuestra aplicación!"

[common.greeting]
en = "Hello {name}!"
es = "¡Hola {name}!"
```

### Example application usage

Notice how that template is in scope, whole expressions can be used
in the templates such as `path::to::function()`, or other constants.

```rust
extern crate translatable;
use translatable::translation;

fn main() {
	let dynamic_lang = "es";
	let dynamic_path = "common.greeting"
	let name = "john";

	assert!(translation!("es", static common::greeting) == "¡Hola john!");
	assert!(translation!("es", dynamic_path).unwrap() == "¡Hola john!".into());
	assert!(translation!(dynamic_lang, static common::greeting).unwrap() == "¡Hola john!".into());
	assert!(translation!(dynamic_lang, dynamic_path).unwrap() == "¡Hola john!".into());
}
```

## License 📜

<sup>
This repository is licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a>
or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<sub>
Unless you explicitly state any contribution intentionally submitted
for inclusion in translatable by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
