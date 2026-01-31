"""Documentation commands for Arawn."""

import subprocess
import os

import angreal
from angreal.integrations.flox import Flox

docs = angreal.command_group(name="docs", about="Build and serve documentation")


@docs()
@angreal.command(name="build", about="Build the documentation site")
def docs_build():
    """Build the mdbook documentation."""
    docs_dir = os.path.join(os.getcwd(), "docs")
    if not os.path.exists(os.path.join(docs_dir, "book.toml")):
        print("Error: docs/book.toml not found")
        return

    with Flox("."):
        subprocess.run(["mdbook", "build"], cwd=docs_dir, check=True)
        print(f"\nDocumentation built to {os.path.join(docs_dir, 'book')}")


@docs()
@angreal.command(name="serve", about="Serve the documentation locally")
@angreal.argument(name="--port", short="p", takes_value=True, help="Port to serve on (default: 3000)")
def docs_serve(port="3000"):
    """Build and serve the documentation with live reload."""
    docs_dir = os.path.join(os.getcwd(), "docs")
    if not os.path.exists(os.path.join(docs_dir, "book.toml")):
        print("Error: docs/book.toml not found")
        return

    with Flox("."):
        cmd = ["mdbook", "serve", "--port", port, "--open"]
        subprocess.run(cmd, cwd=docs_dir, check=True)
