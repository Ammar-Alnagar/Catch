use anyhow::Result;
use std::path::Path;
use which::which;

use crate::config::parse_config;

struct ToolInfo {
    name: &'static str,
    desc: &'static str,
    required: bool,
}

const TOOLS: &[ToolInfo] = &[
    ToolInfo {
        name: "cmake",
        desc: "Required for project build file generation and build orchestration.",
        required: true,
    },
    ToolInfo {
        name: "ninja",
        desc: "Optional fast build tool. Recommended generator for CMake.",
        required: false,
    },
    ToolInfo {
        name: "git",
        desc: "Required for managing and fetching git dependencies.",
        required: true,
    },
    ToolInfo {
        name: "nvcc",
        desc: "CUDA compiler driver. Required if building CUDA code.",
        required: false,
    },
];

pub fn cmd_doctor() -> Result<()> {
    println!("Running catch doctor diagnostic check...\n");
    let mut all_ok = true;

    for tool in TOOLS {
        let found = which(tool.name).is_ok();
        let status_str = if found {
            "FOUND"
        } else if tool.required {
            "MISSING (Required)"
        } else {
            "MISSING (Optional)"
        };

        let icon = if found {
            "[✓]"
        } else if tool.required {
            "[×]"
        } else {
            "[!]"
        };

        println!("{:<3} {:<8}: {}", icon, tool.name, status_str);
        println!("    Reason: {}", tool.desc);

        if !found && tool.required {
            all_ok = false;
        }
        println!();
    }

    let project_dir = std::env::current_dir()?;
    let config_path = project_dir.join("catch.toml");

    if config_path.exists() {
        if let Ok(cfg) = parse_config(&config_path) {
            if let Some(ref llvm) = cfg.llvm {
                println!("Checking [llvm] configuration from catch.toml...");
                if let Some(ref dir_str) = llvm.dir {
                    let p = Path::new(dir_str);
                    if p.exists() && p.is_dir() {
                        println!("[✓] LLVM_DIR  : FOUND ({})", p.display());
                    } else {
                        println!("[×] LLVM_DIR  : NOT FOUND at {}", p.display());
                        println!("    Reason: Needed for CMake find_package(LLVM REQUIRED CONFIG).");
                        all_ok = false;
                    }
                }
                if llvm.mlir {
                    if let Some(ref mlir_dir_str) = llvm.mlir_dir {
                        let p = Path::new(mlir_dir_str);
                        if p.exists() && p.is_dir() {
                            println!("[✓] MLIR_DIR  : FOUND ({})", p.display());
                        } else {
                            println!("[×] MLIR_DIR  : NOT FOUND at {}", p.display());
                            println!("    Reason: Needed for CMake find_package(MLIR REQUIRED CONFIG).");
                            all_ok = false;
                        }
                    }
                }
                println!();
            }
        }
    }

    if all_ok {
        println!("All required toolchain components are ready.");
    } else {
        println!("Some required toolchain components are missing. Please install them to ensure catch works properly.");
    }

    Ok(())
}
