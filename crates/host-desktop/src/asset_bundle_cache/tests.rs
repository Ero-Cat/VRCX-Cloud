use super::*;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("vrcx-0-{name}-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn write_cache_entry(
    cache_root: &Path,
    file_id: &str,
    file_version: i32,
    variant: &str,
    variant_version: i32,
    bytes: &[u8],
    locked: bool,
) -> PathBuf {
    let path = cache_root
        .join(asset_id(file_id, variant))
        .join(asset_version(file_version, variant_version));
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("__data"), bytes).unwrap();
    if locked {
        std::fs::write(path.join("__lock"), b"").unwrap();
    }
    path
}

fn set_cache_entry_modified(path: &Path, seconds: u64) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(path.join("__data"))
        .unwrap();
    let modified = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(seconds);
    file.set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
}

#[test]
fn checks_cache_size_lock_and_location_without_touching_real_vrchat_cache() {
    let dir = TestDir::new("asset-cache-check");
    let cache_path = write_cache_entry(
        &dir.path,
        "file_world",
        42,
        "security",
        7,
        b"cached-world",
        true,
    );

    let result = check_vrchat_cache_in(&dir.path, "file_world", 42, "security", 7);

    assert_eq!(result.item1, 12);
    assert!(result.item2);
    assert_eq!(result.item3, cache_path.to_string_lossy());
    assert_eq!(cache_size_in(&dir.path), 12);
}

#[test]
fn deletes_specific_standard_and_variant_cache_entries() {
    let dir = TestDir::new("asset-cache-delete");
    let standard_path = write_cache_entry(&dir.path, "file_avatar", 9, "", 0, b"standard", false);
    let variant_path = write_cache_entry(
        &dir.path,
        "file_avatar",
        9,
        "security",
        2,
        b"variant",
        false,
    );
    let other_path = write_cache_entry(&dir.path, "file_other", 1, "", 0, b"other", false);

    delete_cache_in(&dir.path, "file_avatar", 9, "security", 2);

    assert!(!standard_path.exists());
    assert!(!variant_path.exists());
    assert!(other_path.exists());
    assert_eq!(cache_size_in(&dir.path), 5);
}

#[test]
fn sweep_cache_trims_oldest_entries_to_size_limit() {
    let dir = TestDir::new("asset-cache-trim");
    let oldest = write_cache_entry(&dir.path, "file_oldest", 1, "", 0, b"123456", false);
    let middle = write_cache_entry(&dir.path, "file_middle", 1, "", 0, b"12345", false);
    let newest = write_cache_entry(&dir.path, "file_newest", 1, "", 0, b"1234", false);
    set_cache_entry_modified(&oldest, 1);
    set_cache_entry_modified(&middle, 2);
    set_cache_entry_modified(&newest, 3);

    let removed = sweep_cache_in(&dir.path, Some(9));

    assert!(!oldest.exists());
    assert!(middle.exists());
    assert!(newest.exists());
    assert_eq!(cache_size_in(&dir.path), 9);
    assert_eq!(
        removed,
        vec![cache_relative_path(oldest.parent().unwrap(), &oldest)]
    );
}

#[test]
fn sweep_cache_without_size_limit_keeps_current_entries() {
    let dir = TestDir::new("asset-cache-sweep-without-limit");
    let first = write_cache_entry(&dir.path, "file_first", 1, "", 0, b"123456", false);
    let second = write_cache_entry(&dir.path, "file_second", 1, "", 0, b"12345", false);

    let removed = sweep_cache_in(&dir.path, None);

    assert!(removed.is_empty());
    assert!(first.exists());
    assert!(second.exists());
    assert_eq!(cache_size_in(&dir.path), 11);
}

#[test]
fn sweep_cache_skips_locked_entries_when_trimming() {
    let dir = TestDir::new("asset-cache-trim-locked");
    let locked = write_cache_entry(&dir.path, "file_locked", 1, "", 0, b"123456", true);
    let middle = write_cache_entry(&dir.path, "file_middle", 1, "", 0, b"12345", false);
    let newest = write_cache_entry(&dir.path, "file_newest", 1, "", 0, b"1234", false);
    set_cache_entry_modified(&locked, 1);
    set_cache_entry_modified(&middle, 2);
    set_cache_entry_modified(&newest, 3);

    let removed = sweep_cache_in(&dir.path, Some(10));

    assert!(locked.exists());
    assert!(!middle.exists());
    assert!(newest.exists());
    assert_eq!(cache_size_in(&dir.path), 10);
    assert_eq!(
        removed,
        vec![cache_relative_path(middle.parent().unwrap(), &middle)]
    );
}

#[test]
fn trim_rechecks_a_cache_lock_immediately_before_deleting() {
    let dir = TestDir::new("asset-cache-trim-late-lock");
    let cache_entry = write_cache_entry(&dir.path, "file_late_lock", 1, "", 0, b"cache", false);
    std::fs::write(cache_entry.join("__lock"), b"").unwrap();

    assert!(!remove_trim_candidate(&cache_entry));
    assert!(cache_entry.exists());
}

#[test]
fn delete_all_cache_recreates_empty_cache_root() {
    let dir = TestDir::new("asset-cache-delete-all");
    write_cache_entry(&dir.path, "file_world", 1, "", 0, b"cache", false);

    delete_all_cache_in(&dir.path).unwrap();

    assert!(dir.path.is_dir());
    assert_eq!(std::fs::read_dir(&dir.path).unwrap().count(), 0);
    assert_eq!(cache_size_in(&dir.path), 0);
}

#[test]
fn delete_all_cache_reports_removal_failures() {
    let dir = TestDir::new("asset-cache-delete-all-error");
    let file_path = dir.path.join("not-a-directory");
    std::fs::write(&file_path, b"cache").unwrap();

    assert!(delete_all_cache_in(&file_path).is_err());
    assert!(file_path.exists());
}

#[test]
fn sweep_keeps_latest_non_empty_cache_when_newer_directory_is_empty() {
    let dir = TestDir::new("asset-cache-sweep-empty-latest");
    let cache_dir = dir.path.join(asset_id("file_world", ""));
    let valid_path = cache_dir.join(asset_version(1, 0));
    std::fs::create_dir_all(&valid_path).unwrap();
    std::fs::write(valid_path.join("__data"), b"cached-world").unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));
    let empty_path = cache_dir.join(asset_version(2, 0));
    std::fs::create_dir_all(&empty_path).unwrap();

    let removed = sweep_cache_in(&dir.path, None);

    assert!(valid_path.exists());
    assert!(!empty_path.exists());
    assert!(removed.is_empty());
}
