use anyhow::{anyhow, Result};
use std::path::Path;

/// Create a symlink at `link_path` pointing to `target`.
/// Creates parent directories as needed.
/// Replaces an existing symlink at `link_path`.
/// Returns Err if `link_path` exists and is NOT a symlink (avoids clobbering files).
pub fn create_symlink(target: &Path, link_path: &Path) -> Result<()> {
    if let Some(parent) = link_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if link_path.exists() || link_path.is_symlink() {
        if link_path.is_symlink() {
            std::fs::remove_file(link_path)?;
        } else {
            return Err(anyhow!(
                "{} already exists and is not a symlink — remove it manually first",
                link_path.display()
            ));
        }
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link_path)?;

    #[cfg(not(unix))]
    return Err(anyhow!("symlinks not supported on this platform"));

    Ok(())
}

/// Copy `src` to `dest`, creating parent directories as needed.
pub fn copy_file(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(src, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn create_symlink_creates_symlink() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_fs_test_create");
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target.txt");
        let link = dir.join("link.txt");
        fs::write(&target, "content").unwrap();
        // act
        create_symlink(&target, &link).unwrap();
        // assert
        assert!(link.is_symlink());
        assert_eq!(fs::read_link(&link).unwrap(), target);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_symlink_replaces_existing_symlink() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_fs_test_replace");
        fs::create_dir_all(&dir).unwrap();
        let target1 = dir.join("target1.txt");
        let target2 = dir.join("target2.txt");
        let link = dir.join("link.txt");
        fs::write(&target1, "v1").unwrap();
        fs::write(&target2, "v2").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target1, &link).unwrap();
        // act
        create_symlink(&target2, &link).unwrap();
        // assert
        assert_eq!(fs::read_link(&link).unwrap(), target2);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_symlink_errors_on_existing_non_symlink() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_fs_test_err");
        fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target.txt");
        let link = dir.join("real_file.txt");
        fs::write(&target, "t").unwrap();
        fs::write(&link, "existing").unwrap();
        // act
        let result = create_symlink(&target, &link);
        // assert
        assert!(result.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_symlink_creates_parent_dirs() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_fs_test_parents");
        let target = dir.join("target.txt");
        let link = dir.join("nested").join("deep").join("link.txt");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&target, "t").unwrap();
        // act
        create_symlink(&target, &link).unwrap();
        // assert
        assert!(link.is_symlink());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn copy_file_copies_content() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_fs_test_copy");
        fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src.txt");
        let dest = dir.join("dest.txt");
        fs::write(&src, "hello qwert").unwrap();
        // act
        copy_file(&src, &dest).unwrap();
        // assert
        assert_eq!(fs::read_to_string(&dest).unwrap(), "hello qwert");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn copy_file_creates_parent_dirs() {
        // arrange
        let dir = std::env::temp_dir().join("qwert_fs_test_copy_parents");
        fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src.txt");
        let dest = dir.join("sub").join("dir").join("dest.txt");
        fs::write(&src, "data").unwrap();
        // act
        copy_file(&src, &dest).unwrap();
        // assert
        assert!(dest.exists());
        fs::remove_dir_all(&dir).ok();
    }
}
