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

pub const MAIN_CU_TEMPLATE: &str = r#"#include <cstdio>
#include <cuda_runtime.h>

__global__ void hello_kernel() {
    printf("Hello from GPU thread %d\n", threadIdx.x);
}

int main() {
    hello_kernel<<<1, 1>>>();
    cudaDeviceSynchronize();
    return 0;
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
