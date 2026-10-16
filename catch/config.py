import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Dict, Any, Optional

class ConfigError(Exception):
    pass

@dataclass
class PackageConfig:
    name: str
    version: str = "0.1.0"
    type: str = "bin"  # "bin" or "lib"

@dataclass
class BuildConfig:
    std: str = "20"
    flags: List[str] = field(default_factory=list)
    defines: List[str] = field(default_factory=list)
    release_flags: List[str] = field(default_factory=lambda: ["-O2"])
    languages: List[str] = field(default_factory=lambda: ["CXX"])

@dataclass
class DependencyConfig:
    name: str
    git: str
    tag: Optional[str] = None
    branch: Optional[str] = None
    rev: Optional[str] = None
    target: Optional[str] = None

@dataclass
class CudaConfig:
    arch: List[str] = field(default_factory=lambda: ["native"])
    std: str = "17"
    flags: List[str] = field(default_factory=list)
    separable: bool = False

@dataclass
class LlvmConfig:
    dir: Optional[str] = None
    mlir_dir: Optional[str] = None
    mlir: bool = False
    components: List[str] = field(default_factory=lambda: ["Support", "Core"])
    mlir_libs: List[str] = field(default_factory=list)

@dataclass
class CatchConfig:
    package: PackageConfig
    build: BuildConfig
    dependencies: Dict[str, DependencyConfig] = field(default_factory=dict)
    cuda: Optional[CudaConfig] = None
    llvm: Optional[LlvmConfig] = None


def find_key_line_in_toml(toml_str: str, target_key: str, section: Optional[str] = None) -> Optional[int]:
    """
    Finds line number (1-indexed) of a key within an optional section in TOML text.
    """
    lines = toml_str.splitlines()
    current_section = None
    for idx, line in enumerate(lines, 1):
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            current_section = stripped[1:-1].strip()
            continue
        if "=" in line:
            key = line.split("=")[0].strip()
            if key == target_key:
                if section is None or current_section == section or (current_section and current_section.startswith(f"{section}.")):
                    return idx
    return None


def parse_config(config_path: Path) -> CatchConfig:
    if not config_path.exists():
        raise ConfigError(f"Configuration file not found: {config_path}")

    try:
        content = config_path.read_text(encoding="utf-8")
    except Exception as e:
        raise ConfigError(f"Failed to read {config_path}: {e}")

    try:
        data = tomllib.loads(content)
    except tomllib.TOMLDecodeError as e:
        # tomllib decode errors include line numbers
        raise ConfigError(f"TOML Syntax Error in {config_path}: {e}") from e

    # Helper to raise error with line number
    def error(msg: str, key: str, section: Optional[str] = None):
        line_num = find_key_line_in_toml(content, key, section)
        loc = f" line {line_num}" if line_num else ""
        sec_str = f" in [{section}]" if section else ""
        raise ConfigError(f"Validation error in {config_path.name}{sec_str}{loc} key '{key}': {msg}")

    # Validate [package]
    if "package" not in data or not isinstance(data["package"], dict):
        raise ConfigError(f"Missing or invalid [package] section in {config_path.name}")

    pkg_data = data["package"]
    if "name" not in pkg_data or not isinstance(pkg_data["name"], str) or not pkg_data["name"].strip():
        error("Field 'name' is required and must be a non-empty string", "name", "package")

    pkg_type = pkg_data.get("type", "bin")
    if pkg_type not in ("bin", "lib"):
        error(f"Invalid package type '{pkg_type}'. Expected 'bin' or 'lib'", "type", "package")

    package = PackageConfig(
        name=pkg_data["name"].strip(),
        version=str(pkg_data.get("version", "0.1.0")),
        type=pkg_type
    )

    # Validate [build]
    build_data = data.get("build", {})
    if not isinstance(build_data, dict):
        raise ConfigError(f"Invalid [build] section in {config_path.name}")

    languages = build_data.get("languages", ["CXX"])
    if not isinstance(languages, list):
        error("'languages' must be a list of strings", "languages", "build")
    allowed_langs = {"C", "CXX", "CUDA"}
    for lang in languages:
        if lang not in allowed_langs:
            error(f"Unsupported language '{lang}'. Allowed languages: C, CXX, CUDA", "languages", "build")

    build = BuildConfig(
        std=str(build_data.get("std", "20")),
        flags=list(build_data.get("flags", [])),
        defines=list(build_data.get("defines", [])),
        release_flags=list(build_data.get("release_flags", ["-O2"])),
        languages=languages
    )

    # Validate [dependencies]
    dependencies: Dict[str, DependencyConfig] = {}
    deps_data = data.get("dependencies", {})
    if not isinstance(deps_data, dict):
        raise ConfigError(f"Invalid [dependencies] section in {config_path.name}")

    for dep_name, dep_info in deps_data.items():
        if not isinstance(dep_info, dict):
            error(f"Dependency '{dep_name}' must be a table", dep_name, "dependencies")
        if "git" not in dep_info or not isinstance(dep_info["git"], str):
            error(f"Dependency '{dep_name}' missing 'git' URL", "git", f"dependencies.{dep_name}")

        dependencies[dep_name] = DependencyConfig(
            name=dep_name,
            git=dep_info["git"],
            tag=dep_info.get("tag"),
            branch=dep_info.get("branch"),
            rev=dep_info.get("rev"),
            target=dep_info.get("target")
        )

    # Validate [cuda]
    cuda: Optional[CudaConfig] = None
    if "cuda" in data or "CUDA" in build.languages:
        cuda_data = data.get("cuda", {})
        if not isinstance(cuda_data, dict):
            raise ConfigError(f"Invalid [cuda] section in {config_path.name}")
        arch = cuda_data.get("arch", ["native"])
        if isinstance(arch, str):
            arch = [arch]
        cuda = CudaConfig(
            arch=arch,
            std=str(cuda_data.get("std", "17")),
            flags=list(cuda_data.get("flags", [])),
            separable=bool(cuda_data.get("separable", False))
        )

    # Validate [llvm]
    llvm: Optional[LlvmConfig] = None
    if "llvm" in data:
        llvm_data = data["llvm"]
        if not isinstance(llvm_data, dict):
            raise ConfigError(f"Invalid [llvm] section in {config_path.name}")
        llvm = LlvmConfig(
            dir=llvm_data.get("dir"),
            mlir_dir=llvm_data.get("mlir_dir"),
            mlir=bool(llvm_data.get("mlir", False)),
            components=list(llvm_data.get("components", ["Support", "Core"])),
            mlir_libs=list(llvm_data.get("mlir_libs", []))
        )

    return CatchConfig(
        package=package,
        build=build,
        dependencies=dependencies,
        cuda=cuda,
        llvm=llvm
    )
