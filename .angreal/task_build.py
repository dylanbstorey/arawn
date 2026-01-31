"""Build commands for Arawn."""

import os
import subprocess

import angreal
from angreal.integrations.flox import Flox

build = angreal.command_group(name="build", about="Build the project")


@build()
@angreal.command(name="workspace", about="Build the workspace")
@angreal.argument(name="--release", long="release", takes_value=False, help="Build in release mode")
def build_workspace(release=False):
    """Build all workspace crates."""
    with Flox("."):
        cmd = ["cargo", "build", "--workspace"]
        if release:
            cmd.append("--release")
        subprocess.run(cmd, check=True)


@build()
@angreal.command(name="runtimes", about="Build WASM runtimes")
@angreal.argument(name="--release", long="release", takes_value=False, help="Build in release mode")
def build_runtimes(release=False):
    """Build each runtime targeting wasm32-wasip1."""
    with Flox("."):
        runtimes_dir = os.path.join(os.getcwd(), "runtimes")
        for entry in sorted(os.listdir(runtimes_dir)):
            runtime_path = os.path.join(runtimes_dir, entry)
            if os.path.isdir(runtime_path) and os.path.exists(
                os.path.join(runtime_path, "Cargo.toml")
            ):
                print(f"\n--- Building runtime: {entry} ---")
                cmd = ["cargo", "build", "--target", "wasm32-wasip1"]
                if release:
                    cmd.append("--release")
                subprocess.run(cmd, cwd=runtime_path, check=True)
