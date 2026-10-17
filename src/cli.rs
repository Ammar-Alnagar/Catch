use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::Path;
use which::which;

use crate::config::{parse_config, CatchConfig};
use crate::generator::generate_cmakelists;
use crate::lock::{load_lock, save_lock, update_lock_for_project};
use crate::runner::run_command;
use crate::templates;

#[derive(Parser)]
#[command(name = "catch", author, version, about = "A Cargo-like tool for C++", long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true, help = "Enable verbose output")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Create a new catch C++ project")]
    New {
        #[arg(help = "Project name")]
        name: String,

        #[arg(long, help = "Enable CUDA support")]
        cuda: bool,

        #[arg(long, help = "Enable MLIR/LLVM support")]
        mlir: bool,

        #[arg(short, long, help = "Force overwrite without prompt")]
        force: bool,
    },

    #[command(about = "Build the project")]
    Build {
        #[arg(long, help = "Build in Release mode")]
        release: bool,
    },

    #[command(about = "Build and run the main executable")]
    Run {
        #[arg(long, help = "Build in Release mode")]
        release: bool,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true, help = "Arguments passed to executable after '--'")]
        args: Vec<String>,
    },

    #[command(about = "Run tests with ctest")]
    Test {
        #[arg(long, help = "Build in Release mode before testing")]
        release: bool,
    },

    #[command(about = "Remove the build directory")]
    Clean,

    #[command(about = "Add a git dependency")]
    Add {
        #[arg(help = "Git repository URL")]
        url: String,

        #[arg(short, long, help = "Git tag")]
        tag: Option<String>,

        #[arg(short, long, help = "Git branch")]
        branch: Option<String>,

        #[arg(short, long, help = "Git revision / commit hash")]
        rev: Option<String>,

        #[arg(short, long, help = "Dependency name override")]
        name: Option<String>,

        #[arg(long, help = "CMake target override (e.g. fmt::fmt)")]
        target: Option<String>,
    },

    #[command(about = "Remove a dependency")]
    Remove {
        #[arg(help = "Dependency name")]
        name: String,
    },

    #[command(about = "Update dependency lock file")]
    Update {
        #[arg(help = "Specific dependency name to update")]
        name: Option<String>,
    },

    #[command(about = "Generate project documentation")]
    Docs {
        #[arg(long, help = "Print documentation to stdout instead of file")]
        stdout: bool,
    },

    #[command(about = "Check system toolchain dependencies")]
    Doctor,
}

pub fn is_tool_available(name: &str) -> bool {
    which(name).is_ok()
}

pub fn infer_dep_name_from_url(url: &str) -> String {
    let mut cleaned = url.trim_end_matches('/');
    if cleaned.ends_with(".git") {
        cleaned = &cleaned[..cleaned.len() - 4];
    }
    let name = cleaned.split('/').next_back().unwrap_or("dep");
    if name.is_empty() {
        "dep".to_string()
    } else {
        name.to_string()
    }
}

pub fn cmd_new(name: &str, cuda: bool, mlir: bool, force: bool) -> Result<()> {
    let project_dir = std::env::current_dir()?.join(name);

    if project_dir.exists() && project_dir.read_dir()?.next().is_some() {
        println!("Directory '{}' exists and is not empty.", name);
        if !force {
            print!("Overwrite files in this directory? [y/N]: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                println!("Aborted.");
                std::process::exit(1);
            }
        }
    }

    std::fs::create_dir_all(&project_dir)?;
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let mut langs = vec!["CXX"];
    if cuda {
        langs.push("CUDA");
    }

    let cuda_sec = if cuda {
        "\n[cuda]\narch = [\"native\"]\nstd = \"17\"\nflags = []\n"
    } else {
        ""
    };

    let llvm_sec = if mlir {
        "\n[llvm]\ndir = \"/path/to/llvm-build/lib/cmake/llvm\"\nmlir_dir = \"/path/to/llvm-build/lib/cmake/mlir\"\nmlir = true\ncomponents = [\"Support\", \"Core\"]\nmlir_libs = [\"MLIRIR\", \"MLIRParser\", \"MLIRSupport\"]\n"
    } else {
        ""
    };

    let langs_formatted = format!("{:?}", langs).replace('\'', "\"");
    let catch_toml = templates::CATCH_TOML_TEMPLATE
        .replace("{name}", name)
        .replace("{languages}", &langs_formatted)
        .replace("{cuda_section}", cuda_sec)
        .replace("{llvm_section}", llvm_sec);

    std::fs::write(project_dir.join("catch.toml"), catch_toml)?;
    std::fs::write(project_dir.join(".gitignore"), templates::GITIGNORE_TEMPLATE)?;

    if mlir {
        std::fs::write(src_dir.join("main.cpp"), templates::MLIR_MAIN_CPP_TEMPLATE)?;
    } else if cuda {
        std::fs::write(src_dir.join("main.cu"), templates::MAIN_CU_TEMPLATE)?;
    } else {
        std::fs::write(src_dir.join("main.cpp"), templates::MAIN_CPP_TEMPLATE)?;
    }

    println!("Created catch project '{}'.", name);
    Ok(())
}

pub fn ensure_cmakelists(project_dir: &Path, verbose: bool) -> Result<CatchConfig> {
    let config_path = project_dir.join("catch.toml");
    if !config_path.exists() {
        eprintln!("Error: catch.toml not found in {}", project_dir.display());
        std::process::exit(1);
    }

    let cfg = parse_config(&config_path)?;
    let lock_data = update_lock_for_project(project_dir, None)?;

    let cmake_content = generate_cmakelists(&cfg, project_dir, Some(&lock_data));
    let cmakelists_path = project_dir.join("CMakeLists.txt");

    if cmakelists_path.exists() {
        let existing = std::fs::read_to_string(&cmakelists_path)?;
        if !existing.starts_with("# Generated by catch. Do not edit.") {
            eprintln!("Error: Existing CMakeLists.txt was not generated by catch. Refusing to overwrite user file.");
            std::process::exit(1);
        }
    }

    std::fs::write(&cmakelists_path, cmake_content)?;
    if verbose {
        println!("Generated CMakeLists.txt successfully.");
    }

    Ok(cfg)
}

pub fn cmd_build(release: bool, verbose: bool) -> Result<()> {
    let project_dir = std::env::current_dir()?;
    let cfg = ensure_cmakelists(&project_dir, verbose)?;

    if !is_tool_available("cmake") {
        eprintln!("Error: 'cmake' executable not found in PATH.");
        std::process::exit(1);
    }

    let mut configure_cmd = vec!["cmake".to_string(), "-S".to_string(), ".".to_string(), "-B".to_string(), "build".to_string()];
    if is_tool_available("ninja") {
        configure_cmd.push("-G".to_string());
        configure_cmd.push("Ninja".to_string());
    }

    let build_type = if release { "Release" } else { "Debug" };
    configure_cmd.push(format!("-DCMAKE_BUILD_TYPE={}", build_type));

    if let Some(ref llvm) = cfg.llvm {
        if let Some(ref dir) = llvm.dir {
            configure_cmd.push(format!("-DLLVM_DIR={}", dir));
        }
        if let Some(ref mlir_dir) = llvm.mlir_dir {
            configure_cmd.push(format!("-DMLIR_DIR={}", mlir_dir));
        }
    }

    if run_command(&configure_cmd, Some(&project_dir), verbose, true, false).is_err() {
        if cfg.llvm.is_some() {
            eprintln!("Set [llvm].dir and mlir_dir to the cmake folders inside your LLVM build, not a system LLVM package without MLIR.");
        }
        std::process::exit(1);
    }

    let build_cmd = vec!["cmake", "--build", "build", "--config", build_type];
    if run_command(&build_cmd, Some(&project_dir), verbose, true, false).is_err() {
        std::process::exit(1);
    }

    Ok(())
}

pub fn cmd_run(release: bool, verbose: bool, run_args: Vec<String>) -> Result<()> {
    cmd_build(release, verbose)?;

    let project_dir = std::env::current_dir()?;
    let cfg = parse_config(&project_dir.join("catch.toml"))?;

    if cfg.package.r#type != "bin" {
        eprintln!("Error: 'catch run' can only be executed on binary packages (type = 'bin').");
        std::process::exit(1);
    }

    let exe_name = if cfg!(windows) {
        format!("{}.exe", cfg.package.name)
    } else {
        cfg.package.name.clone()
    };

    let mut exe_path = project_dir.join("build").join(&exe_name);
    if cfg!(windows) && !exe_path.exists() {
        let sub = if release { "Release" } else { "Debug" };
        exe_path = project_dir.join("build").join(sub).join(&exe_name);
    }

    if !exe_path.exists() {
        eprintln!("Error: Executable not found at {}", exe_path.display());
        std::process::exit(1);
    }

    let mut exec_args = vec![exe_path.to_string_lossy().to_string()];
    let filtered_args: Vec<_> = if !run_args.is_empty() && run_args[0] == "--" {
        run_args[1..].to_vec()
    } else {
        run_args
    };
    exec_args.extend(filtered_args);

    if run_command(&exec_args, Some(&project_dir), verbose, true, false).is_err() {
        std::process::exit(1);
    }

    Ok(())
}

pub fn cmd_test(release: bool, verbose: bool) -> Result<()> {
    cmd_build(release, verbose)?;

    let project_dir = std::env::current_dir()?;
    if !is_tool_available("ctest") {
        eprintln!("Error: 'ctest' executable not found in PATH.");
        std::process::exit(1);
    }

    let ctest_cmd = vec!["ctest", "--test-dir", "build", "--output-on-failure"];
    if run_command(&ctest_cmd, Some(&project_dir), verbose, true, false).is_err() {
        std::process::exit(1);
    }

    Ok(())
}

pub fn cmd_clean() -> Result<()> {
    let build_dir = std::env::current_dir()?.join("build");
    if build_dir.exists() {
        println!("Removing {}", build_dir.display());
        std::fs::remove_dir_all(build_dir)?;
    } else {
        println!("build/ directory does not exist.");
    }
    Ok(())
}

pub fn cmd_add(
    url: String,
    tag: Option<String>,
    branch: Option<String>,
    rev: Option<String>,
    name: Option<String>,
    target: Option<String>,
) -> Result<()> {
    let project_dir = std::env::current_dir()?;
    let toml_path = project_dir.join("catch.toml");
    if !toml_path.exists() {
        eprintln!("Error: catch.toml not found.");
        std::process::exit(1);
    }

    let dep_name = name.unwrap_or_else(|| infer_dep_name_from_url(&url));

    let content = std::fs::read_to_string(&toml_path)?;
    let mut filtered_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", dep_name)) || trimmed.starts_with(&format!("\"{}\" =", dep_name)) {
            continue;
        }
        filtered_lines.push(line);
    }

    let mut content = filtered_lines.join("\n");
    if !content.ends_with('\n') {
        content.push('\n');
    }

    let mut ref_part = String::new();
    if let Some(t) = tag {
        ref_part = format!(", tag = \"{}\"", t);
    } else if let Some(b) = branch {
        ref_part = format!(", branch = \"{}\"", b);
    } else if let Some(r) = rev {
        ref_part = format!(", rev = \"{}\"", r);
    }

    let target_part = if let Some(t) = target {
        format!(", target = \"{}\"", t)
    } else {
        String::new()
    };

    let dep_line = format!("{} = {{ git = \"{}\"{}{} }}\n", dep_name, url, ref_part, target_part);

    if content.contains("[dependencies]") {
        let lines: Vec<_> = content.lines().collect();
        let mut new_lines = Vec::new();
        let mut in_deps = false;
        let mut appended = false;

        for line in lines {
            let trimmed = line.trim();
            if trimmed == "[dependencies]" {
                in_deps = true;
                new_lines.push(line.to_string());
            } else if in_deps && trimmed.starts_with('[') {
                new_lines.push(dep_line.trim_end().to_string());
                appended = true;
                in_deps = false;
                new_lines.push(line.to_string());
            } else {
                new_lines.push(line.to_string());
            }
        }

        if in_deps && !appended {
            new_lines.push(dep_line.trim_end().to_string());
        }

        content = new_lines.join("\n") + "\n";
    } else {
        content.push_str(&format!("\n[dependencies]\n{}", dep_line));
    }

    std::fs::write(&toml_path, content)?;
    println!("Added dependency '{}' to catch.toml", dep_name);

    update_lock_for_project(&project_dir, Some(&dep_name))?;
    Ok(())
}

pub fn cmd_remove(dep_name: &str) -> Result<()> {
    let project_dir = std::env::current_dir()?;
    let toml_path = project_dir.join("catch.toml");
    if !toml_path.exists() {
        eprintln!("Error: catch.toml not found.");
        std::process::exit(1);
    }

    let content = std::fs::read_to_string(&toml_path)?;
    let mut new_lines = Vec::new();
    let mut removed = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{} =", dep_name)) || trimmed.starts_with(&format!("\"{}\" =", dep_name)) {
            removed = true;
            continue;
        }
        new_lines.push(line);
    }

    if !removed {
        eprintln!("Dependency '{}' not found in catch.toml.", dep_name);
        std::process::exit(1);
    }

    std::fs::write(&toml_path, new_lines.join("\n") + "\n")?;
    println!("Removed dependency '{}' from catch.toml", dep_name);

    let lock_path = project_dir.join("catch.lock");
    if lock_path.exists() {
        let mut lock_data = load_lock(&lock_path);
        if lock_data.dependencies.remove(dep_name).is_some() {
            save_lock(&lock_path, &lock_data)?;
        }
    }

    Ok(())
}

pub fn cmd_update(dep_name: Option<&str>) -> Result<()> {
    let project_dir = std::env::current_dir()?;
    update_lock_for_project(&project_dir, dep_name)?;
    println!("Updated catch.lock");
    Ok(())
}
