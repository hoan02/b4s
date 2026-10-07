import importlib.util
import json
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest

SCRIPT = Path(__file__).with_name("extract-model-profile-drafts.py")
SPEC = importlib.util.spec_from_file_location("draft_extraction", SCRIPT)
extract = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(extract)


def model(name, key="server-test", paths=("Earbuds",)):
    return {"id": key, "model": name, "audio": True,
        "variants": [{"region": "us", "categoryPath": [path]} for path in paths]}


class DraftExtractionTests(unittest.TestCase):
    def test_cli_generates_nonimportable_draft_with_source_and_gesture_evidence(self):
        temporary_root = extract.ROOT / ".tmp"
        temporary_root.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=temporary_root) as directory:
            work = Path(directory)
            source = work / "apk/jadx-out/sources/other"
            source.mkdir(parents=True)
            (source / "Model.java").write_text('return "Test Model";', encoding="utf-8")
            assets = work / "apk/extracted/assets/gesture"
            assets.mkdir(parents=True)
            (assets / "functions.json").write_text(json.dumps({"modelList": ["Test Model"], "functionList": [1]}), encoding="utf-8")
            catalog = work / "catalog.json"
            catalog.write_text(json.dumps({"schemaVersion": 1, "models": [model("Test Model")]}), encoding="utf-8")
            result = subprocess.run([sys.executable, str(SCRIPT), "--apk-dir", str(work / "apk"),
                "--catalog", str(catalog), "--output", str(work / "output")], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            draft = json.loads((work / "output/profiles/server-test.json").read_text(encoding="utf-8"))
            self.assertEqual(draft["evidenceStatus"], "exact-reference")
            self.assertEqual(draft["gestureConfigurationEvidence"][0]["functionIds"], [1])
            self.assertFalse(draft["importable"])
            self.assertEqual(draft["runtimeProfileDraft"]["capabilities"], {})
            self.assertEqual(draft["runtimeProfileDraft"]["schemaVersion"], 3)
            self.assertEqual(draft["runtimeProfileDraft"]["featureEvidence"]["gestureV2"]["status"], "unknown")
            self.assertTrue((work / "output/evidence-review.md").is_file())

    def test_gesture_assets_keep_model_branch_and_child_actions_separate(self):
        with tempfile.TemporaryDirectory() as directory:
            assets = Path(directory) / "extracted/assets/gesture"
            assets.mkdir(parents=True)
            (assets / "functions.json").write_text(json.dumps({"singleClick": [
                {"modelList": ["Baseus BP1 Ultra"], "functionList": [1, 0, 18],
                    "childList": [{"functionId": 18, "functionValue": [0, 1]}]},
                {"modelList": ["Baseus BP1 Pro"], "functionList": [7]}]}), encoding="utf-8")
            refs = extract.index_gesture_configuration(Path(directory), [model("BP1 Ultra", "ultra"), model("BP1 Pro", "pro")])
            self.assertEqual(refs["ultra"][0]["functionIds"], [1, 0, 18])
            self.assertEqual(refs["ultra"][0]["childActionIds"], [18])
            self.assertEqual(refs["ultra"][0]["jsonPointer"], "/singleClick/0")
            self.assertEqual(refs["pro"][0]["functionIds"], [7])
            self.assertNotIn("capabilities", refs["ultra"][0])

    def test_legacy_gesture_layout_is_not_mistaken_for_v2_buttons(self):
        with tempfile.TemporaryDirectory() as directory:
            assets = Path(directory) / "extracted/assets/gesture"
            assets.mkdir(parents=True)
            (assets / "layout.json").write_text(json.dumps({"gestureData": [{"model": ["Legacy"],
                "layoutType": [0, 2, 1], "defaultFuntion": {"doubleClickLeft": 1}}]}), encoding="utf-8")
            refs = extract.index_gesture_configuration(Path(directory), [model("Legacy")])["server-test"]
            self.assertEqual(refs[0]["layoutTypes"], [0, 2, 1])
            self.assertEqual(refs[0]["defaultFunctions"], {"doubleClickLeft": 1})
            self.assertNotIn("buttons", refs[0])

    def test_all_packages_long_mapping_lines_and_explicit_aliases(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "jadx-out/sources/vendor/other"
            source.mkdir(parents=True)
            (source / "DeviceManager.java").write_text(' ' * 4500 +
                'TuplesKt.to("BP1 Ultra", "Baseus Bass BP1 Ultra");', encoding="utf-8")
            (source / "UnknownName.java").write_text('return "BP1 Ultra";', encoding="utf-8")
            refs = extract.index_source(Path(directory), [model("Baseus Bass BP1 Ultra")])["server-test"]
            self.assertTrue(any(ref["matchKind"] == "explicit-alias" and ref["areas"] == ["other-source"] for ref in refs))
            self.assertEqual(extract.evidence_status(refs), "exact-reference")

    def test_comments_do_not_establish_executable_model_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "jadx-out/sources/other"
            source.mkdir(parents=True)
            (source / "Unknown.java").write_text('/* JADX WARN\n if (x.equals("Only Comment"))\n */\n'
                '// "Only Comment"\nreturn "Real Model";', encoding="utf-8")
            refs = extract.index_source(Path(directory), [model("Only Comment", "comment"), model("Real Model", "real")])
            self.assertEqual(extract.evidence_status(refs["comment"]), "comment-only")
            self.assertEqual(extract.evidence_status(refs["real"]), "exact-reference")

    def test_commented_and_ambiguous_alias_mappings_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "jadx-out/sources/other"
            source.mkdir(parents=True)
            (source / "DeviceManager.java").write_text(
                '/* TuplesKt.to("Fake", "Model A"); */\n'
                'TuplesKt.to("Shared", "Model A"); TuplesKt.to("Shared", "Model B");', encoding="utf-8")
            (source / "Consumer.java").write_text('return "Fake" + "Shared";', encoding="utf-8")
            refs = extract.index_source(Path(directory), [model("Model A", "a"), model("Model B", "b")])
            self.assertFalse(any(ref["matchKind"] == "explicit-alias" for values in refs.values() for ref in values))

    def test_text_resources_are_indexed_without_binary_files(self):
        with tempfile.TemporaryDirectory() as directory:
            apk = Path(directory)
            (apk / "jadx-out/sources").mkdir(parents=True)
            assets = apk / "extracted/assets"
            assets.mkdir(parents=True)
            (assets / "models.json").write_text('{"model":"Resource Model"}', encoding="utf-8")
            duplicate = apk / "jadx-out/resources/assets"
            duplicate.mkdir(parents=True)
            (duplicate / "models.json").write_text('{"model":"Resource Model"}', encoding="utf-8")
            (assets / "image.png").write_bytes(b'"Binary Model"')
            refs = extract.index_source(apk, [model("Resource Model", "resource"), model("Binary Model", "binary")])
            self.assertEqual(extract.evidence_status(refs["resource"]), "exact-reference")
            self.assertEqual(len(refs["resource"]), 1)
            self.assertEqual(refs["binary"], [])

    def test_legacy_cache_identity_is_checked_using_exact_url_model(self):
        value = {"url": "https://example.com/params?model=Baseus+BP1+Ultra", "region": "us"}
        extract.validate_cached_identity(value, "Baseus BP1 Ultra", "us")
        self.assertEqual(value["model"], "Baseus BP1 Ultra")
        with self.assertRaises(ValueError):
            extract.validate_cached_identity(value, "Baseus BP1 Pro", "us")
        with self.assertRaises(ValueError):
            extract.validate_cached_identity(value, "Baseus BP1 Ultra", "eu")

    def test_speakers_require_consistent_regional_classification(self):
        candidates = extract.headphone_candidates({"schemaVersion": 1, "models": [
            model("Speaker", "speaker", ("Speakers",)),
            model("Mixed", "mixed", ("Speakers", "Earbuds")),
            model("Unknown", "unknown", ("Audio",)),
        ]})
        self.assertEqual([m["id"] for m in candidates], ["mixed", "unknown"])

    def test_duplicate_and_path_traversal_ids_are_rejected(self):
        for entries in [[model("A"), model("B")], [model("A", "../bad")]]:
            with self.assertRaises(ValueError):
                extract.headphone_candidates({"schemaVersion": 1, "models": entries})

    def test_full_model_identity_does_not_match_a_shorter_product(self):
        with tempfile.TemporaryDirectory() as directory:
            apk = Path(directory)
            source = apk / "jadx-out/sources/com/control_center"
            source.mkdir(parents=True)
            path = source / "NoiseReduceDataModel.java"
            path.write_text('if (model.equals("Baseus BP1 Ultra")) {}\n'
                '@Metadata(value="Baseus BP1 Pro")\n'
                'if (model.equals("BP1")) {}\n', encoding="utf-8")
            result = extract.index_source(apk, [model("Baseus BP1 Ultra", "ultra"), model("Baseus BP1 Pro", "pro")])
            self.assertEqual(len(result["ultra"]), 1)
            self.assertEqual(result["ultra"][0]["line"], 1)
            self.assertEqual(result["ultra"][0]["areas"], ["listening"])
            self.assertEqual(len(result["ultra"][0]["sha256"]), 64)
            self.assertEqual(result["pro"], [])
            self.assertNotIn("source", result["ultra"][0])

    def test_ambiguous_names_never_select_a_profile(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "jadx-out/sources/com/base"
            source.mkdir(parents=True)
            (source / "DeviceVersionUtil.java").write_text('return "BP1";', encoding="utf-8")
            result = extract.index_source(Path(directory), [model("BP1", "first"), model("Baseus BP1", "second")])
            self.assertEqual(result, {"first": [], "second": []})

    def test_error_responses_and_nonobjects_are_not_configuration(self):
        for response in [[], {"code": 1, "data": {}}, {"code": 0, "data": []}]:
            with self.assertRaises(ValueError):
                extract.summarize_params(response)

    def test_empty_eq_is_metadata_not_a_feature_permission(self):
        result = extract.summarize_params({"code": 0, "data": {"eq_sound_mode": [], "icons": [1]}})
        self.assertEqual(result["listCounts"], {"eq_sound_mode": 0, "icons": 1})
        self.assertNotIn("capabilities", result)
        self.assertEqual(result["eqPresetSorts"], [])

    def test_eq_summary_does_not_copy_command_payloads(self):
        result = extract.summarize_params({"code": 0, "data": {"eq_sound_mode": [{"dictSort": 7, "description": "payload"}]}})
        self.assertEqual(result["eqPresetSorts"], [7])
        self.assertNotIn("payload", json.dumps(result))


if __name__ == "__main__":
    unittest.main()
