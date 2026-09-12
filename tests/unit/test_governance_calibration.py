import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("calibration", Path(__file__).resolve().parents[2] / "scripts/governance-calibration.py")
calibration = importlib.util.module_from_spec(spec)
spec.loader.exec_module(calibration)


def row(user="one", session="session-one"):
    return dict(plane="chain", outcome="warn", user=user, session_id=session,
                policy_or_category="secret_scan", scope="user", phase="request",
                reason="fingerprint:" + "a" * 64, at="2026-09-11T00:00:00Z", trace_id="trace-one")


class CalibrationTests(unittest.TestCase):
    def test_repeated_evidence_groups_without_crossing_users_or_sessions(self):
        report = calibration.summarize([row(), row(), row(user="two"), row(session="session-two")])
        self.assertEqual(len(report["groups"]), 3)
        self.assertEqual(report["groups"][0]["occurrences"], 2)

    def test_labels_remain_explicit_and_do_not_change_evidence(self):
        original = calibration.summarize([row()])["groups"][0]
        report = calibration.summarize([row()], {original["group_id"]: {"label": "benign", "note": "reviewed"}})
        self.assertEqual(report["groups"][0]["label"], "benign")
        self.assertEqual(report["groups"][0]["occurrences"], 1)
        with self.assertRaises(ValueError):
            calibration.summarize([row()], {original["group_id"]: {"label": "auto_allow"}})

    def test_legacy_and_truncated_exports_are_not_claimed_complete(self):
        legacy = row(session="")
        legacy["export_truncated"] = "true"
        report = calibration.summarize([legacy])
        self.assertTrue(report["export_truncated"])
        self.assertEqual(report["groups"][0]["grouping"], "legacy_reason_only")


if __name__ == "__main__":
    unittest.main()
