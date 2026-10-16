import pytest
from pathlib import Path
from catch.config import parse_config, ConfigError

def test_parse_valid_config(tmp_path):
    toml_content = """
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
"""
    config_file = tmp_path / "catch.toml"
    config_file.write_text(toml_content)

    cfg = parse_config(config_file)
    assert cfg.package.name == "test_project"
    assert cfg.package.type == "bin"
    assert cfg.build.std == "20"
    assert "CUDA" in cfg.build.languages
    assert "fmt" in cfg.dependencies
    assert cfg.dependencies["fmt"].target == "fmt::fmt"
    assert cfg.cuda is not None
    assert cfg.llvm is not None
    assert cfg.llvm.mlir is True

def test_parse_invalid_package_name(tmp_path):
    toml_content = """
[package]
version = "1.0.0"
"""
    config_file = tmp_path / "catch.toml"
    config_file.write_text(toml_content)

    with pytest.raises(ConfigError) as exc_info:
        parse_config(config_file)
    assert "name" in str(exc_info.value)

def test_parse_invalid_language(tmp_path):
    toml_content = """
[package]
name = "invalid_lang"

[build]
languages = ["INVALID"]
"""
    config_file = tmp_path / "catch.toml"
    config_file.write_text(toml_content)

    with pytest.raises(ConfigError) as exc_info:
        parse_config(config_file)
    assert "languages" in str(exc_info.value)
    assert "line 6" in str(exc_info.value)
