use catch::cli::find_cmakelists_in;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_cmakelists_in_base_dir() {
    let tmp = TempDir::new().unwrap();
    let cmake_file = tmp.path().join("CMakeLists.txt");
    fs::write(&cmake_file, "cmake_minimum_required(VERSION 3.20)\n").unwrap();

    let found = find_cmakelists_in(tmp.path(), None).unwrap();
    assert_eq!(found, cmake_file);
}

#[test]
fn test_find_cmakelists_in_parent_dir() {
    let tmp = TempDir::new().unwrap();
    let cmake_file = tmp.path().join("CMakeLists.txt");
    fs::write(&cmake_file, "cmake_minimum_required(VERSION 3.20)\n").unwrap();

    let sub_dir = tmp.path().join("src").join("nested");
    fs::create_dir_all(&sub_dir).unwrap();

    let found = find_cmakelists_in(&sub_dir, None).unwrap();
    assert_eq!(found, cmake_file);
}

#[test]
fn test_find_cmakelists_with_explicit_path() {
    let tmp = TempDir::new().unwrap();
    let sub_proj = tmp.path().join("sub_proj");
    fs::create_dir_all(&sub_proj).unwrap();
    let cmake_file = sub_proj.join("CMakeLists.txt");
    fs::write(&cmake_file, "cmake_minimum_required(VERSION 3.20)\n").unwrap();

    // Passing directory path
    let found = find_cmakelists_in(tmp.path(), Some("sub_proj")).unwrap();
    assert_eq!(found, cmake_file);

    // Passing direct file path
    let found_file = find_cmakelists_in(tmp.path(), Some("sub_proj/CMakeLists.txt")).unwrap();
    assert_eq!(found_file, cmake_file);
}

#[test]
fn test_find_cmakelists_not_found() {
    let tmp = TempDir::new().unwrap();
    let res = find_cmakelists_in(tmp.path(), None);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("No CMakeLists.txt found"));
}
