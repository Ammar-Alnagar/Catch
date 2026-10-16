import pytest
from pathlib import Path
from catch.config import parse_config
from catch.generator import generate_cmakelists

def test_llvm_mlir_cmakelists_generation(tmp_path):
    project_dir = tmp_path / "mlir_proj"
    project_dir.mkdir()
    (project_dir / "src").mkdir()
    (project_dir / "src" / "main.cpp").write_text("int main(){}\n")

    toml_content = """[package]
name = "mlir_proj"
version = "0.1.0"
type = "bin"

[build]
std = "20"
languages = ["CXX"]

[llvm]
dir = "/usr/lib/cmake/llvm"
mlir_dir = "/usr/lib/cmake/mlir"
mlir = true
components = ["Support", "Core"]
mlir_libs = ["MLIRIR", "MLIRParser", "MLIRSupport"]
"""
    (project_dir / "catch.toml").write_text(toml_content)
    cfg = parse_config(project_dir / "catch.toml")
    cmake_out = generate_cmakelists(cfg, project_dir)

    assert "find_package(LLVM REQUIRED CONFIG)" in cmake_out
    assert 'message(STATUS "Using LLVMConfig.cmake in ${LLVM_DIR}")' in cmake_out
    assert "find_package(MLIR REQUIRED CONFIG)" in cmake_out
    assert 'message(STATUS "Using MLIRConfig.cmake in ${MLIR_DIR}")' in cmake_out
    assert "include(TableGen)" in cmake_out
    assert "include(AddLLVM)" in cmake_out
    assert "include(AddMLIR)" in cmake_out
    assert "include(HandleLLVMOptions)" in cmake_out
    assert "include_directories(${LLVM_INCLUDE_DIRS})" in cmake_out
    assert "include_directories(${MLIR_INCLUDE_DIRS})" in cmake_out
    assert "add_definitions(${LLVM_DEFINITIONS})" in cmake_out
    assert "if(NOT LLVM_ENABLE_RTTI)" in cmake_out
    assert "add_compile_options(-fno-rtti)" in cmake_out
    assert "llvm_map_components_to_libnames(llvm_libs Support Core)" in cmake_out
    assert "target_link_libraries(mlir_proj PRIVATE ${llvm_libs})" in cmake_out
    assert "target_link_libraries(mlir_proj PRIVATE MLIRIR MLIRParser MLIRSupport)" in cmake_out
