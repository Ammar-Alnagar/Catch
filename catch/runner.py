import sys
import subprocess
import shlex

class RunnerError(Exception):
    pass

def run_command(cmd, cwd=None, verbose=False, check=True, capture_output=False, env=None):
    """
    Executes a command list using subprocess.
    Prints the real command being run.
    """
    cmd_str = " ".join(shlex.quote(str(arg)) for arg in cmd)
    print(f"Running: {cmd_str}")
    try:
        res = subprocess.run(
            cmd,
            cwd=cwd,
            check=check,
            capture_output=capture_output,
            text=True,
            env=env
        )
        return res
    except subprocess.CalledProcessError as e:
        print(f"Error executing command: {cmd_str}", file=sys.stderr)
        if e.stdout and verbose:
            print(f"Stdout:\n{e.stdout}", file=sys.stderr)
        if e.stderr:
            print(f"Stderr:\n{e.stderr}", file=sys.stderr)
        raise RunnerError(f"Command failed with exit code {e.returncode}: {cmd_str}") from e
    except FileNotFoundError as e:
        print(f"Error: Command not found '{cmd[0]}'", file=sys.stderr)
        raise RunnerError(f"Executable not found: {cmd[0]}") from e
