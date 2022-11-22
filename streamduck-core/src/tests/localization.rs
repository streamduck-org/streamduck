use serde_json::json;
use crate::localization::{Localization, LocalizationManager, LocalizedString};

#[tokio::test]
async fn test_localization() {
    let data = serde_json::from_value(json!({
        "en": {
            "display_name": "English",
            "translations": {
                "localization.test": "Testing localization",
                "parameter.test": "Testing '{0}' parameter",
                "missing.translation.test": "It should be translated to this"
            }
        },
        "ru": {
            "display_name": "Русский",
            "translations": {
                "localization.test": "Проверяем локализацию",
                "parameter.test": "Проверяем '{0}' параметр"
            }
        }
    })).unwrap();

    let manager = LocalizationManager::from_serializable_data(data);

    assert_eq!(manager.translate(
        "en",
        &LocalizedString::new("localization.test")
    ).await.unwrap(), "Testing localization");

    assert_eq!(manager.translate(
        "ru",
        &LocalizedString::new("localization.test")
    ).await.unwrap(), "Проверяем локализацию");

    assert_eq!(manager.translate(
        "en",
        &LocalizedString::new("parameter.test")
            .with_parameter("this")
    ).await.unwrap(), "Testing 'this' parameter");

    assert_eq!(manager.translate(
        "ru",
        &LocalizedString::new("parameter.test")
        .with_parameter("этот")
    ).await.unwrap(), "Проверяем 'этот' параметр");

    assert_eq!(manager.translate(
        "ru",
        &LocalizedString::new("missing.translation.test")
    ).await.unwrap(), "It should be translated to this");
}