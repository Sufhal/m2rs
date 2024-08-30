pub fn write<T: AsRef<[u8]>>(path: &str, content: T) {
    std::fs::write(path, content).unwrap();
    println!("📝 Writing \"{path}\"");
}