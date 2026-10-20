use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const APP_DIRECTORY: &str = "snake-game";
const SCORE_FILE: &str = "high-score";

pub fn load() -> io::Result<usize> {
    load_from(&score_path()?)
}

pub fn save(score: usize) -> io::Result<()> {
    save_to(&score_path()?, score)
}

fn score_path() -> io::Result<PathBuf> {
    let data_directory = if cfg!(target_os = "windows") {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "APPDATA is not set"))?
    } else if cfg!(target_os = "macos") {
        home_directory()?
            .join("Library")
            .join("Application Support")
    } else if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME") {
        PathBuf::from(xdg_data_home)
    } else {
        home_directory()?.join(".local").join("share")
    };
    Ok(data_directory.join(APP_DIRECTORY).join(SCORE_FILE))
}

fn home_directory() -> io::Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))
}

fn load_from(path: &Path) -> io::Result<usize> {
    match fs::read_to_string(path) {
        Ok(contents) => contents.trim().parse().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid high score: {error}"),
            )
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error),
    }
}

fn save_to(path: &Path, score: usize) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "high score path has no parent")
    })?;
    fs::create_dir_all(parent)?;

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let temporary_path = parent.join(format!(
        ".{SCORE_FILE}.{}.{}.tmp",
        std::process::id(),
        nonce
    ));
    let mut temporary_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)?;
    let result = write_score(&mut temporary_file, score)
        .and_then(|()| temporary_file.sync_all())
        .and_then(|()| replace_file(&temporary_path, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

fn write_score(file: &mut File, score: usize) -> io::Result<()> {
    writeln!(file, "{score}")
}

#[cfg(not(windows))]
fn replace_file(temporary_path: &Path, path: &Path) -> io::Result<()> {
    fs::rename(temporary_path, path)
}

#[cfg(windows)]
fn replace_file(temporary_path: &Path, path: &Path) -> io::Result<()> {
    if !path.exists() {
        return fs::rename(temporary_path, path);
    }

    let backup_path = path.with_extension("high-score-backup");
    if backup_path.exists() {
        fs::remove_file(&backup_path)?;
    }
    fs::rename(path, &backup_path)?;
    match fs::rename(temporary_path, path) {
        Ok(()) => {
            let _ = fs::remove_file(backup_path);
            Ok(())
        }
        Err(error) => {
            let _ = fs::rename(backup_path, path);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("snake-game-test-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn missing_score_defaults_to_zero() {
        let path = test_path().join(SCORE_FILE);
        assert_eq!(load_from(&path).unwrap(), 0);
    }

    #[test]
    fn saved_score_can_be_loaded_again() {
        let directory = test_path();
        let path = directory.join(SCORE_FILE);
        save_to(&path, 27).unwrap();
        assert_eq!(load_from(&path).unwrap(), 27);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn malformed_score_is_reported() {
        let directory = test_path();
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join(SCORE_FILE);
        fs::write(&path, "not-a-number\n").unwrap();
        assert_eq!(
            load_from(&path).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
