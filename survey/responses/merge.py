#!/usr/bin/env python3
"""Merge survey/responses/*.csv|*.json into one dataset for analysis.
Usage: python3 survey/responses/merge.py [--out dataset.json]"""
import csv, json, sys, glob
out = sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else None
rows = []
for f in sorted(glob.glob("survey/responses/*.json")):
    try:
        data = json.load(open(f))
        rows += data if isinstance(data, list) else [data]
    except Exception as e:
        print(f"skip {f}: {e}")
for f in sorted(glob.glob("survey/responses/*.csv")):
    try:
        rows += list(csv.DictReader(open(f)))
    except Exception as e:
        print(f"skip {f}: {e}")
print(f"{len(rows)} responses from manual inbox")
print(f"cats: {sorted({str(r.get('cat')) for r in rows})}")
if out:
    json.dump(rows, open(out, "w"), indent=1)
    print(f"wrote {out}")
