use std::{fs::{read_dir, read_to_string}, io::Error as IoError, sync::OnceLock};
use proc_macro::TokenStream;
use strum::IntoEnumIterator;
use thiserror::Error;
use toml::{de::Error as TomlError, Table, Value};
use crate::{config::{load_config, ConfigError, SeekMode, TranslationOverlap}, macros::{TranslationLanguageType, TranslationPathType}, languages::Iso639a};

#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("{0}")]
    Config(#[from] ConfigError),

    #[error("An IO Error occurred: {0:#}")]
    Io(#[from] IoError),

    #[error("The path contains invalid unicode characters.")]
    InvalidUnicode,

    #[error(
        "Toml parse error '{}'{}",
        .0.message(),
        .0.span()
            .map(|l| format!(" in {}:{}:{}", .1, l.start, l.end))
            .unwrap_or("".into())
    )]
    ParseToml(TomlError, String),

    #[error(
        "'{0}' is not valid ISO 639-1, valid languages including '{0}' are:\n{valid}",
        valid = Iso639a::iter()
            .filter(|lang| format!("{lang:?}")
                .to_lowercase()
                .contains(&.0.to_lowercase()
            ))
            .map(|lang| format!("{} ({lang:#})", format!("{lang:?}").to_lowercase()))
            .collect::<Vec<_>>()
            .join(",\n")
    )]
    InvalidLanguage(String)

    #[error("{}", "
Translation files can only contain objects,
objects in objects, if an object contains a string,
all it's other branches should also be strings
where it's keys are valid ISO 639-1 languages
in lowercase.
    ".trim())]
    InvalidTomlFormat
}

static TRANSLATIONS: OnceLock<Vec<Table>> = OnceLock::new();

fn walk_dir(path: &str) -> Result<Vec<String>, TranslationError> {
    let directory = read_dir(path)?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    let mut result = Vec::new();

    for path in directory {
        let path = path.path();

        if path.is_dir() {
            result.extend(walk_dir(
                path
                    .to_str()
                    .ok_or(TranslationError::InvalidUnicode)?
            )?);
        } else {
            result.push(
                path
                    .to_string_lossy()
                    .to_string()
            );
        }
    }

    Ok(result)
}

fn translations_valid(table: &Table) -> bool {
    let mut contains_translation = false;
    let mut contains_table = false;

    for (key, raw) in table {
        match raw {
            Value::Table(table) => {
                // if the current nesting contains a translation
                // it can't contain anything else, thus invalid.
                if contains_translation {
                    return false;
                }

                // if the value is a table call the function recursively.
                // if the nesting it's invalid it invalidates the whole file.
                if !translations_valid(table) {
                    return false;
                }

                // since it passes the validation and it's inside the Table match
                // it contains a table.
                contains_table = true;
            },

            Value::String(translation) => {
                // if the current nesting contains a table
                // it can't contain anything else, thus invalid.
                if contains_table {
                    return false;
                }

                // an object that contains translations
                // can't contain an invalid key.
                if !Iso639a::is_valid(&key) {
                    return false;
                }

                // a translation can't contain unclosed
                // delimiters for '{' and '}', because
                // these are used for templating.
                let mut open_templates = 0i32;

                for c in translation.chars() {
                    match c {
                        '{' => open_templates += 1,
                        '}' => open_templates -= 1,
                        _ => {}
                    }
                }

                if open_templates != 0 {
                    return false;
                }

                // if all the checks above pass
                // it means the table defintively
                // contains a translation.
                contains_translation = true;
            },

            // if the table contains anything else than
            // a translation (string) or a nested table
            // it's automatically invalid.
            _ => return false
        }
    }

    // if nothing returns false it means everything
    // is valid.
    true
}

fn load_translations() -> Result<&'static Vec<Table>, TranslationError> {
    if let Some(translations) = TRANSLATIONS.get() {
        return Ok(translations);
    }

    let config = load_config()?;

    let mut translation_paths = walk_dir(config.path())?;
    translation_paths.sort_by_key(|path| path.to_lowercase());

    if let SeekMode::Unalphabetical = config.seek_mode() {
        translation_paths.reverse();
    }

    let translations = translation_paths
        .iter()
        .map(|path| Ok(
            read_to_string(&path)?
                .parse::<Table>()
                .map_err(|err| TranslationError::ParseToml(err, path.clone()))
                .and_then(|table| translations_valid(&table)
                    .then_some(table)
                    .ok_or(TranslationError::InvalidTomlFormat)
                )?
        ))
        .collect::<Result<Vec<_>, TranslationError>>()?;

    Ok(TRANSLATIONS.get_or_init(|| translations))
}

pub fn load_translation_static(lang: &str, path: &str) -> Result<Option<String>, TranslationError> {
    let translations = load_translations()?;
    let config = load_config()?;

    if !Iso639a::is_valid(lang) {
        return Err(TranslationError::InvalidLanguage(lang.into()))
    }

    let mut choosen_translation = None;
    for translation in translations {
        choosen_translation = path
            .split('.')
            .fold(Some(translation), |acc, key| acc?.get(key)?.as_table())
            .and_then(|translation| translation.get(lang))
            .map(|translation| translation.to_string());

        if choosen_translation.is_some() && matches!(config.overlap(), TranslationOverlap::Ignore) {
            break;
        }
    }

    Ok(choosen_translation)
}

pub fn load_translation_dynamic(lang: TranslationLanguageType, path: TranslationPathType) -> TokenStream {
    let lang = lang.dynamic();
    let path = path.dynamic();

    todo!()
}
