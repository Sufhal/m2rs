pub fn generate_unique_string() -> String {
    uuid::Uuid::new_v4().to_string()
}