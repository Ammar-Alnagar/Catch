use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_type")]
    pub r#type: String, // "bin" or "lib"
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_type() -> String {
    "bin".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_std")]
    pub std: String,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub defines: Vec<String>,
    #[serde(default = "default_release_flags")]
    pub release_flags: Vec<String>,
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
}

fn default_std() -> String {
    "20".to_string()
}

fn default_release_flags() -> Vec<String> {
    vec!["-O3".to_string(), "-DNDEBUG".to_string()]
}

fn default_languages() -> Vec<String> {
    vec!["CXX".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    pub git: String,
    pub tag: Option<String>,
    pub branch: Option<String>,
    pub rev: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CudaConfig {
    #[serde(default = "default_cuda_arch")]
    pub arch: Vec<String>,
    #[serde(default = "default_cuda_std")]
    pub std: String,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub separable: bool,
}

fn default_cuda_arch() -> Vec<String> {
    vec!["native".to_string()]
}

fn default_cuda_std() -> String {
    "17".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlvmConfig {
    pub dir: Option<String>,
    pub mlir_dir: Option<String>,
    #[serde(default)]
    pub mlir: bool,
    #[serde(default = "default_llvm_components")]
    pub components: Vec<String>,
    #[serde(default)]
    pub mlir_libs: Vec<String>,
}

fn default_llvm_components() -> Vec<String> {
    vec!["Support".to_string(), "Core".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchConfig {
    pub package: PackageConfig,
    #[serde(default = "default_build_config")]
    pub build: BuildConfig,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencyConfig>,
    pub cuda: Option<CudaConfig>,
    pub llvm: Option<LlvmConfig>,
}

fn default_build_config() -> BuildConfig {
    BuildConfig {
        std: default_std(),
        flags: Vec::new(),
        defines: Vec::new(),
        release_flags: default_release_flags(),
        languages: default_languages(),
    }
}

pub fn find_key_line_in_toml(toml_str: &str, target_key: &str, section: Option<&str>) -> Option<usize> {
    let lines = toml_str.lines();
    let mut current_section: Option<String> = None;

    for (idx, line) in lines.enumerate() {
        let line_num = idx + 1;
        let stripped = line.trim();
        if stripped.starts_with('[') && stripped.ends_with(']') {
            current_section = Some(stripped[1..stripped.len() - 1].trim().to_string());
            continue;
        }
        if line.contains('=') {
            let key = line.split('=').next().unwrap().trim();
            if key == target_key {
                if let Some(sec) = section {
                    if let Some(ref cur) = current_section {
                        if cur == sec || cur.starts_with(&format!("{}.", sec)) {
                            return Some(line_num);
                        }
                    }
                } else {
                    return Some(line_num);
                }
            }
        }
    }
    None
}

pub fn parse_config(config_path: &Path) -> Result<CatchConfig> {
    if !config_path.exists() {
        return Err(anyhow!("Configuration file not found: {}", config_path.display()));
    }

    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;

    let data: toml::Value = toml::from_str(&content)
        .map_err(|e| anyhow!("TOML Syntax Error in {}: {}", config_path.display(), e))?;

    let file_name = config_path.file_name().unwrap_or_default().to_string_lossy();

    // Validate [package]
    let pkg_val = data.get("package")
        .ok_or_else(|| anyhow!("Missing or invalid [package] section in {}", file_name))?;

    if !pkg_val.is_table() {
        return Err(anyhow!("Missing or invalid [package] section in {}", file_name));
    }

    let name = pkg_val.get("name").and_then(|v| v.as_str());
    if name.is_none() || name.unwrap().trim().is_empty() {
        let line_num = find_key_line_in_toml(&content, "name", Some("package"));
        let loc = line_num.map(|l| format!(" line {}", l)).unwrap_or_default();
        return Err(anyhow!(
            "Validation error in {} in [package]{} key 'name': Field 'name' is required and must be a non-empty string",
            file_name, loc
        ));
    }

    if let Some(pkg_type) = pkg_val.get("type").and_then(|v| v.as_str()) {
        if pkg_type != "bin" && pkg_type != "lib" {
            let line_num = find_key_line_in_toml(&content, "type", Some("package"));
            let loc = line_num.map(|l| format!(" line {}", l)).unwrap_or_default();
            return Err(anyhow!(
                "Validation error in {} in [package]{} key 'type': Invalid package type '{}'. Expected 'bin' or 'lib'",
                file_name, loc, pkg_type
            ));
        }
    }

    // Validate [build] languages
    if let Some(build_val) = data.get("build") {
        if let Some(langs_val) = build_val.get("languages") {
            if let Some(arr) = langs_val.as_array() {
                let allowed = ["C", "CXX", "CUDA"];
                for item in arr {
                    if let Some(lang) = item.as_str() {
                        if !allowed.contains(&lang) {
                            let line_num = find_key_line_in_toml(&content, "languages", Some("build"));
                            let loc = line_num.map(|l| format!(" line {}", l)).unwrap_or_default();
                            return Err(anyhow!(
                                "Validation error in {} in [build]{} key 'languages': Unsupported language '{}'. Allowed languages: C, CXX, CUDA",
                                file_name, loc, lang
                            ));
                        }
                    } else {
                        let line_num = find_key_line_in_toml(&content, "languages", Some("build"));
                        let loc = line_num.map(|l| format!(" line {}", l)).unwrap_or_default();
                        return Err(anyhow!(
                            "Validation error in {} in [build]{} key 'languages': 'languages' must be a list of strings",
                            file_name, loc
                        ));
                    }
                }
            } else {
                let line_num = find_key_line_in_toml(&content, "languages", Some("build"));
                let loc = line_num.map(|l| format!(" line {}", l)).unwrap_or_default();
                return Err(anyhow!(
                    "Validation error in {} in [build]{} key 'languages': 'languages' must be a list of strings",
                    file_name, loc
                ));
            }
        }
    }

    // Deserialization
    let mut config: CatchConfig = toml::from_str(&content)
        .map_err(|e| anyhow!("Failed to parse config {}: {}", file_name, e))?;

    // Default cuda section if CUDA language is specified in build languages but no [cuda] table
    if config.build.languages.contains(&"CUDA".to_string()) && config.cuda.is_none() {
        config.cuda = Some(CudaConfig {
            arch: default_cuda_arch(),
            std: default_cuda_std(),
            flags: Vec::new(),
            separable: false,
        });
    }

    Ok(config)
}
