"""Code quality commands for Arawn."""

import subprocess

import angreal
from angreal.integrations.flox import Flox

check = angreal.command_group(name="check", about="Code quality checks")


@check()
@angreal.command(name="all", about="Run all checks, auto-fixing where possible")
@angreal.argument(name="check_only", long="check-only", takes_value=False, help="Only check, don't fix")
def check_all(check_only=False):
    """Run fmt, clippy, and cargo check. Auto-fixes by default."""
    apply = not check_only
    with Flox("."):
        _run_fmt(apply)
        _run_clippy(apply)
        subprocess.run(["cargo", "check", "--workspace"], check=True)


@check()
@angreal.command(name="workspace", about="Run cargo check")
def check_workspace():
    """Run cargo check on the workspace."""
    with Flox("."):
        subprocess.run(["cargo", "check", "--workspace"], check=True)


@check()
@angreal.command(name="fmt", about="Format code")
@angreal.argument(name="check_only", long="check-only", takes_value=False, help="Only check, don't fix")
def check_fmt(check_only=False):
    """Format code with rustfmt. Auto-fixes by default."""
    with Flox("."):
        _run_fmt(not check_only)


@check()
@angreal.command(name="clippy", about="Run clippy lints")
@angreal.argument(name="check_only", long="check-only", takes_value=False, help="Only check, don't fix")
def check_clippy(check_only=False):
    """Run clippy on the workspace. Auto-fixes by default."""
    with Flox("."):
        _run_clippy(not check_only)


def _run_fmt(apply_fix):
    if apply_fix:
        subprocess.run(["cargo", "fmt", "--all"], check=True)
    else:
        subprocess.run(["cargo", "fmt", "--all", "--", "--check"], check=True)


def _run_clippy(apply_fix):
    cmd = ["cargo", "clippy", "--workspace"]
    if apply_fix:
        cmd.append("--fix")
        cmd.append("--allow-dirty")
    cmd.extend(["--", "-D", "warnings"])
    subprocess.run(cmd, check=True)
