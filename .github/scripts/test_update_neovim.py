from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import update_neovim as updater


class ReleaseTests(unittest.TestCase):
    def test_latest_stable_numeric_release_across_pages(self):
        def release(tag, **flags):
            return {"tag_name": tag, "draft": False, "prerelease": False, **flags}

        pages = [
            [
                release("nightly"),
                release("stable"),
                release("v0.13.0", prerelease=True),
                release("v1.0.0", draft=True),
                release("v0.12.11-rc1"),
                release("v0.12.12-r2"),
                release("v0.12.9"),
            ],
            [release("v0.12.10"), release("v0.11.99")],
        ]
        self.assertEqual(updater.latest_release(pages), "0.12.10")

    def test_missing_stable_release_fails(self):
        with self.assertRaisesRegex(ValueError, "no stable numeric"):
            updater.latest_release([[]])

    def test_invalid_versions_fail(self):
        for value in ["0.12", "v0.12.6", "0.12.6-r2", "0.12.6-rc1", "0.12.6+dev", "01.2.3"]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                updater.version(value)

    def test_sys_revisions_target_the_base_neovim_release(self):
        for value in ["0.12.6", "0.12.6-r0", "0.12.6-r2", "0.12.6-r12"]:
            with self.subTest(value=value):
                self.assertEqual(updater.sys_neovim_version(value), (0, 12, 6))

    def test_invalid_sys_versions_fail(self):
        for value in [
            "0.12", "01.2.3-r1", "0.12.6-r", "0.12.6-r01", "0.12.6-r-1",
            "0.12.6-rc1", "0.12.6-r2+dev", "v0.12.6", "v0.12.6-r2", "vv0.12.6-r2",
        ]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                updater.sys_neovim_version(value)


class UpdateTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        (self.root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["crates/*"]\n\n'
            '[workspace.dependencies]\n'
            'mp = { version = "0.1.9", path = "crates/rmp-bytes", package = "rmp-bytes" } # alias\n'
            'nvim-rpc = { version = "0.2.3", path = "crates/nvim-rpc" }\n'
            'nvim-rpc-sys = { version = "=0.12.6-r2", path = "crates/nvim-rpc-sys" }\n'
            'external = { version = "1" }\n'
        )
        for name, version in [
            ("rmp-bytes", "0.1.9"),
            ("nvim-rpc", "0.2.3"),
            ("nvim-rpc-plugin", "0.3.1"),
            ("nvim-rpc-sys", "0.12.6-r2"),
        ]:
            path = self.root / "crates" / name / "Cargo.toml"
            path.parent.mkdir(parents=True)
            path.write_text(
                f'[package]\nname = "{name}"\nversion = "{version}" # package\n'
                '\n[dependencies]\nexternal = { version = "0.9.0" }\n'
            )

    def test_current_and_older_releases_are_skipped(self):
        path = self.root / "crates/nvim-rpc-sys/Cargo.toml"
        text = path.read_text()
        for current in ["0.12.6", "0.12.6-r2", "0.12.6-r12"]:
            with self.subTest(current=current):
                path.write_text(text.replace('"0.12.6-r2"', f'"{current}"'))
                self.assertTrue(updater.needs_update(self.root, "0.12.7"))
                self.assertTrue(updater.needs_update(self.root, "0.13.0"))
                self.assertFalse(updater.needs_update(self.root, "0.12.6"))
                self.assertFalse(updater.needs_update(self.root, "0.11.99"))

    def test_revision_of_current_release_is_not_regenerated(self):
        before = {p: p.read_bytes() for p in self.root.rglob("Cargo.toml")}
        with patch.object(updater.subprocess, "run") as run:
            updater.update(self.root, "0.12.6")
        run.assert_not_called()
        self.assertEqual(before, {p: p.read_bytes() for p in before})

    def test_bumps_packages_and_aliased_workspace_requirements(self):
        changes = updater.bumped_manifests(self.root)
        for name, expected in [
            ("rmp-bytes", "0.1.10"),
            ("nvim-rpc", "0.2.4"),
            ("nvim-rpc-plugin", "0.3.2"),
        ]:
            text = changes[self.root / "crates" / name / "Cargo.toml"]
            data = updater.tomllib.loads(text)
            self.assertEqual(data["package"]["version"], expected)
            self.assertEqual(data["dependencies"]["external"]["version"], "0.9.0")
            self.assertIn("# package", text)

        self.assertNotIn(self.root / "crates/nvim-rpc-sys/Cargo.toml", changes)
        text = changes[self.root / "Cargo.toml"]
        deps = updater.tomllib.loads(text)["workspace"]["dependencies"]
        self.assertEqual(deps["mp"]["version"], "0.1.10")
        self.assertEqual(deps["nvim-rpc"]["version"], "0.2.4")
        self.assertEqual(deps["nvim-rpc-sys"]["version"], "=0.12.6-r2")
        self.assertEqual(deps["external"]["version"], "1")
        self.assertIn("# alias", text)

    def test_update_is_repeatable_without_another_bump(self):
        def cargo(args, **kwargs):
            self.assertEqual(kwargs["cwd"], self.root)
            if args[:2] == ["cargo", "gen"]:
                for path in [
                    self.root / "Cargo.toml",
                    self.root / "crates/nvim-rpc-sys/Cargo.toml",
                ]:
                    text = path.read_text()
                    text = text.replace('"0.12.6-r2"', '"0.12.7"')
                    text = text.replace('"=0.12.6-r2"', '"0.12.7"')
                    path.write_text(text)

        with patch.object(updater.subprocess, "run", side_effect=cargo) as run:
            updater.update(self.root, "0.12.7")
            self.assertEqual(run.call_count, 2)
            self.assertEqual(run.call_args.args[0], ["cargo", "metadata", "--format-version", "1"])
            first = {p: p.read_bytes() for p in self.root.rglob("Cargo.toml")}
            updater.update(self.root, "0.12.7")
            self.assertEqual(run.call_count, 2)
            self.assertEqual(first, {p: p.read_bytes() for p in first})

    def test_generation_must_match_the_requested_release(self):
        with patch.object(updater.subprocess, "run"), self.assertRaisesRegex(
            ValueError, "does not match"
        ):
            updater.update(self.root, "0.12.7")
        package = updater.read_manifest(self.root / "crates/nvim-rpc/Cargo.toml")["package"]
        self.assertEqual(package["version"], "0.2.3")

    def test_unsupported_manifest_layout_fails_before_bumps_are_written(self):
        path = self.root / "Cargo.toml"
        text = path.read_text()
        line = next(line for line in text.splitlines(keepends=True) if line.startswith("mp ="))
        path.write_text(
            text.replace(line, "")
            + '\n[workspace.dependencies.mp]\nversion = "0.1.9"\n'
            'path = "crates/rmp-bytes"\npackage = "rmp-bytes"\n'
        )
        before = {p: p.read_bytes() for p in self.root.rglob("Cargo.toml")}
        with self.assertRaisesRegex(ValueError, "expected one version assignment"):
            updater.bumped_manifests(self.root)
        self.assertEqual(before, {p: p.read_bytes() for p in before})


if __name__ == "__main__":
    unittest.main()
