"""Offline tests for public catalog parsing (no account or network needed)."""
import importlib.util
import unittest
import tempfile
import json
from unittest.mock import patch
from pathlib import Path

spec = importlib.util.spec_from_file_location("sync_catalog", Path(__file__).with_name("sync-baseus-catalog.py"))
sync = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sync)


def response(model="Baseus Test Buds", region_name="Earphone", type_=2):
    return {"code": 0, "data": [{"name": region_name, "type": type_, "child": [
        {"name": "In ear series", "products": [{
            "model": model, "prodName": "Test Buds", "categoryId": 15,
            "icon": "https://cdn.example.test/buds.png", "colorList": None,
            "auth": "must-not-be-copied", "sn": "must-not-be-copied",
        }], "child": None},
    ]}]}


class CatalogTests(unittest.TestCase):
    def test_nullable_children_and_allowlisted_metadata(self):
        product = sync.flatten(response(), "us")[0]
        self.assertTrue(product["audio"])
        self.assertEqual(product["variant"]["colors"], [])
        self.assertNotIn("auth", str(product))
        self.assertNotIn("sn", product["variant"])

    def test_regions_merge_by_model_not_category_id_or_product_label(self):
        cn = response()
        cn["data"][0]["child"][0]["products"][0]["categoryId"] = 29
        models, sources = sync.merge({"us": response(), "cn": cn})
        self.assertEqual(len(models), 1)
        self.assertEqual(len(models[0]["variants"]), 2)
        self.assertEqual(len(sources), 2)

    def test_type_not_localized_category_name_selects_audio(self):
        self.assertTrue(sync.flatten(response(region_name="Tai nghe"), "cn")[0]["audio"])
        self.assertFalse(sync.flatten(response(region_name="Office", type_=6), "us")[0]["audio"])

    def test_bad_response_cannot_replace_a_snapshot(self):
        for invalid in [{"code": 1, "data": []}, {"code": 0, "data": {}}, {"code": 0, "data": []}]:
            with self.assertRaises(ValueError):
                sync.flatten(invalid, "us")
        with self.assertRaises(ValueError):
            sync.flatten(response(model=""), "us")

    def test_slug_collisions_preserve_distinct_models(self):
        models, _ = sync.merge({"us": response(model="Baseus A+"), "cn": response(model="Baseus A Plus")})
        self.assertEqual(len({m["id"] for m in models}), 2)

    def test_only_https_urls_without_credentials_are_kept(self):
        for value in [None, "file:///tmp/test", "http://example.test", "https://user:password@example.test"]:
            self.assertIsNone(sync.public_url(value))

    def test_failed_region_preserves_existing_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "snapshot.json"
            output.write_text("previous snapshot", encoding="utf-8")
            (root / "catalog-us-raw.json").write_text(json.dumps(response()), encoding="utf-8")
            (root / "catalog-eu-raw.json").write_text(json.dumps({"code": 1}), encoding="utf-8")
            with patch("sys.argv", ["sync", "--regions", "us", "eu", "--input-dir", str(root), "--output", str(output)]):
                self.assertEqual(sync.main(), 1)
            self.assertEqual(output.read_text(encoding="utf-8"), "previous snapshot")


if __name__ == "__main__":
    unittest.main()
