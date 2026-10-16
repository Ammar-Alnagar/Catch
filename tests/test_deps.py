import pytest
import os
import tomllib
from pathlib import Path
import catch.cli as cli

def test_dep_management_and_lock(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)

    # Create new project
    class ArgsNew:
        name = "dep_test"
        cuda = False
        mlir = False
        force = True

    cli.cmd_new(ArgsNew())
    proj_dir = tmp_path / "dep_test"
    monkeypatch.chdir(proj_dir)

    # Test catch add fmt
    class ArgsAdd:
        url = "https://github.com/fmtlib/fmt.git"
        tag = "10.1.1"
        branch = None
        rev = None
        name = "fmt"
        target = "fmt::fmt"

    cli.cmd_add(ArgsAdd())

    lock_file = proj_dir / "catch.lock"
    assert lock_file.exists()
    lock_data = tomllib.loads(lock_file.read_text(encoding="utf-8"))
    assert "fmt" in lock_data["dependencies"]
    assert lock_data["dependencies"]["fmt"]["url"] == "https://github.com/fmtlib/fmt.git"
    assert len(lock_data["dependencies"]["fmt"]["commit"]) == 40

    # Ensure generated CMakeLists.txt contains FetchContent for fmt
    cli.ensure_cmakelists(proj_dir)
    cmakelists = (proj_dir / "CMakeLists.txt").read_text(encoding="utf-8")
    assert "FetchContent_Declare(" in cmakelists
    assert "fmt" in cmakelists
    assert "GIT_REPOSITORY https://github.com/fmtlib/fmt.git" in cmakelists

    # Test catch update
    class ArgsUpdate:
        name = "fmt"

    cli.cmd_update(ArgsUpdate())
    assert lock_file.exists()

    # Test catch remove
    class ArgsRemove:
        name = "fmt"

    cli.cmd_remove(ArgsRemove())
    lock_data = tomllib.loads(lock_file.read_text(encoding="utf-8"))
    assert "fmt" not in lock_data.get("dependencies", {})
