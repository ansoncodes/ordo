#!/usr/bin/env python3
"""Compiles hand-written cases from cases.txt into gold.json.

Writing gold cases directly as JSON means repeating every span's surface text and keeping
a dozen `expect` fields in sync by hand. This lets a case be one marked-up line, with
everything derivable derived:

    conv-020 | 2026-09-14 | conversational,filler-prefix
    be sure to [ring the dentist|TITLE] [first thing|TIME:09:00] [on tuesday|DATE:2026-09-15]

`expect.title` comes from the TITLE span, tags from TAG spans, steps from STEP spans, notes
from NOTE spans, project from the PROJECT span. Only values needing *resolution* are written
out, inline after the label. Priority carries its class in the label: [urgent|PRIORITY_P1].

Optional extra lines on a case:
    > title:fix the printer      override a derived field (rewording cases)
    # note text                  why an expectation is what it is
    !review                      the expectation is a product decision, not a fact

    python eval/compile_gold.py            # merge cases.txt into gold.json
    python eval/compile_gold.py --check    # parse and report, write nothing
"""

import argparse
import json
import re
import sys
from collections import OrderedDict
from pathlib import Path

HERE = Path(__file__).resolve().parent
CASES = HERE / "cases.txt"
GOLD = HERE / "gold.json"

SPAN = re.compile(r"\[([^\[\]|]+)\|([A-Z_0-9]+)(?::([^\]]+))?\]")

VALUE_SLOT = {
    "DATE": "due_date",
    "TIME": "due_time",
    "DURATION": "estimate_minutes",
    "RECURRENCE": "recurrence",
    "SCHEDULED": "scheduled",
}


def parse_case(block, lineno):
    lines = [l for l in block.strip().splitlines() if l.strip()]
    if len(lines) < 2:
        raise ValueError(f"line {lineno}: a case needs a header and a text line")

    head = [p.strip() for p in lines[0].split("|")]
    if len(head) < 2:
        raise ValueError(f"line {lineno}: header must be 'id | today | categories'")
    case_id, today = head[0], head[1]
    categories = [c.strip() for c in head[2].split(",")] if len(head) > 2 and head[2] else []

    marked = lines[1]
    overrides, note, review = {}, None, False
    for extra in lines[2:]:
        if extra.startswith(">"):
            for pair in extra[1:].split("|"):
                if ":" in pair:
                    k, _, v = pair.strip().partition(":")
                    overrides[k.strip()] = v.strip()
        elif extra.startswith("#"):
            note = extra[1:].strip()
        elif extra.startswith("!review"):
            review = True
        else:
            raise ValueError(f"line {lineno}: unrecognised line {extra!r}")

    # Strip the markup to recover the sentence, recording each span as surface text.
    text, spans, pos = "", [], 0
    for m in SPAN.finditer(marked):
        text += marked[pos:m.start()]
        surface, label, value = m.group(1), m.group(2), m.group(3)
        spans.append({"label": label, "text": surface, "value": value})
        text += surface
        pos = m.end()
    text += marked[pos:]
    text = re.sub(r"\s+", " ", text).strip()

    expect = {}
    gold_spans = []
    for s in spans:
        label, surface, value = s["label"], s["text"], s["value"]
        if label == "TITLE":
            expect["title"] = surface
        elif label.startswith("PRIORITY"):
            cls = label.partition("_")[2].lower()
            if not cls:
                raise ValueError(f"{case_id}: PRIORITY needs a class, e.g. PRIORITY_P2")
            expect["priority"] = cls
            label = "PRIORITY"
        elif label == "TAG":
            expect.setdefault("tags", []).append(surface.lstrip("@"))
        elif label == "STEP":
            expect.setdefault("subtasks", []).append(surface.lstrip("+"))
        elif label == "NOTE":
            expect["notes"] = surface
        elif label == "PROJECT":
            name = surface.lstrip("#")
            head_word, _, rest = name.partition(" ")
            if head_word.lower() in ("for", "in", "under", "into") and rest:
                name = rest
            expect["project"] = value or name
            value = None
        elif label in VALUE_SLOT:
            if value is None:
                raise ValueError(f"{case_id}: {label} span {surface!r} needs a value, e.g. [{surface}|{label}:...]")
            slot = VALUE_SLOT[label]
            expect[slot] = int(value) if slot == "estimate_minutes" else value
        else:
            raise ValueError(f"{case_id}: unknown label {label!r}")
        gold_spans.append({"label": label, "text": surface})

    # A time with no date means today: "haircut at 4" is today at 4, not an undated task.
    # Applied here rather than as an override on every such case so the rule stays one
    # statement instead of twenty copies that can drift apart.
    labels = {s["label"] for s in gold_spans}
    if "TIME" in labels and not labels & {"DATE", "SCHEDULED"} and "due_date" not in expect:
        expect["due_date"] = today

    for k, v in overrides.items():
        expect[k] = [] if v == "-" and k in ("tags", "subtasks") else ("" if v == "-" else v)

    if "title" not in expect:
        raise ValueError(f"{case_id}: no TITLE span and no title override")

    # The span must still be findable in the stripped sentence - bench resolves it that way.
    for s in gold_spans:
        if s["text"] not in text:
            raise ValueError(f"{case_id}: span {s['text']!r} is not in the compiled text")

    case = OrderedDict(id=case_id, text=text, today=today, categories=categories)
    if note:
        case["note"] = note
    if review:
        case["review"] = True
    case["spans"] = gold_spans
    case["expect"] = expect
    return case


def parse_file(path):
    raw = Path(path).read_text(encoding="utf-8")
    blocks, cur, start, lineno = [], [], 1, 0
    for line in raw.splitlines():
        lineno += 1
        if line.strip().startswith("//"):
            continue
        if not line.strip():
            if cur:
                blocks.append(("\n".join(cur), start))
                cur = []
            continue
        if not cur:
            start = lineno
        cur.append(line)
    if cur:
        blocks.append(("\n".join(cur), start))

    cases, errors = [], []
    for block, ln in blocks:
        try:
            cases.append(parse_case(block, ln))
        except ValueError as e:
            errors.append(str(e))
    return cases, errors


def main():
    ap = argparse.ArgumentParser(description="Compile cases.txt into gold.json.")
    ap.add_argument("--cases", default=str(CASES))
    ap.add_argument("--gold", default=str(GOLD))
    ap.add_argument("--check", action="store_true", help="parse and report, write nothing")
    args = ap.parse_args()

    if not Path(args.cases).exists():
        sys.exit(f"{args.cases} not found")

    cases, errors = parse_file(args.cases)
    if errors:
        print(f"{len(errors)} problem(s):", file=sys.stderr)
        for e in errors:
            print(f"  {e}", file=sys.stderr)
        sys.exit(1)

    ids = [c["id"] for c in cases]
    dupes = {i for i in ids if ids.count(i) > 1}
    if dupes:
        sys.exit(f"duplicate ids in cases.txt: {sorted(dupes)}")

    with open(args.gold, encoding="utf-8") as f:
        gold = json.load(f, object_pairs_hook=OrderedDict)

    existing = {c["id"]: i for i, c in enumerate(gold["cases"])}
    added = replaced = 0
    for case in cases:
        if case["id"] in existing:
            gold["cases"][existing[case["id"]]] = case
            replaced += 1
        else:
            gold["cases"].append(case)
            added += 1

    print(f"  parsed {len(cases)} cases from {Path(args.cases).name}")
    print(f"  {added} new, {replaced} replaced  ->  {len(gold['cases'])} total in gold.json")
    if args.check:
        print("  --check: nothing written")
        return

    with open(args.gold, "w", encoding="utf-8") as f:
        json.dump(gold, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print("  Now: cargo run --bin bench      (it validates every span)")


if __name__ == "__main__":
    main()
