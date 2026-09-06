"""Offline regression tests; no Cargo invocation or GitHub environment required."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from check_release_tag import check_release_tag


def package(name, version="0.10.0", publish=None):
    return {"id": f"opaque-id:{name}@{version}", "name": name, "version": version, "publish": publish}


def metadata(*packages, external=()):
    return {
        "workspace_members": [item["id"] for item in packages],
        "packages": [*packages, *external],
    }


class ReleaseTagTests(unittest.TestCase):
    def test_all_cargo_publishing_representations_and_workspace_membership(self):
        data = metadata(
            package("unrestricted"),
            package("restricted", publish=["crates-io"]),
            package("multiple-registries", publish=["crates-io", "private-registry"]),
            package("paft-decimal-consumer", "99.0.0", publish=[]),
            external=(package("external-dependency", "88.0.0"),),
        )
        self.assertEqual(
            check_release_tag(data, "v0.10.0"),
            [("multiple-registries", "0.10.0"), ("restricted", "0.10.0"), ("unrestricted", "0.10.0")],
        )

    def test_empty_release_sets_fail_with_diagnostics(self):
        for data in (
            metadata(),
            metadata(package("private-fixture", publish=[])),
            metadata(external=(package("external"),)),
        ):
            with self.subTest(data=data), self.assertRaises(ValueError) as error:
                check_release_tag(data, "v0.10.0")
            self.assertIn("release set is empty", str(error.exception))
            self.assertIn("Publishable workspace packages:\n  (none)", str(error.exception))

    def test_versions_must_agree_and_failure_names_every_selected_package(self):
        data = metadata(package("alpha"), package("beta", "0.11.0"))
        with self.assertRaises(ValueError) as error:
            check_release_tag(data, "v0.10.0")
        self.assertIn("versions differ", str(error.exception))
        self.assertIn("alpha 0.10.0", str(error.exception))
        self.assertIn("beta 0.11.0", str(error.exception))

    def test_tag_equality_is_literal(self):
        data = metadata(package("paft"))
        for tag in ("v0.10.1", "v0.10.0-rc.1", "0.10.0", "V0.10.0", "v0.10.0 ", "v0.10.0\n"):
            with self.subTest(tag=tag), self.assertRaises(ValueError) as error:
                check_release_tag(data, tag)
            self.assertIn("does not equal 'v0.10.0'", str(error.exception))
            self.assertIn("paft 0.10.0", str(error.exception))

    def test_prereleases_match_without_version_normalization(self):
        data = metadata(package("paft", "0.10.0-rc.1"))
        self.assertEqual(check_release_tag(data, "v0.10.0-rc.1"), [("paft", "0.10.0-rc.1")])
        for tag in ("v0.10.0", "v0.10.0-rc.2"):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                check_release_tag(data, tag)

    def run_cli(self, raw, tag):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cargo metadata.json"
            path.write_text(raw, encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(Path(__file__).with_name("check_release_tag.py")),
                 "--metadata", str(path), "--tag", tag],
                # An unrelated GitHub environment cannot override explicit inputs.
                env={**os.environ, "GITHUB_REF_NAME": "v999.0.0"},
                capture_output=True, text=True, check=False,
            )

    def test_cli_accepts_explicit_inputs_and_reports_the_discovered_set(self):
        result = self.run_cli(json.dumps(metadata(package("paft"))), "v0.10.0")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, "")
        self.assertIn("paft 0.10.0", result.stdout)

    def test_cli_rejects_mismatch_with_actionable_diagnostics(self):
        result = self.run_cli(json.dumps(metadata(package("paft"))), "v0.10.1")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stdout, "")
        self.assertIn("tag 'v0.10.1' does not equal 'v0.10.0'", result.stderr)
        self.assertIn("Publishable workspace packages:\n  paft 0.10.0", result.stderr)

    def test_cli_rejects_unreadable_metadata_without_tracebacks(self):
        for raw in ("not JSON", "{}", "[]"):
            with self.subTest(raw=raw):
                result = self.run_cli(raw, "v0.10.0")
                self.assertEqual(result.returncode, 1)
                self.assertIn("Release preflight failed:", result.stderr)
                self.assertNotIn("Traceback", result.stderr)


if __name__ == "__main__":
    unittest.main()
