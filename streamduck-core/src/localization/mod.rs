use std::collections::HashMap;
use std::sync::Arc;
use futures::future::join_all;
use serde_json::Value;
use tokio::sync::RwLock;

use serde::{Serialize, Deserialize};

/// Language tag for English language, used as fallback
pub const ENGLISH_LANG_TAG: &'static str = "en";

/// Manages localizations of the software
pub struct LocalizationManager {
    localizations: RwLock<HashMap<String, Arc<Localization>>>,
}

impl LocalizationManager {
    /// Creates new instance of the manager
    pub fn new() -> Arc<LocalizationManager> {
        Arc::new(Self {
            localizations: Default::default()
        })
    }

    /// Retrieves localization if it exists
    pub async fn get(&self, language_tag: &str) -> Option<Arc<Localization>> {
        self.localizations.read().await
            .get(language_tag).cloned()
    }

    /// Attempts to translate the localized string using preferred language, falls back to [English](ENGLISH_LANG_TAG) if can't
    pub async fn translate(&self, preferred_language: &str, l_str: &LocalizedString) -> Option<String> {
        if let Some(localization) = self.get(preferred_language).await {
            if let Some(translation) = localization.translate(l_str).await {
                return Some(translation);
            }
        }

        Some(self.get(ENGLISH_LANG_TAG).await?.translate(l_str).await?)
    }

    /// Inserts localization into the manager using the language tag, overrides if already exists!
    ///
    /// Please use [IETF BCP 47 language tags](https://en.wikipedia.org/wiki/IETF_language_tag) for consistency
    pub async fn insert(&self, language_tag: &str, localization: Arc<Localization>) {
        self.localizations.write().await
            .insert(language_tag.to_string(), localization);
    }

    /// Retrieves localization if it exists, or inserts if it doesn't
    ///
    /// Please use [IETF BCP 47 language tags](https://en.wikipedia.org/wiki/IETF_language_tag) for consistency
    pub async fn get_or_insert<F>(&self, language_tag: &str, localization: F) -> Arc<Localization>
        where F: FnOnce() -> Arc<Localization> {
        if let Some(localization) = self.get(language_tag).await {
            localization
        } else {
            let localization = localization();
            self.insert(language_tag, localization.clone()).await;
            localization
        }
    }

    /// Retrieves a map of language tag to localization
    pub async fn language_map(&self) -> HashMap<String, Arc<Localization>> {
        self.localizations.read().await
            .clone()
    }

    /// Retrieves complete translation map, language tags to translation maps
    pub async fn translation_map(&self) -> HashMap<String, HashMap<String, String>> {
        join_all(self.language_map().await
            .into_iter()
            .map(|(k, l)| {
                async move { (k, l.translation_map().await) }
            })
        ).await.into_iter().collect()
    }

    /// Retrieves all the languages that the manager has
    pub async fn languages(&self) -> Vec<String> {
        self.localizations.read().await
            .keys().cloned().collect()
    }

    /// Retrieves all the localizations that the manager has
    pub async fn localizations(&self) -> Vec<Arc<Localization>> {
        self.localizations.read().await
            .values().cloned().collect()
    }

    /// Serializes the manager into data that can be serialized
    pub async fn serializable_data(&self) -> HashMap<String, LocalizationData> {
        join_all(self.language_map().await
            .into_iter()
            .map(|(key, localization)| {
                async move { (key, localization.data.read().await.clone()) }
            })
        ).await.into_iter().collect()
    }

    /// Deserializes the manager from serializable data
    pub fn from_serializable_data(data: HashMap<String, LocalizationData>) -> Arc<LocalizationManager> {
        Arc::new(LocalizationManager {
            localizations: RwLock::new(
                data.into_iter()
                    .map(|(str, data)| {
                        (str, Arc::new(Localization {
                            data: RwLock::new(data)
                        }))
                    })
                    .collect()
            )
        })
    }
}

/// Localization for a specific language, contains the localization data and helpful functions for it
pub struct Localization {
    data: RwLock<LocalizationData>,
}

/// Actual data of the localization
#[derive(Serialize, Deserialize, Clone)]
pub struct LocalizationData {
    /// Display name of the language, can be in its language (eg. English, Русский)
    pub display_name: String,
    /// Translation pairs for the language, key to translation
    pub translations: HashMap<String, String>,
}

impl Localization {
    /// Creates new instance of a localization, name of the language can be in its language (eg. English, Русский)
    pub fn new(display_name: &str) -> Arc<Localization> {
        Arc::new(Self {
            data: RwLock::new(LocalizationData {
                display_name: display_name.to_string(),
                translations: Default::default(),
            })
        })
    }

    /// Retrieves display name of the language
    pub async fn display_name(&self) -> String {
        self.data.read().await.display_name.clone()
    }

    /// Retrieves translation from the localization
    pub async fn get(&self, key: &str) -> Option<String> {
        self.data.read().await.translations
            .get(key).cloned()
    }

    /// Attempts to translate the localized string
    pub async fn translate(&self, l_str: &LocalizedString) -> Option<String> {
        let mut translation = self.get(&l_str.key).await?;

        for (index, parameter) in l_str.parameters.iter().enumerate() {
            let representation = match parameter {
                Value::Null => "null".to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                Value::Array(a) => serde_json::to_string(a).ok()?,
                Value::Object(o) => serde_json::to_string(o).ok()?
            };

            translation = translation.replace(&format!("{{{}}}", index), &representation);
        }

        Some(translation)
    }

    /// Inserts translation into the localization, overrides if already exists!
    pub async fn insert(&self, key: &str, value: &str) {
        self.data.write().await.translations
            .insert(key.to_string(), value.to_string());
    }

    /// Extends the localization with translation pairs
    pub async fn extend<I>(&self, iter: I)
        where I: IntoIterator<Item=(String, String)> {
        self.data.write().await.translations
            .extend(iter)
    }

    /// Extends the localization using [Value]
    ///
    /// Requires structure similar to this:
    /// ```json
    /// {
    ///   "localization.test": "Проверяем локализацию",
    /// }
    /// ```
    pub async fn extend_from_value(&self, value: Value) -> Result<(), serde_json::Error> {
        let map: HashMap<String, String> = serde_json::from_value(value)?;

        Ok(self.extend(map.into_iter()).await)
    }

    /// Retrieves a map of translation key to translation
    pub async fn translation_map(&self) -> HashMap<String, String> {
        self.data.read().await.translations
            .clone()
    }

    /// Retrieves all the languages that the manager has
    pub async fn translation_keys(&self) -> Vec<String> {
        self.data.read().await.translations
            .keys().cloned().collect()
    }
}

/// Localized string that needs to be translated before being used
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalizedString {
    /// Translation key
    pub key: String,
    /// Parameters
    pub parameters: Vec<Value>,
}

impl LocalizedString {
    /// Creates new localized string with no parameters
    pub fn new(key: &str) -> LocalizedString {
        LocalizedString {
            key: key.to_string(),
            parameters: Default::default(),
        }
    }

    /// Adds a parameter to the localized string
    pub fn add_parameter<P>(&mut self, parameter: P)
        where P: Into<Value> {
        self.parameters.push(parameter.into());
    }

    /// Adds a parameter to the localized string and returns it, made for chaining
    pub fn with_parameter<P>(mut self, parameter: P) -> LocalizedString
        where P: Into<Value> {
        self.add_parameter(parameter);
        self
    }
}