"""
Code templates for `catch new`
"""

CATCH_TOML_TEMPLATE = """[package]
name = "{name}"
version = "0.1.0"
type = "bin"

[build]
std = "20"
flags = []
defines = []
release_flags = ["-O2"]
languages = {languages}
{cuda_section}{llvm_section}"""

MAIN_CPP_TEMPLATE = """#include <iostream>

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
"""

MLIR_MAIN_CPP_TEMPLATE = """#include "mlir/IR/MLIRContext.h"
#include "mlir/IR/BuiltinOps.h"
#include <iostream>

int main() {
    mlir::MLIRContext context;
    std::cout << "MLIR Context initialized successfully!" << std::endl;
    return 0;
}
"""

KERNEL_CU_TEMPLATE = """#include <iostream>
#include <cuda_runtime.h>

__global__ void hello_kernel() {
    // CUDA Kernel
}

void run_kernel() {
    hello_kernel<<<1, 1>>>();
    cudaDeviceSynchronize();
}
"""

GITIGNORE_TEMPLATE = """build/
.cache/
*.o
*.a
*.so
*.dylib
*.exe
"""
