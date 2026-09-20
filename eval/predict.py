#!/usr/bin/env python3
"""Runs the trained tagger over the gold set and writes its spans for the Rust benchmark.

Two numbers come out of this, and they measure different things:

  Span F1        how well the tagger IDENTIFIES - purely the model's job.
  Exact match    tagger + Rust resolution end to end - reported by `bench --predictions`,
                 and directly comparable to the parser's 47.8%.

Keeping them apart matters. A date the tagger correctly finds but Rust cannot resolve
("tmrw") costs exact match without being a tagging failure, and fixing it belongs in the
resolver, not the model.

    python eval/predict.py
    python eval/predict.py --model eval/model --out eval/predictions.jsonl

Gold is never trained on, so this is a clean test set.
"""

import argparse
import json
import sys
import time
from pathlib import Path

import torch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from train import ID2LABEL, decode_spans, entity_type, span_f1  # noqa: E402


def main():
    ap = argparse.ArgumentParser(description="Tag the gold set with the trained model.")
    ap.add_argument("--model", default=str(HERE / "model"))
    ap.add_argument("--gold", default=str(HERE / "gold.json"))
    ap.add_argument("--out", default=str(HERE / "predictions.jsonl"))
    ap.add_argument("--max-len", type=int, default=64)
    ap.add_argument("--skip", action="append", default=[], help="exclude a category")
    ap.add_argument("--show", type=int, default=0, help="print N predictions")
    args = ap.parse_args()

    from transformers import AutoModelForTokenClassification, AutoTokenizer

    model_dir = Path(args.model)
    if not (model_dir / "config.json").exists():
        sys.exit(f"No model at {model_dir} - run: python eval/train.py")

    with open(args.gold, encoding="utf-8") as f:
        cases = json.load(f)["cases"]
    if args.skip:
        cases = [c for c in cases if not any(k in args.skip for k in c.get("categories", []))]

    tokenizer = AutoTokenizer.from_pretrained(model_dir)
    model = AutoModelForTokenClassification.from_pretrained(model_dir)
    model.eval()

    texts = [c["text"] for c in cases]
    enc = tokenizer(
        texts, return_offsets_mapping=True, truncation=True,
        max_length=args.max_len, padding=True, return_tensors="pt",
    )
    offsets = enc.pop("offset_mapping")

    started = time.perf_counter()
    with torch.no_grad():
        logits = model(**enc).logits
    elapsed = time.perf_counter() - started
    preds = logits.argmax(-1)

    rows, gold_sets, pred_sets = [], [], []
    for i, case in enumerate(cases):
        spans = decode_spans(
            case["text"], offsets[i].tolist(), preds[i].tolist(), enc["attention_mask"][i].tolist()
        )
        rows.append({
            "id": case["id"],
            "text": case["text"],
            "today": case["today"],
            "entities": [{"start": s["start"], "end": s["end"], "label": s["type"], "text": s["text"]} for s in spans],
        })
        pred_sets.append({(s["start"], s["end"], s["type"]) for s in spans})
        gold = set()
        for ent in case.get("spans", []):
            # Gold stores spans as surface text; resolve to offsets the same way bench does.
            start = case["text"].find(ent["text"])
            if start < 0:
                continue
            t = entity_type({"label": ent["label"], "canonical": (case["expect"].get("priority") or "")})
            if t:
                gold.add((start, start + len(ent["text"]), t))
        gold_sets.append(gold)

    micro, per_type = span_f1(gold_sets, pred_sets)
    n = len(cases)
    print(f"\n  Gold set  ·  {n} cases  ·  {elapsed * 1000 / n:.1f} ms/sentence (batched, CPU)\n")
    print(f"  Span F1 (micro): {micro:.4f}\n")
    print(f"  {'Type':<16}{'P':>8}{'R':>8}{'F1':>8}{'n':>8}")
    print("  " + "-" * 48)
    for t, (p, r, f, cnt) in sorted(per_type.items(), key=lambda kv: kv[1][2]):
        print(f"  {t:<16}{p:>8.3f}{r:>8.3f}{f:>8.3f}{cnt:>8}")

    if args.show:
        print()
        for row in rows[:args.show]:
            print(f"  {row['text']!r}")
            for e in row["entities"]:
                print(f"      {e['label']:<14} {e['text']!r}")

    with open(args.out, "w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(f"\n  Wrote {len(rows)} predictions to {args.out}")
    print("  Now: cargo run --bin bench -- --predictions eval/predictions.jsonl\n")


if __name__ == "__main__":
    main()
