use catch::config::parse_config;
use tempfile::TempDir;

#[test]
fn test_parse_valid_config() {
    let tmp_dir = TempDir::new().unwrap();
    let toml_content = r#"
[package]
name = "test_project"
version = "1.0.0"
type = "bin"

[build]
std = "20"
flags = ["-Wall"]
defines = ["FOO=1"]
languages = ["CXX", "CUDA"]

[dependencies]
fmt = { git = "https://github.com/fmtlib/fmt.git", tag = "10.0.0", target = "fmt::fmt" }

[cuda]
arch = ["native"]
std = "17"

[llvm]
dir = "/opt/llvm/lib/cmake/llvm"
mlir = true
components = ["Support"]
"#;
    let config_file = tmp_dir.path().join("catch.toml");
    std::fs::write(&config_file, toml_content).unwrap();

    let cfg = parse_config(&config_file).unwrap();
    assert_eq!(cfg.package.name, "test_project");
    assert_eq!(cfg.package.r#type, "bin");
    assert_eq!(cfg.build.std, "20");
    assert!(cfg.build.languages.contains(&"CUDA".to_string()));
    assert!(cfg.dependencies.contains_key("fmt"));
    assert_eq!(cfg.dependencies["fmt"].target.as_deref(), Some("fmt::fmt"));
    assert!(cfg.cuda.is_some());
    assert!(cfg.llvm.is_some());
    assert!(cfg.llvm.as_ref().unwrap().mlir);
}

#[test]
fn test_parse_invalid_package_name() {
    let tmp_dir = TempDir::new().unwrap();
    let toml_content = r#"
[package]
version = "1.0.0"
"#;
    let config_file = tmp_dir.path().join("catch.toml");
    std::fs::write(&config_file, toml_content).unwrap();

    let res = parse_config(&config_file);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("name"));
}

#[test]
fn test_parse_invalid_language() {
    let tmp_dir = TempDir::new().unwrap();
    let toml_content = r#"
[package]
name = "invalid_lang"

[build]
languages = ["INVALID"]
"#;
    let config_file = tmp_dir.path().join("catch.toml");
    std::fs::write(&config_file, toml_content).unwrap();

    let res = parse_config(&config_file);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("languages"));
    assert!(err_msg.contains("line 6"));
}
