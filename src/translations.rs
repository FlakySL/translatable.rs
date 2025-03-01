use std::{fs::{read_dir, read_to_string}, io::Error as IoError, sync::OnceLock};
use proc_macro::TokenStream;
use thiserror::Error;
use toml::{Table, de::Error as TomlError};
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
        .0.span().map(|l| format!(" in {}:{}:{}", .1, l.start, l.end)).unwrap_or("".into())
    )]
    ParseToml(TomlError, String),

    #[error(
        "'{0}' is not valid ISO 639-1, valid languages include: {valid}",
        valid = Iso639a::languages().join(", ")
    )]
    InvalidLangauge(String)
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
                .map_err(|err| TranslationError::ParseToml(err, path.clone()))?
        ))
        .collect::<Result<Vec<_>, TranslationError>>()?;

    Ok(TRANSLATIONS.get_or_init(|| translations))
}

pub fn load_translation_static(lang: &str, path: &str) -> Result<Option<String>, TranslationError> {
    let translations = load_translations()?;
    let config = load_config()?;

    if !Iso639a::is_valid(lang) {
        return Err(TranslationError::InvalidLangauge(lang.into()))
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
