"""Publish the merged crate versions, resuming after a partial release."""

import json
from pathlib import Path
import subprocess
import time
import tomllib
from urllib.error import HTTPError
from urllib.request import Request, urlopen


# The two independent libraries must be available before their dependents.
CRATES = ("rmp-bytes", "nvim-rpc-sys", "nvim-rpc", "nvim-rpc-plugin")
USER_AGENT = "nvim-rpc-rs publishing (https://github.com/naughie/nvim-rpc-rs)"


def fetch(url):
    request = Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urlopen(request, timeout=30) as response:
            return response.read()
    except HTTPError as e:
        if e.code == 404:
            return None
        raise


def is_published(name, version):
    data = fetch(f"https://crates.io/api/v1/crates/{name}/{version}")
    if data is None:
        return False
    published = json.loads(data)["version"]
    if published["crate"] != name or published["num"] != version:
        raise ValueError(f"unexpected registry version for {name} {version}")
    if published["yanked"]:
        raise ValueError(f"{name} {version} is yanked; choose a new version")
    return True


def wait_for_index(name, version):
    # Cargo waits after a successful upload. Also wait when resuming an upload
    # that succeeded on crates.io but timed out before Cargo saw the index entry.
    url = f"https://index.crates.io/{name[:2]}/{name[2:4]}/{name}"
    deadline = time.monotonic() + 180
    while True:
        data = fetch(url)
        if data is not None:
            for line in data.splitlines():
                entry = json.loads(line)
                if entry["vers"] == version:
                    if entry["yanked"]:
                        raise ValueError(f"{name} {version} is yanked in the registry index")
                    return
        if time.monotonic() >= deadline:
            raise TimeoutError(f"{name} {version} did not appear in the registry index")
        time.sleep(5)


def publish(root):
    for name in CRATES:
        manifest = root / "crates" / name / "Cargo.toml"
        package = tomllib.loads(manifest.read_text())["package"]
        if package["name"] != name:
            raise ValueError(f"unexpected package name in {manifest}")
        version = package["version"]
        if is_published(name, version):
            print(f"{name} {version} is already published; skipping", flush=True)
            wait_for_index(name, version)
            continue
        print(f"Publishing {name} {version}", flush=True)
        subprocess.run(
            ["cargo", "publish", "--package", name, "--locked", "--registry", "crates-io"],
            cwd=root,
            check=True,
        )


if __name__ == "__main__":
    publish(Path(__file__).resolve().parents[2])
