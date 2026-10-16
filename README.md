# Catch: Cargo-like C++ Build Tool

`catch` is a simple, Cargo-like command-line tool for C++ projects that wraps CMake. The user manages project settings exclusively through `catch.toml`, while `catch` generates `CMakeLists.txt` and manages build, run, test, dependency locking, documentation, and diagnostics workflows.

## Installation

Install locally in editable mode:
```sh
pip install -e .
```

This registers the `catch` executable script globally on your system.

## Usage & Commands

- `catch new <name> [--cuda] [--mlir]`: Create a new C++ project directory with `catch.toml`, `src/main.cpp`, and `.gitignore`.
- `catch build [--release]`: Regenerates `CMakeLists.txt` and compiles the project using CMake and Ninja (if present).
- `catch run [-- args]`: Builds and executes the main package executable.
- `catch test`: Builds and runs test suite via CTest (`enable_testing()` and one executable per `tests/*.cpp` file).
- `catch clean`: Removes `build/` directory.
- `catch add <git-url> [--tag T | --branch B | --rev R] [--name N] [--target X::Y]`: Adds a git dependency to `catch.toml` and locks the commit in `catch.lock`.
- `catch remove <name>`: Removes a dependency.
- `catch update [name]`: Re-resolves git commit hashes in `catch.lock`.
- `catch docs [--stdout]`: Generates `docs/PROJECT.md` documentation.
- `catch doctor`: Checks system toolchain setup (cmake, ninja, git, nvcc, LLVM/MLIR).

## Example Flow

```sh
catch new demo
cd demo
catch add https://github.com/fmtlib/fmt.git --tag 10.1.1 --target fmt::fmt
catch build
catch run
catch test
catch docs
```
