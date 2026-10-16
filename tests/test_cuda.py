import pytest
import shutil
from pathlib import Path
from catch.config import parse_config
from catch.generator import generate_cmakelists
import catch.cli as cli

def test_cuda_cmakelists_generation(tmp_path):
    project_dir = tmp_path / "cuda_proj"
    project_dir.mkdir()
    (project_dir / "src").mkdir()
    (project_dir / "src" / "main.cpp").write_text("int main(){}\n")
    (project_dir / "src" / "kernel.cu").write_text("// cuda kernel\n")

    toml_content = """[package]
name = "cuda_proj"
version = "0.1.0"
type = "bin"

[build]
std = "20"
languages = ["CXX", "CUDA"]

[cuda]
arch = ["native", "80"]
std = "17"
flags = ["-Xcompiler", "-Wall"]
separable = true
"""
    (project_dir / "catch.toml").write_text(toml_content)
    cfg = parse_config(project_dir / "catch.toml")
    cmake_out = generate_cmakelists(cfg, project_dir)

    assert "enable_language(CUDA)" in cmake_out
    assert "set(CMAKE_CUDA_STANDARD 17)" in cmake_out
    assert 'set(CMAKE_CUDA_ARCHITECTURES "native;80")' in cmake_out
    assert "set(CMAKE_CUDA_SEPARABLE_COMPILATION ON)" in cmake_out
    assert "CUDA::cudart" in cmake_out
    assert "src/kernel.cu" in cmake_out


@pytest.mark.skipif(shutil.which("nvcc") is None, reason="nvcc compiler not found in PATH")
def test_cuda_build_and_run(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)

    class ArgsNew:
        name = "cuda_demo"
        cuda = True
        mlir = False
        force = True

    cli.cmd_new(ArgsNew())
    proj_dir = tmp_path / "cuda_demo"
    monkeypatch.chdir(proj_dir)

    class ArgsBuild:
        release = False
        verbose = True

    cli.cmd_build(ArgsBuild())
    assert (proj_dir / "build").exists()
