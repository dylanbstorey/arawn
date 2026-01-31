"""Test commands for Arawn."""

import os
import subprocess

import angreal
from angreal.integrations.flox import Flox

test = angreal.command_group(name="test", about="Run tests")


@test()
@angreal.command(name="all", about="Run all tests (workspace + runtimes)")
def test_all():
    """Run workspace tests then runtime tests."""
    with Flox("."):
        _run_unit()
        _run_runtimes()


@test()
@angreal.command(name="unit", about="Run workspace unit tests")
def test_unit():
    """Run cargo test across the workspace."""
    with Flox("."):
        _run_unit()


@test()
@angreal.command(name="runtimes", about="Run runtime tests individually")
def test_runtimes():
    """Run tests for each WASM runtime crate."""
    with Flox("."):
        _run_runtimes()


@test()
@angreal.command(name="integration", about="Run integration tests (ignored tests)")
def test_integration():
    """Run tests marked with #[ignore]."""
    with Flox("."):
        subprocess.run(
            ["cargo", "test", "--workspace", "--", "--ignored", "--test-threads=1"],
            check=True,
        )


def _run_unit():
    subprocess.run(
        ["cargo", "test", "--workspace", "--", "--test-threads=1"],
        check=True,
    )


def _run_runtimes():
    runtimes_dir = os.path.join(os.getcwd(), "runtimes")
    for entry in sorted(os.listdir(runtimes_dir)):
        runtime_path = os.path.join(runtimes_dir, entry)
        if os.path.isdir(runtime_path) and os.path.exists(
            os.path.join(runtime_path, "Cargo.toml")
        ):
            print(f"\n--- Testing runtime: {entry} ---")
            subprocess.run(["cargo", "test"], cwd=runtime_path, check=True)
