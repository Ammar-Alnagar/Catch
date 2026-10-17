pub const CATCH_TOML_TEMPLATE: &str = r#"[package]
name = "{name}"
version = "0.1.0"
type = "bin"

[build]
std = "20"
flags = ["-O3", "-DNDEBUG"]
defines = []
release_flags = ["-O3", "-DNDEBUG"]
languages = {languages}
{cuda_section}{llvm_section}"#;

pub const MAIN_CPP_TEMPLATE: &str = r#"#include <iostream>

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
"#;

pub const MLIR_MAIN_CPP_TEMPLATE: &str = r#"#include "mlir/IR/MLIRContext.h"
#include "mlir/IR/BuiltinOps.h"
#include <iostream>

int main() {
    mlir::MLIRContext context;
    std::cout << "MLIR Context initialized successfully!" << std::endl;
    return 0;
}
"#;

pub const KERNEL_CU_TEMPLATE: &str = r#"#include <iostream>
#include <cuda_runtime.h>

__global__ void hello_kernel() {
    // CUDA Kernel
}

void run_kernel() {
    hello_kernel<<<1, 1>>>();
    cudaDeviceSynchronize();
}
"#;

pub const GITIGNORE_TEMPLATE: &str = r#"build/
.cache/
*.o
*.a
*.so
*.dylib
*.exe
"#;
