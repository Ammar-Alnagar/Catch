# Catch: Cargo-like C++ Build Tool

`catch` is a simple, modern, Cargo-like command-line tool for C++ projects written in Rust that wraps CMake and Ninja. It enables C++ developers to declare project settings, build options, external dependencies, CUDA options, and LLVM/MLIR integration exclusively through a single `catch.toml` file.

With `catch`, you no longer need to write complex `CMakeLists.txt` boilerplate by hand. `catch` automatically generates clean, standard `CMakeLists.txt` files and manages build, run, test, dependency locking, documentation generation, and environment diagnostics workflows.

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
- [Configuration Reference (`catch.toml`)](#configuration-reference-catchtoml)
- [Detailed Usage Examples & Use Cases](#detailed-usage-examples--use-cases)
  - [1. Creating and Running a Basic C++ Binary Project](#1-creating-and-running-a-basic-c-binary-project)
  - [2. Creating a C++ Static/Shared Library Project](#2-creating-a-c-staticshared-library-project)
  - [3. Managing External Git Dependencies (`catch add`, `catch update`, `catch remove`)](#3-managing-external-git-dependencies-catch-add-catch-update-catch-remove)
  - [4. Writing and Executing Unit Tests (`catch test`)](#4-writing-and-executing-unit-tests-catch-test)
  - [5. CUDA Project Development (`catch new --cuda`)](#5-cuda-project-development-catch-new---cuda)
  - [6. LLVM and MLIR Compiler Projects (`catch new --mlir`)](#6-llvm-and-mlir-compiler-projects-catch-new---mlir)
  - [7. Automated Markdown Documentation Generation (`catch docs`)](#7-automated-markdown-documentation-generation-catch-docs)
  - [8. System Toolchain Diagnostic Check (`catch doctor`)](#8-system-toolchain-diagnostic-check-catch-doctor)
  - [9. Building Existing CMake Projects (`catch cmake`)](#9-building-existing-cmake-projects-catch-cmake)
- [License](#license)

---

## Features

- **Cargo-like Workflow**: Simple CLI for creating, building, running, testing, and cleaning C++ projects.
- **Declarative Configuration**: Manage all settings in `catch.toml` without touching `CMakeLists.txt`.
- **Existing CMake Support**: Detect and build existing CMake projects (`catch cmake`) without requiring `catch.toml`.
- **Git Dependency Management**: Add git dependencies with precise refs (`--tag`, `--branch`, `--rev`), target overrides, and deterministic locking (`catch.lock`).
- **Built-in Testing**: Automated CTest test suite generation for files placed under `tests/*.cpp`.
- **CUDA & LLVM/MLIR Support**: First-class flags for GPU acceleration and compiler construction.
- **Doc Generation**: Extract project layout, settings, targets, and dependencies directly into `docs/PROJECT.md`.
- **Doctor Diagnostic**: Quickly verify system dependencies (`cmake`, `ninja`, `git`, `nvcc`) and configured LLVM/MLIR directory paths.

---

## Installation

### Requirements

- Rust (Cargo) 1.70+
- CMake 3.20+
- Ninja (optional, recommended)
- Git (for dependency management)

### Install Locally

Clone the repository and install using `cargo`:

```sh
git clone https://github.com/your-org/catch.git
cd catch
cargo install --path .
```

This compiles and registers the `catch` executable globally on your system PATH.

---

## Quick Start

```sh
# 1. Create a new project
catch new my_app
cd my_app

# 2. Add an external dependency
catch add https://github.com/fmtlib/fmt.git --tag 10.1.1 --target fmt::fmt

# 3. Build and run
catch build
catch run

# 4. Run tests
catch test

# 5. Generate documentation
catch docs
```

---

## Command Reference

`catch` provides a set of intuitive subcommands:

### `catch new <name> [--cuda] [--mlir] [--force]`
Creates a new C++ project in a directory named `<name>`.
- `--cuda`: Configures the project for CUDA support (adds `CUDA` language, `[cuda]` section, and a single `src/main.cu`).
- `--mlir`: Configures the project for LLVM/MLIR support (adds `[llvm]` section and MLIR context template).
- `--force`, `-f`: Overwrites existing files in the directory without interactive confirmation.

### `catch build [--release] [--verbose]`
Generates `CMakeLists.txt` (if missing or outdated) and compiles the project using CMake and Ninja (or standard build toolchain).
- `--release`: Compiles with release optimizations (`CMAKE_BUILD_TYPE=Release`).
- `--verbose`, `-v`: Enables verbose output from build tools.

### `catch run [--release] [--verbose] [-- <args...>]`
Builds and runs the binary executable (only for `type = "bin"` packages). Arguments following `--` are passed directly to the compiled executable.
- Example: `catch run -- --flag value input.txt`

### `catch test [--release] [--verbose]`
Builds the project and executes the test suite using CTest. Each `.cpp` file in `tests/` is compiled as a separate test binary.

### `catch clean`
Deletes the `build/` directory and compiled artifacts.

### `catch add <url> [--tag T | --branch B | --rev R] [--name N] [--target X::Y]`
Adds a git dependency to `catch.toml` and locks the commit hash in `catch.lock`.
- `url`: Repository Git URL (e.g. `https://github.com/fmtlib/fmt.git`).
- `--tag`, `-t`: Specify a Git tag.
- `--branch`, `-b`: Specify a Git branch.
- `--rev`, `-r`: Specify an exact Git revision/commit hash.
- `--name`, `-n`: Dependency name override (defaults to repository name).
- `--target`: CMake target name override (e.g. `fmt::fmt` or `nlohmann_json::nlohmann_json`).

### `catch remove <name>`
Removes a dependency from `catch.toml` and deletes its locked entry in `catch.lock`.

### `catch update [name]`
Re-resolves git commit hashes for specified dependency (or all dependencies if no name given) and updates `catch.lock`.

### `catch docs [--stdout]`
Generates Markdown project documentation summarizing metadata, source layout, targets, dependencies, and settings.
- `--stdout`: Output generated Markdown to `stdout` instead of saving to `docs/PROJECT.md`.

### `catch doctor`
Runs diagnostic checks on required and optional tools (`cmake`, `ninja`, `git`, `nvcc`) and validates configured LLVM/MLIR build paths.

### `catch cmake [path] [--release] [-t, --target <name>] [--build-dir <dir>] [-- <cmake_args...>]`
Detects an existing `CMakeLists.txt` (in current directory, parent directories, or specified path) and builds it using CMake and Ninja without requiring `catch.toml`.
- `path`: Path to directory containing `CMakeLists.txt` or `CMakeLists.txt` file itself (defaults to current directory, traversing upwards if not found).
- `--release`: Compiles in Release mode (`CMAKE_BUILD_TYPE=Release`).
- `-t`, `--target`: Specific CMake target to compile.
- `--build-dir`: Build output directory (default: `build`).
- `-- <cmake_args...>`: Arbitrary extra arguments passed directly to CMake configuration.

---

## Configuration Reference (`catch.toml`)

All project behavior is configured inside `catch.toml`. Here is a complete breakdown of supported sections and keys:

### `[package]`
Defines basic package metadata.
- `name` *(string, required)*: The project name and target executable/library name.
- `version` *(string, default: `"0.1.0"`)*: Package version.
- `type` *(string, default: `"bin"`)*: Package type (`"bin"` for executables, `"lib"` for static/shared libraries).

```toml
[package]
name = "my_project"
version = "1.0.0"
type = "bin"
```

### `[build]`
Defines build settings and flags.
- `std` *(string, default: `"20"`)*: C++ standard version (e.g. `"17"`, `"20"`, `"23"`).
- `flags` *(list of strings, default: `[]`)*: Compiler flags for all build types (e.g. `["-Wall", "-Wextra"]`).
- `defines` *(list of strings, default: `[]`)*: Preprocessor macro definitions (e.g. `["ENABLE_LOGGING", "BUFFER_SIZE=1024"]`).
- `release_flags` *(list of strings, default: `["-O2"]`)*: Additional compiler flags used during release builds.
- `languages` *(list of strings, default: `["CXX"]`)*: Languages required by CMake (`"C"`, `"CXX"`, `"CUDA"`).

```toml
[build]
std = "20"
flags = ["-Wall", "-Wextra"]
defines = ["DEBUG_MODE"]
release_flags = ["-O3", "-DNDEBUG"]
languages = ["CXX"]
```

### `[dependencies]`
Table specifying external git dependencies brought in via CMake `FetchContent`.
- `git` *(string, required)*: Git clone URL.
- `tag` *(string, optional)*: Git tag name.
- `branch` *(string, optional)*: Git branch name.
- `rev` *(string, optional)*: Git revision / commit hash.
- `target` *(string, optional)*: CMake target to link against (defaults to `<dep_name>::<dep_name>`).

```toml
[dependencies]
fmt = { git = "https://github.com/fmtlib/fmt.git", tag = "10.1.1", target = "fmt::fmt" }
nlohmann_json = { git = "https://github.com/nlohmann/json.git", tag = "v3.11.2", target = "nlohmann_json::nlohmann_json" }
```

### `[cuda]`
*(Optional)* CUDA compilation settings. Automatically populated when running `catch new --cuda`.
- `arch` *(list of strings, default: `["native"]`)*: CUDA target architecture architectures (e.g. `["native"]` or `["75", "80", "86"]`).
- `std` *(string, default: `"17"`)*: CUDA C++ standard version.
- `flags` *(list of strings, default: `[]`)*: NVCC flags (e.g. `["--expt-relaxed-constexpr"]`).
- `separable` *(boolean, default: `false`)*: Enables `CUDA_SEPARABLE_COMPILATION`.

```toml
[cuda]
arch = ["native"]
std = "17"
flags = ["--expt-relaxed-constexpr"]
separable = false
```

### `[llvm]`
*(Optional)* LLVM and MLIR integration settings. Automatically populated when running `catch new --mlir`.
- `dir` *(string, optional)*: Path to CMake configuration directory inside LLVM build (`/path/to/llvm-build/lib/cmake/llvm`).
- `mlir_dir` *(string, optional)*: Path to CMake configuration directory inside MLIR build (`/path/to/llvm-build/lib/cmake/mlir`).
- `mlir` *(boolean, default: `false`)*: Enables MLIR CMake targets.
- `components` *(list of strings, default: `["Support", "Core"]`)*: LLVM components to link.
- `mlir_libs` *(list of strings, default: `[]`)*: MLIR libraries to link (e.g. `["MLIRIR", "MLIRParser", "MLIRSupport"]`).

```toml
[llvm]
dir = "/usr/lib/cmake/llvm"
mlir_dir = "/usr/lib/cmake/mlir"
mlir = true
components = ["Support", "Core", "ExecutionEngine"]
mlir_libs = ["MLIRIR", "MLIRParser", "MLIRSupport"]
```

---

## Detailed Usage Examples & Use Cases

### 1. Creating and Running a Basic C++ Binary Project

#### Step 1: Create Project
```sh
catch new hello_world
cd hello_world
```

#### Generated Structure:
```
hello_world/
├── catch.toml
├── .gitignore
└── src/
    └── main.cpp
```

#### File Contents:
`catch.toml`:
```toml
[package]
name = "hello_world"
version = "0.1.0"
type = "bin"

[build]
std = "20"
flags = []
defines = []
release_flags = ["-O2"]
languages = ["CXX"]
```

`src/main.cpp`:
```cpp
#include <iostream>

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
```

#### Step 2: Build and Run
```sh
catch build
catch run
```

#### Output:
```text
Hello, World!
```

#### Passing Runtime Arguments:
```sh
catch run -- --greeting "Hello C++" --count 3
```

---

### 2. Creating a C++ Static/Shared Library Project

`catch` can build reusable libraries for C++ projects.

#### Step 1: Initialize Project & Edit `catch.toml`
Create a project and set `type = "lib"` in `catch.toml`:

```sh
catch new my_math_lib
cd my_math_lib
```

Modify `catch.toml`:
```toml
[package]
name = "my_math_lib"
version = "1.0.0"
type = "lib"

[build]
std = "20"
flags = ["-Wall"]
defines = []
release_flags = ["-O3"]
languages = ["CXX"]
```

#### Step 2: Add Header and Source Files
Create `include/math_utils.hpp` and `src/math_utils.cpp`:

`include/math_utils.hpp`:
```cpp
#ifndef MATH_UTILS_HPP
#define MATH_UTILS_HPP

namespace math {
    int add(int a, int b);
    int multiply(int a, int b);
}

#endif
```

`src/math_utils.cpp`:
```cpp
#include "math_utils.hpp"

namespace math {
    int add(int a, int b) { return a + b; }
    int multiply(int a, int b) { return a * b; }
}
```

#### Step 3: Build the Library
```sh
catch build --release
```
`catch` generates a CMake configuration building `libmy_math_lib.a` (or `.lib` on Windows) inside `build/`.

---

### 3. Managing External Git Dependencies (`catch add`, `catch update`, `catch remove`)

`catch` provides integrated dependency management wrapping CMake's `FetchContent` and maintaining commit hashes in `catch.lock`.

#### Step 1: Add Dependencies
Add `fmt` and `nlohmann_json`:

```sh
catch add https://github.com/fmtlib/fmt.git --tag 10.1.1 --target fmt::fmt
catch add https://github.com/nlohmann/json.git --tag v3.11.2 --target nlohmann_json::nlohmann_json
```

#### Resulting `catch.toml`:
```toml
[package]
name = "demo_app"
version = "0.1.0"
type = "bin"

[build]
std = "20"
flags = []
defines = []
release_flags = ["-O2"]
languages = ["CXX"]

[dependencies]
fmt = { git = "https://github.com/fmtlib/fmt.git", tag = "10.1.1", target = "fmt::fmt" }
nlohmann_json = { git = "https://github.com/nlohmann/json.git", tag = "v3.11.2", target = "nlohmann_json::nlohmann_json" }
```

#### Generated `catch.lock`:
```toml
# Generated by catch. Do not edit directly.
[dependencies.fmt]
commit = "a3370fc305e0281b32d2c16c3104617150a58145"

[dependencies.nlohmann_json]
commit = "9bccb39e2a0174c10d401311be6f1c4df27c95a2"
```

#### Step 2: Use Dependencies in Code (`src/main.cpp`)
```cpp
#include <fmt/core.h>
#include <nlohmann/json.hpp>
#include <iostream>

int main() {
    nlohmann::json j = {{"project", "catch"}, {"status", "awesome"}};
    fmt::print("JSON output: {}\n", j.dump());
    return 0;
}
```

#### Step 3: Updating or Removing Dependencies
Update `fmt` to re-resolve the latest commit hash for its tag/branch:
```sh
catch update fmt
```

Remove a dependency:
```sh
catch remove nlohmann_json
```

---

### 4. Writing and Executing Unit Tests (`catch test`)

`catch` automatically turns any `.cpp` file placed inside the `tests/` directory into a separate test executable registered with CTest.

#### Step 1: Project Setup
```sh
catch new test_demo
cd test_demo
mkdir tests
```

#### Step 2: Create Test Files
Create `tests/test_math.cpp`:
```cpp
#include <cassert>
#include <iostream>

int add(int a, int b) { return a + b; }

int main() {
    assert(add(2, 3) == 5);
    assert(add(-1, 1) == 0);
    std::cout << "Math tests passed!" << std::endl;
    return 0;
}
```

Create `tests/test_strings.cpp`:
```cpp
#include <cassert>
#include <string>

int main() {
    std::string s = "catch";
    assert(s.length() == 5);
    return 0;
}
```

#### Step 3: Run Tests
```sh
catch test
```

#### CTest Output:
```text
Test project /path/to/test_demo/build
    Start 1: test_math
1/2 Test #1: test_math ........................   Passed    0.00 sec
    Start 2: test_strings
2/2 Test #2: test_strings .....................   Passed    0.00 sec

100% tests passed, 0 tests failed out of 2
```

---

### 5. CUDA Project Development (`catch new --cuda`)

`catch` supports GPU development using NVIDIA CUDA.

#### Step 1: Create CUDA Project
```sh
catch new cuda_app --cuda
cd cuda_app
```

#### Generated Layout:
```
cuda_app/
├── catch.toml
├── .gitignore
└── src/
    └── main.cu
```

#### Configured `catch.toml`:
```toml
[package]
name = "cuda_app"
version = "0.1.0"
type = "bin"

[build]
std = "20"
flags = []
defines = []
release_flags = ["-O2"]
languages = ["CXX", "CUDA"]

[cuda]
arch = ["native"]
std = "17"
flags = []
```

#### Sample program (`src/main.cu`):
```cuda
#include <cstdio>
#include <cuda_runtime.h>

__global__ void hello_kernel() {
    printf("Hello from GPU thread %d\n", threadIdx.x);
}

int main() {
    hello_kernel<<<1, 1>>>();
    cudaDeviceSynchronize();
    return 0;
}
```

#### Build and Run:
```sh
catch build
catch run
```

---

### 6. LLVM and MLIR Compiler Projects (`catch new --mlir`)

`catch` streamlines building LLVM and MLIR compiler plugins and tooling.

#### Step 1: Create MLIR Project
```sh
catch new mlir_compiler --mlir
cd mlir_compiler
```

#### Configured `catch.toml`:
```toml
[package]
name = "mlir_compiler"
version = "0.1.0"
type = "bin"

[build]
std = "20"
flags = []
defines = []
release_flags = ["-O2"]
languages = ["CXX"]

[llvm]
dir = "/path/to/llvm-build/lib/cmake/llvm"
mlir_dir = "/path/to/llvm-build/lib/cmake/mlir"
mlir = true
components = ["Support", "Core"]
mlir_libs = ["MLIRIR", "MLIRParser", "MLIRSupport"]
```

#### Update Paths & Code (`src/main.cpp`):
Set `dir` and `mlir_dir` in `catch.toml` to your local LLVM/MLIR installation/build directories.

`src/main.cpp`:
```cpp
#include "mlir/IR/MLIRContext.h"
#include "mlir/IR/BuiltinOps.h"
#include <iostream>

int main() {
    mlir::MLIRContext context;
    std::cout << "MLIR Context initialized successfully!" << std::endl;
    return 0;
}
```

#### Build and Run:
```sh
catch build
catch run
```

---

### 7. Automated Markdown Documentation Generation (`catch docs`)

Keep your project documentation in sync with your codebase without manual editing.

#### Command:
```sh
catch docs
```

#### Output File (`docs/PROJECT.md`):
`catch docs` produces a complete, structured Markdown overview of your project, code metrics, pulled libraries with metadata, and extracted functions/symbols:

```markdown
<!-- Generated by catch docs. Do not edit directly. -->
# Project: demo_app

- **Version**: `0.1.0`
- **Type**: `bin`

## Project Details & Code Metrics
- **Total Source Files**: 2 files
- **Total Lines of Code**: 48 lines (34 code, 8 comments, 6 blank)
- **Total Source Tree Size**: 1.4 KB
- **Built Artifact**: `build/demo_app` (48.2 KB)
- **Build Directory**: `build/` (2.4 MB across 42 files)

## Languages and Standard
- **Languages**: CXX
- **C++ Standard**: `20`
- **Compile Flags**: `[]`
- **Defines**: `[]`
- **Release Flags**: `['-O2']`

## Source Tree
### Source Files (`src/`)
- `src/main.cpp`
### Test Files (`tests/`)
- `tests/test_main.cpp`

## Targets
- **Main Target**: `demo_app` (bin)
- **Test Executables**:
  - `test_test_main` (from `tests/test_main.cpp`)

## Dependencies & Pulled Libraries
### Library: `fmt`
- **Repository URL**: `https://github.com/fmtlib/fmt.git`
- **Requested Ref**: `10.1.1`
- **Locked Commit**: `a3370fc305e0281b32d2c16c3104617150a58145`
- **CMake Target**: `fmt::fmt`
- **Fetch Status**: `Fetched & Cached`
- **Local Cache Path**: `build/_deps/fmt-src`
- **Pulled Size**: 2.3 MB across 142 files
- **Headers Found**: 28 header files
- **Detected License**: `Copyright (c) 2012 - present, Victor Zverovich`
- **Description**: A modern formatting library

## Source Code Analysis & API Overview

| File | Total Lines | Code Lines | Comments | Blank | Size | Symbols |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| `src/main.cpp` | 32 | 22 | 6 | 4 | 920 B | 2 |
| `tests/test_main.cpp` | 16 | 12 | 2 | 2 | 480 B | 1 |

### Extracted Functions & Symbols

#### `src/main.cpp` (32 lines, 920 B)
- **Class** `Calculator` *(line 8)*:
```cpp
class Calculator
```
  *Doc*: Calculator class providing arithmetic operations.

- **Function** `main` *(line 20)*:
```cpp
int main(int argc, char* argv[])
```
  *Doc*: Entry point of the application.

## Build Commands
Commands run by `catch` for this project:
```sh
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
cmake --build build
ctest --test-dir build --output-on-failure
```
```

---

### 8. System Toolchain Diagnostic Check (`catch doctor`)

Verify your environment before building to prevent missing dependency errors.

#### Command:
```sh
catch doctor
```

#### Sample Output:
```text
Running catch doctor diagnostic check...

[✓] cmake   : FOUND
    Reason: Required for project build file generation and build orchestration.

[✓] ninja   : FOUND
    Reason: Optional fast build tool. Recommended generator for CMake.

[✓] git     : FOUND
    Reason: Required for managing and fetching git dependencies.

[!] nvcc    : MISSING (Optional)
    Reason: CUDA compiler driver. Required if building CUDA code.

Checking [llvm] configuration from catch.toml...
[✓] LLVM_DIR  : FOUND (/usr/lib/cmake/llvm)
[✓] MLIR_DIR  : FOUND (/usr/lib/cmake/mlir)

All required toolchain components are ready.
```

---

### 9. Building Existing CMake Projects (`catch cmake`)

`catch cmake` allows you to seamlessly build existing CMake projects that already have their own `CMakeLists.txt`, without needing a `catch.toml` file. It automatically discovers the `CMakeLists.txt` in the current directory or traverses parent directories, configures with Ninja (if available), and compiles your targets.

#### Basic Usage:
```sh
# Build the CMake project in the current directory (or nearest parent directory)
catch cmake

# Build in Release mode
catch cmake --release

# Build a specific target
catch cmake --target my_target

# Build a project located in another folder
catch cmake ./libs/my_legacy_project

# Pass custom CMake configuration arguments
catch cmake -- -DCMAKE_EXPORT_COMPILE_COMMANDS=ON -DENABLE_CUSTOM_FEATURE=ON
```

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
