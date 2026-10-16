import shutil
import sys
from pathlib import Path
from catch.config import parse_config

TOOLS = [
    ("cmake", "Required for project build file generation and build orchestration.", True),
    ("ninja", "Optional fast build tool. Recommended generator for CMake.", False),
    ("git", "Required for managing and fetching git dependencies.", True),
    ("nvcc", "CUDA compiler driver. Required if building CUDA code.", False),
]

def check_tool(name: str) -> bool:
    return shutil.which(name) is not None

def cmd_doctor(args):
    print("Running catch doctor diagnostic check...\n")
    all_ok = True

    for tool, desc, required in TOOLS:
        found = check_tool(tool)
        status_str = "FOUND" if found else ("MISSING (Required)" if required else "MISSING (Optional)")
        icon = "[✓]" if found else ("[×]" if required else "[!]")
        print(f"{icon} {tool:8s}: {status_str}")
        print(f"    Reason: {desc}")
        if not found and required:
            all_ok = False
        print()

    # Check LLVM / MLIR directory if catch.toml is present
    project_dir = Path.cwd()
    config_path = project_dir / "catch.toml"
    if config_path.exists():
        try:
            cfg = parse_config(config_path)
            if cfg.llvm:
                print("Checking [llvm] configuration from catch.toml...")
                if cfg.llvm.dir:
                    p = Path(cfg.llvm.dir)
                    if p.exists() and p.is_dir():
                        print(f"[✓] LLVM_DIR  : FOUND ({p})")
                    else:
                        print(f"[×] LLVM_DIR  : NOT FOUND at {p}")
                        print("    Reason: Needed for CMake find_package(LLVM REQUIRED CONFIG).")
                        all_ok = False
                if cfg.llvm.mlir and cfg.llvm.mlir_dir:
                    p = Path(cfg.llvm.mlir_dir)
                    if p.exists() and p.is_dir():
                        print(f"[✓] MLIR_DIR  : FOUND ({p})")
                    else:
                        print(f"[×] MLIR_DIR  : NOT FOUND at {p}")
                        print("    Reason: Needed for CMake find_package(MLIR REQUIRED CONFIG).")
                        all_ok = False
                print()
        except Exception as e:
            print(f"[!] Warning: Could not parse catch.toml: {e}\n")

    if all_ok:
        print("All required toolchain components are ready.")
    else:
        print("Some required toolchain components are missing. Please install them to ensure catch works properly.")
