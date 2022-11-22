use serde_json::json;
use crate::localization::{Localization, LocalizationManager, LocalizedString};

#[tokio::test]
async fn test_localization() {
    let localization = Localization::new("Русский");

    localization.extend_from_value(json!({
        "localization.test": "Проверяем локализацию",
        "parameter.test": "Проверяем '{0}' параметр"
    })).await.unwrap();

    assert_eq!(localization.translate(
        &LocalizedString::new("localization.test")
    ).await.unwrap(), "Проверяем локализацию");

    assert_eq!(localization.translate(
        &LocalizedString::new("parameter.test")
        .with_parameter("этот")
    ).await.unwrap(), "Проверяем 'этот' параметр");

    let manager = LocalizationManager::new();

    manager.insert("ru", localization).await;

    println!("{:#?}", manager.translation_map().await);
}