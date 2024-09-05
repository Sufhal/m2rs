use std::path::{Path, PathBuf};

pub fn search_files_in_directory(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Si l'entrée est un fichier, l'ajouter à la liste
        if path.is_file() {
            files.push(path);
        } 
        // Si l'entrée est un dossier, appeler la fonction récursivement
        else if path.is_dir() {
            search_files_in_directory(&path, files)?;
        }
    }
    Ok(())
}

pub fn write<T: AsRef<[u8]>>(path: &str, content: T) {
    std::fs::write(path, content).unwrap();
    println!("📝 Writing \"{path}\"");
}