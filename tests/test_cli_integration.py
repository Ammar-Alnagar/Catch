import subprocess
import os
import shutil
import tomllib
from pathlib import Path

CATCH_BIN = Path(__file__).parent.parent / "target" / "debug" / "catch"

def run_catch(args, cwd=None, check=True):
    cmd = [str(CATCH_BIN)] + list(args)
    return subprocess.run(cmd, cwd=cwd, check=check, capture_output=True, text=True)

def test_cli_new_build_run_clean(tmp_path):
    # catch new demo
    res = run_catch(["new", "demo"], cwd=tmp_path)
    assert res.returncode == 0

    demo_dir = tmp_path / "demo"
    assert (demo_dir / "catch.toml").exists()
    assert (demo_dir / "src" / "main.cpp").exists()
    assert (demo_dir / ".gitignore").exists()

    # catch build
    res = run_catch(["build"], cwd=demo_dir)
    assert res.returncode == 0
    assert (demo_dir / "CMakeLists.txt").exists()
    assert (demo_dir / "build").exists()

    # catch run
    res = run_catch(["run"], cwd=demo_dir)
    assert res.returncode == 0

    # catch clean
    res = run_catch(["clean"], cwd=demo_dir)
    assert res.returncode == 0
    assert not (demo_dir / "build").exists()

def test_cli_new_cuda_single_source(tmp_path):
    res = run_catch(["new", "cuda_app", "--cuda"], cwd=tmp_path)
    assert res.returncode == 0

    src = tmp_path / "cuda_app" / "src"
    assert (src / "main.cu").exists()
    assert not (src / "main.cpp").exists()
    assert not (src / "kernel.cu").exists()
    source = (src / "main.cu").read_text(encoding="utf-8")
    assert "int main()" in source
    assert "hello_kernel<<<1, 1>>>()" in source

def test_cli_test_command(tmp_path):
    run_catch(["new", "test_proj"], cwd=tmp_path)
    proj_dir = tmp_path / "test_proj"

    tests_dir = proj_dir / "tests"
    tests_dir.mkdir()
    (tests_dir / "test_sample.cpp").write_text("""#include <cassert>
int main() {
    assert(1 + 1 == 2);
    return 0;
}
""")

    res = run_catch(["test"], cwd=proj_dir)
    assert res.returncode == 0

def test_dep_management_and_lock(tmp_path):
    run_catch(["new", "dep_test"], cwd=tmp_path)
    proj_dir = tmp_path / "dep_test"

    res = run_catch(["add", "https://github.com/fmtlib/fmt.git", "--tag", "10.1.1", "--target", "fmt::fmt"], cwd=proj_dir)
    assert res.returncode == 0

    lock_file = proj_dir / "catch.lock"
    assert lock_file.exists()
    lock_data = tomllib.loads(lock_file.read_text(encoding="utf-8"))
    assert "fmt" in lock_data["dependencies"]
    assert lock_data["dependencies"]["fmt"]["url"] == "https://github.com/fmtlib/fmt.git"
    assert len(lock_data["dependencies"]["fmt"]["commit"]) == 40

    res = run_catch(["build"], cwd=proj_dir)
    cmakelists = (proj_dir / "CMakeLists.txt").read_text(encoding="utf-8")
    assert "FetchContent_Declare(" in cmakelists
    assert "fmt" in cmakelists
    assert "GIT_REPOSITORY https://github.com/fmtlib/fmt.git" in cmakelists
    assert "elseif(TARGET fmt)" in cmakelists

    res = run_catch(["update", "fmt"], cwd=proj_dir)
    assert res.returncode == 0

    res = run_catch(["remove", "fmt"], cwd=proj_dir)
    assert res.returncode == 0
    lock_data = tomllib.loads(lock_file.read_text(encoding="utf-8"))
    assert "fmt" not in lock_data.get("dependencies", {})

def test_docs_and_doctor(tmp_path):
    run_catch(["new", "doc_proj"], cwd=tmp_path)
    proj_dir = tmp_path / "doc_proj"

    res = run_catch(["docs"], cwd=proj_dir)
    assert res.returncode == 0
    assert (proj_dir / "docs" / "PROJECT.md").exists()

    res = run_catch(["doctor"], cwd=proj_dir)
    assert res.returncode == 0
    assert "Running catch doctor diagnostic check" in res.stdout
