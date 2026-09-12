#!/usr/bin/env python3
"""Summarize governance exports and retain explicit reviewer labels offline."""
import argparse
import collections
import csv
import hashlib
import json
import re
from pathlib import Path

LABELS = {"benign", "credible_concern", "unresolved"}


def summarize(rows, reviews=None):
    reviews = reviews or {}
    groups = {}
    totals = collections.Counter()
    for row in rows:
        if None in row or any(value is None for value in row.values()):
            raise ValueError("Malformed CSV row")
        totals[row["plane"]] += 1
        if row["outcome"] == "allow":
            continue
        fingerprint = re.search(r"fingerprint:([0-9a-f]{64})", row["reason"])
        session = row.get("session_id", "")
        evidence = fingerprint.group(1) if fingerprint else row["reason"]
        key = [row["plane"], row["user"], session, row["policy_or_category"], row.get("phase") or row["scope"], evidence]
        group_id = hashlib.sha256(json.dumps(key).encode()).hexdigest()
        review = reviews.get(group_id, {})
        label = review.get("label", "unresolved")
        if label not in LABELS:
            raise ValueError(f"Invalid review label: {label}")
        item = groups.setdefault(group_id, {
            "group_id": group_id, "category": row["policy_or_category"],
            "user": row["user"], "session_id": session, "occurrences": 0,
            "first_seen": row["at"], "last_seen": row["at"],
            "grouping": "fingerprint" if fingerprint and session else "legacy_reason_only",
            "label": label, "note": review.get("note", ""),
            "example_reason": row["reason"],
        })
        item["occurrences"] += 1
        item["first_seen"] = min(item["first_seen"], row["at"])
        item["last_seen"] = max(item["last_seen"], row["at"])
    return {
        "rows": len(rows), "planes": dict(totals),
        "export_truncated": any(row.get("export_truncated") == "true" for row in rows),
        "missing_correlation": sum(not any(row.get(k) for k in ("trace_id", "call_id", "request_id")) for row in rows),
        "note": "Groups are review candidates, not confirmed incidents. Legacy masked reasons cannot identify distinct secrets. Labels do not change runtime policy.",
        "groups": sorted(groups.values(), key=lambda item: (-item["occurrences"], item["group_id"])),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("csv", type=Path)
    parser.add_argument("--reviews", type=Path)
    parser.add_argument("--review-template", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    with args.csv.open(newline="", encoding="utf-8-sig") as stream:
        rows = list(csv.DictReader(stream))
    reviews = {}
    if args.reviews:
        with args.reviews.open(newline="", encoding="utf-8") as stream:
            for row in csv.DictReader(stream):
                if row["group_id"] in reviews:
                    raise ValueError("Duplicate group in review file")
                reviews[row["group_id"]] = row
    report = summarize(rows, reviews)
    if args.review_template:
        with args.review_template.open("x", newline="", encoding="utf-8") as stream:
            writer = csv.DictWriter(stream, fieldnames=["group_id", "label", "note"])
            writer.writeheader()
            writer.writerows({key: row[key] for key in writer.fieldnames} for row in report["groups"])
    text = json.dumps(report, indent=2) + "\n"
    if args.output:
        args.output.write_text(text, encoding="utf-8")
    else:
        print(text, end="")


if __name__ == "__main__":
    main()
