"""Prepare a Neovim release update from the checked-out default branch."""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib


def version(s):
    if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", s):
        raise ValueError(f"expected a stable major.minor.patch version, got {s!r}")
    return tuple(map(int, s.split(".")))


def latest_release(pages):
    versions = []
    for page in pages:
        for release in page:
            if release["draft"] or release["prerelease"]:
                continue
            tag = release["tag_name"]
            if not tag.startswith("v"):
                continue
            try:
                versions.append(version(tag[1:]))
            except ValueError:
                continue
    if not versions:
        raise ValueError("no stable numeric Neovim releases found")
    return ".".join(map(str, max(versions)))


def read_manifest(path):
    return tomllib.loads(path.read_text())


def needs_update(root, target):
    current = read_manifest(root / "crates/nvim-api-sys/Cargo.toml")["package"]["version"]
    return version(target) > version(current)


def replace_version(text, section, key, new):
    # Edit only the existing assignment, retaining comments and manifest layout.
    pattern = rf"(?ms)(^\[{re.escape(section)}\][^\n]*\n)(.*?)(?=^\[|\Z)"

    def replace_section(m):
        prefix = rf"^[ \t]*{re.escape(key)}[ \t]*=[ \t]*"
        if section == "workspace.dependencies":
            prefix += r"\{[^\n]*?\bversion[ \t]*=[ \t]*"
        body, n = re.subn(
            rf"(?m)({prefix})([\"'])([^\"']*)(\2)",
            lambda v: f"{v[1]}{v[2]}{new}{v[4]}",
            m[2],
        )
        if n != 1:
            raise ValueError(f"expected one version assignment for [{section}] {key}")
        return m[1] + body

    text, n = re.subn(pattern, replace_section, text)
    if n != 1:
        raise ValueError(f"expected one [{section}] section")
    tomllib.loads(text)
    return text


def bumped_manifests(root):
    changes = {}
    versions = {}
    for path in sorted((root / "crates").glob("*/Cargo.toml")):
        package = read_manifest(path)["package"]
        if package["name"] == "nvim-api-sys":
            continue
        major, minor, patch = version(package["version"])
        new = f"{major}.{minor}.{patch + 1}"
        versions[package["name"]] = new
        changes[path] = replace_version(path.read_text(), "package", "version", new)

    path = root / "Cargo.toml"
    text = path.read_text()
    for key, dep in read_manifest(path)["workspace"]["dependencies"].items():
        if not isinstance(dep, dict) or "path" not in dep:
            continue
        name = dep.get("package", key)
        if name in versions:
            text = replace_version(text, "workspace.dependencies", key, versions[name])
    changes[path] = text
    return changes


def update(root, target):
    if not needs_update(root, target):
        print(f"Neovim {target} is already covered; no update needed")
        return

    subprocess.run(["cargo", "gen", target], cwd=root, check=True)
    sys = read_manifest(root / "crates/nvim-api-sys/Cargo.toml")
    workspace = read_manifest(root / "Cargo.toml")
    if (
        sys["package"]["version"] != target
        or workspace["workspace"]["dependencies"]["nvim-api-sys"]["version"] != target
    ):
        raise ValueError("generated Neovim version does not match the requested release")

    for path, text in bumped_manifests(root).items():
        path.write_text(text)
    subprocess.run(
        ["cargo", "metadata", "--format-version", "1"],
        cwd=root,
        check=True,
        stdout=subprocess.DEVNULL,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("check")
    check.add_argument("releases", type=Path)
    prepare = commands.add_parser("update")
    prepare.add_argument("version")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]

    if args.command == "check":
        target = latest_release(json.loads(args.releases.read_text()))
        needed = needs_update(root, target)
        print(f"Newest stable Neovim: {target}; update needed: {needed}")
        with open(os.environ["GITHUB_OUTPUT"], "a") as output:
            output.write(f"version={target}\nupdate={str(needed).lower()}\n")
    else:
        update(root, args.version)


if __name__ == "__main__":
    main()
