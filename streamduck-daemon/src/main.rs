use streamduck_core::ui::ValuePath;

fn main() {
    let serialized = r#"["a", "b"]"#;

    let data = serde_json::from_str::<ValuePath>(serialized).unwrap();

    println!("{:?}", data);
}
