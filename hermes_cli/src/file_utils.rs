use std::path::PathBuf;

pub fn read_folder(folder_path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.insert(0, path);
        } else if path.is_dir() {
            files.extend(read_folder(&path)?);
        }
    }

    files.sort();

    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_read_folder() {
        let current_dir = env::current_dir().unwrap();
        let folder_path = current_dir.join("tests/fixtures/read_folder");
        let files = read_folder(&folder_path).unwrap();

        assert!(files.len() == 3);
        assert_eq!(
            files,
            vec![
                current_dir.join("tests/fixtures/read_folder/file1.txt"),
                current_dir.join("tests/fixtures/read_folder/subfolder1/file2.txt"),
                current_dir.join("tests/fixtures/read_folder/subfolder2/file3.txt"),
            ]
        );
    }
}
