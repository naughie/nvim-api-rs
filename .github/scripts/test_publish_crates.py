from io import BytesIO
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import call, patch
from urllib.error import HTTPError

import publish_crates as publisher


class RegistryTests(unittest.TestCase):
    def test_published_version_is_recognized(self):
        response = {"version": {"crate": "nvim-api", "num": "0.1.1", "yanked": False}}
        with patch.object(publisher, "urlopen", return_value=BytesIO(json.dumps(response).encode())) as get:
            self.assertTrue(publisher.is_published("nvim-api", "0.1.1"))
        request = get.call_args.args[0]
        self.assertEqual(request.full_url, "https://crates.io/api/v1/crates/nvim-api/0.1.1")
        self.assertIn("User-agent", request.headers)

    def test_only_not_found_is_treated_as_unpublished(self):
        for status in (404, 403, 429, 500):
            error = HTTPError("https://crates.io", status, "registry error", {}, None)
            with self.subTest(status=status), patch.object(publisher, "urlopen", side_effect=error):
                if status == 404:
                    self.assertFalse(publisher.is_published("nvim-api", "0.1.1"))
                else:
                    with self.assertRaises(HTTPError):
                        publisher.is_published("nvim-api", "0.1.1")

    def test_yanked_or_unexpected_version_is_rejected(self):
        for entry in (
            {"crate": "nvim-api", "num": "0.1.1", "yanked": True},
            {"crate": "nvim-api", "num": "0.1.0", "yanked": False},
            {"crate": "other", "num": "0.1.1", "yanked": False},
        ):
            with self.subTest(entry=entry), patch.object(
                publisher, "fetch", return_value=json.dumps({"version": entry}).encode()
            ), self.assertRaises(ValueError):
                publisher.is_published("nvim-api", "0.1.1")

    def test_resumed_upload_waits_for_the_dependency_index(self):
        old = b'{"vers":"0.1.0","yanked":false}\n'
        new = old + b'{"vers":"0.1.1","yanked":false}\n'
        with patch.object(publisher, "fetch", side_effect=[None, old, new]) as get, patch.object(
            publisher.time, "sleep"
        ) as sleep:
            publisher.wait_for_index("nvim-api", "0.1.1")
        self.assertEqual(get.call_args.args[0], "https://index.crates.io/nv/im/nvim-api")
        self.assertEqual(sleep.call_count, 2)

    def test_missing_index_entry_times_out(self):
        with patch.object(publisher, "fetch", return_value=None), patch.object(
            publisher.time, "monotonic", side_effect=[0, 181]
        ), self.assertRaises(TimeoutError):
            publisher.wait_for_index("nvim-api", "0.1.1")

    def test_yanked_index_entry_fails(self):
        with patch.object(
            publisher, "fetch", return_value=b'{"vers":"0.1.1","yanked":true}\n'
        ), self.assertRaisesRegex(ValueError, "yanked"):
            publisher.wait_for_index("nvim-api", "0.1.1")


class PublishTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        for name in publisher.CRATES:
            path = self.root / "crates" / name / "Cargo.toml"
            path.parent.mkdir(parents=True)
            path.write_text(f'[package]\nname = "{name}"\nversion = "0.1.1"\n')

    def test_dependencies_are_published_before_dependents(self):
        with patch.object(publisher, "is_published", return_value=False), patch.object(
            publisher.subprocess, "run"
        ) as run:
            publisher.publish(self.root)
        names = [c.args[0][3] for c in run.call_args_list]
        self.assertEqual(set(names), {"rmp-bytes", "nvim-api-sys", "nvim-api", "nvim-api-plugin"})
        self.assertEqual(len(names), 4)
        self.assertLess(names.index("rmp-bytes"), names.index("nvim-api"))
        self.assertLess(names.index("nvim-api-sys"), names.index("nvim-api"))
        self.assertLess(names.index("nvim-api"), names.index("nvim-api-plugin"))
        for c in run.call_args_list:
            self.assertEqual(c.args[0][-3:], ["--locked", "--registry", "crates-io"])
            self.assertEqual(c.kwargs, {"cwd": self.root, "check": True})

    def test_partial_release_resumes_without_republishing(self):
        with patch.object(publisher, "is_published", side_effect=[True, True, False, False]), patch.object(
            publisher, "wait_for_index"
        ) as wait, patch.object(publisher.subprocess, "run") as run:
            publisher.publish(self.root)
        self.assertEqual(wait.call_args_list, [call("rmp-bytes", "0.1.1"), call("nvim-api-sys", "0.1.1")])
        self.assertEqual([c.args[0][3] for c in run.call_args_list], ["nvim-api", "nvim-api-plugin"])

    def test_failed_upload_stops_before_publishing_dependents(self):
        with patch.object(publisher, "is_published", return_value=False) as check, patch.object(
            publisher.subprocess, "run", side_effect=subprocess.CalledProcessError(101, "cargo publish")
        ) as run, self.assertRaises(subprocess.CalledProcessError):
            publisher.publish(self.root)
        self.assertEqual(run.call_count, 1)
        self.assertEqual(check.call_count, 1)


if __name__ == "__main__":
    unittest.main()
