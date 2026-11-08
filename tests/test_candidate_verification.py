from hashlib import sha256
from io import BytesIO
import importlib.util
import json
from pathlib import Path
import stat
import unittest
import warnings
from zipfile import ZipFile, ZipInfo

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("check_candidate", ROOT / "scripts/check_candidate.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def candidate(contents=None, mutate=None):
    files = {"svelte/package.json": b'{"version":"0.8.3"}',
             "reference/svelte/package.json": b'{"version":"0.8.3"}',
             "tokens/contract.json": b'{"version":"0.3.0"}',
             **(contents or {})}
    manifest = {"status": "candidate", "svelteVersion": "0.8.3", "tokenContract": "0.3.0",
                "sha256": {name: sha256(value).hexdigest() for name, value in files.items()}}
    if mutate:
        mutate(manifest)
    output = BytesIO()
    with ZipFile(output, "w") as archive:
        for name, value in files.items():
            archive.writestr(name, value)
        archive.writestr("manifest.json", json.dumps(manifest))
    output.seek(0)
    return output


class CandidateVerificationTests(unittest.TestCase):
    def test_complete_archive_and_versions_pass(self):
        with ZipFile(candidate()) as archive:
            self.assertEqual(module.verify(archive)["svelteVersion"], "0.8.3")

    def test_corruption_missing_entries_and_stale_versions_fail(self):
        mutations = [
            (lambda m: m["sha256"].update({"svelte/package.json": "0" * 64}), "checksum mismatch"),
            (lambda m: m["sha256"].pop("tokens/contract.json"), "archive membership"),
            (lambda m: m["sha256"].update({"missing.txt": "0" * 64}), "archive membership"),
            (lambda m: m.update({"svelteVersion": "0.8.2"}), "version mismatch"),
            (lambda m: m.update({"tokenContract": "0.2.0"}), "version mismatch"),
            (lambda m: m.update({"status": "accepted"}), "unaccepted candidate"),
            (lambda m: m.update({"status": "accepted-local"}), "unaccepted candidate"),
            (lambda m: m.update({"libraryVersion": "1.0.0"}), "library and Svelte version mismatch"),
        ]
        for mutation, message in mutations:
            with self.subTest(message=message), ZipFile(candidate(mutate=mutation)) as archive:
                with self.assertRaisesRegex(ValueError, message):
                    module.verify(archive)

    def test_local_release_requires_explicit_verification_mode(self):
        record = "docs/verification/m3-m4.md"
        acceptance = {"owner": "Xerrion", "date": "2026-11-08", "record": record}
        update = lambda m: m.update({"status": "accepted-local", "distribution": "local", "acceptance": acceptance})
        with ZipFile(candidate({record: b"Accepted with recorded limits"}, mutate=update)) as archive:
            with self.assertRaisesRegex(ValueError, "unaccepted candidate"):
                module.verify(archive)
            manifest = module.verify(archive, expected_status="accepted-local")
            self.assertEqual(manifest["acceptance"], acceptance)
        with ZipFile(candidate()) as archive:
            with self.assertRaisesRegex(ValueError, "accepted local release"):
                module.verify(archive, expected_status="accepted-local")
            with self.assertRaisesRegex(ValueError, "unknown expected archive status"):
                module.verify(archive, expected_status="accepted")

    def test_local_release_needs_local_distribution_and_packaged_acceptance(self):
        for fields, message in [
            ({"distribution": "registry"}, "local distribution"),
            ({"distribution": "local"}, "acceptance record"),
            ({"distribution": "local", "acceptance": {"owner": "Xerrion", "date": "2026-11-08", "record": "absent.md"}}, "acceptance record"),
        ]:
            with self.subTest(fields=fields), ZipFile(candidate(mutate=lambda m: m.update({"status": "accepted-local", **fields}))) as archive:
                with self.assertRaisesRegex(ValueError, message):
                    module.verify(archive, expected_status="accepted-local")

    def test_unsafe_paths_are_rejected_before_extraction(self):
        for name in ("../outside.txt", "/absolute.txt", "a/../../outside.txt", "a\\outside.txt", "./alias.txt"):
            with self.subTest(name=name), ZipFile(candidate({name: b"file"})) as archive:
                with self.assertRaisesRegex(ValueError, "unsafe file path"):
                    module.verify(archive)

    def test_invalid_version_document_reports_its_path(self):
        with ZipFile(candidate({"tokens/contract.json": b"[]"})) as archive:
            with self.assertRaisesRegex(ValueError, "JSON must be an object: tokens/contract.json"):
                module.verify(archive)

    def test_duplicates_and_symlinks_are_rejected(self):
        duplicate = candidate()
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            with ZipFile(duplicate, "a") as archive:
                archive.writestr("tokens/contract.json", b"duplicate")
        with ZipFile(duplicate) as archive:
            with self.assertRaisesRegex(ValueError, "duplicate archive paths"):
                module.verify(archive)
        linked = candidate()
        with ZipFile(linked, "a") as archive:
            info = ZipInfo("linked-file")
            info.create_system = 3
            info.external_attr = (stat.S_IFLNK | 0o777) << 16
            archive.writestr(info, "../outside")
        with ZipFile(linked) as archive:
            with self.assertRaisesRegex(ValueError, "unsafe file path"):
                module.verify(archive)


if __name__ == "__main__":
    unittest.main()
