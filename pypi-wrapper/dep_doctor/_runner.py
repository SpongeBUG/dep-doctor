"""
Downloads and invokes the dep-doctor native binary.
Same pattern as the npm wrapper — fetches from GitHub Releases on first run.
"""

import os
import platform
import stat
import subprocess
import sys
import urllib.request
from pathlib import Path

VERSION = "0.1.0"
REPO = "YOUR_USERNAME/dep-doctor"
BIN_DIR = Path(__file__).parent / "bin"


def _asset_name() -> str:
    system = platform.system().lower()
    machine = platform.machine().lower()

    matrix = {
        ("linux",  "x86_64"):  "dep-doctor-x86_64-unknown-linux-musl",
        ("linux",  "aarch64"): "dep-doctor-aarch64-unknown-linux-musl",
        ("darwin", "x86_64"):  "dep-doctor-x86_64-apple-darwin",
        ("darwin", "arm64"):   "dep-doctor-aarch64-apple-darwin",
        ("windows","amd64"):   "dep-doctor-x86_64-pc-windows-msvc.exe",
    }

    key = (system, machine)
    asset = matrix.get(key)
    if not asset:
        raise RuntimeError(
            f"Unsupported platform: {system}/{machine}. "
            "Build from source: cargo install dep-doctor"
        )
    return asset


def _bin_path() -> Path:
    suffix = ".exe" if platform.system().lower() == "windows" else ""
    return BIN_DIR / f"dep-doctor{suffix}"


def _ensure_binary() -> Path:
    path = _bin_path()
    if path.exists():
        return path

    asset = _asset_name()
    url = f"https://github.com/{REPO}/releases/download/v{VERSION}/{asset}"

    print(f"dep-doctor: downloading binary ({asset})...", file=sys.stderr)
    BIN_DIR.mkdir(parents=True, exist_ok=True)

    urllib.request.urlretrieve(url, path)

    if platform.system().lower() != "windows":
        path.chmod(path.stat().st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)

    print("dep-doctor: installed.", file=sys.stderr)
    return path


def main() -> None:
    try:
        binary = _ensure_binary()
    except Exception as e:
        print(f"dep-doctor: failed to install binary: {e}", file=sys.stderr)
        print("Install manually: cargo install dep-doctor", file=sys.stderr)
        sys.exit(1)

    result = subprocess.run([str(binary)] + sys.argv[1:])
    sys.exit(result.returncode)


if __name__ == "__main__":
    main()
