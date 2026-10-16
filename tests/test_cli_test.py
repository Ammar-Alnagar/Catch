import pytest
import os
from pathlib import Path
import catch.cli as cli

def test_catch_test_and_extra_cmake(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)

    # Create new project
    class ArgsNew:
        name = "testproj"
        cuda = False
        mlir = False
        force = True

    cli.cmd_new(ArgsNew())
    proj_dir = tmp_path / "testproj"
    monkeypatch.chdir(proj_dir)

    # Add tests directory and test files
    tests_dir = proj_dir / "tests"
    tests_dir.mkdir()
    (tests_dir / "test_math.cpp").write_text("""
#include <cassert>
int main() {
    assert(1 + 1 == 2);
    return 0;
}
""")
    (tests_dir / "test_extra.cpp").write_text("""
#ifndef EXTRA_FLAG
#error "EXTRA_FLAG not defined!"
#endif
int main() {
    return 0;
}
""")

    # Add extra.cmake
    (proj_dir / "extra.cmake").write_text("""
add_compile_definitions(EXTRA_FLAG=1)
""")

    # Test catch test command
    class ArgsTest:
        release = False
        verbose = False

    cli.cmd_test(ArgsTest())
