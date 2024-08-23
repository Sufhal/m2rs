pub fn debug_using_trash_file(
    name: &str,
    content: String,
) {
    let _ = std::fs::write(
        std::path::Path::new(&format!("trash/{name}.txt")), 
        content
    );
}

pub fn is_browser() -> bool {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            return true;
        } 
        else {
            return false;
        }
    }
}