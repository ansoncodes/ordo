#!/usr/bin/env python3
"""Fine-tunes a small encoder to tag Ordo task language.

BIO token classification over a pretrained encoder. The model labels the input; it never
generates, so it cannot reword a title or invent a value - the two failure modes that made
the 0.5B generative model risky.

    python eval/train.py --smoke                    # 300 examples, 1 epoch - does it run
    python eval/train.py                            # the real thing
    python eval/train.py --epochs 4 --model prajjwal1/bert-mini

Deliberately unpruned and unquantized: this establishes whether tagging solves the problem
at all. Shrinking comes after, measured against this number so each optimization's cost is
visible. Trains on eval/synthetic.jsonl only; eval/gold.json is never seen during training.

PRIORITY is split into PRIORITY_P1..P4, so the 4-way class falls out of the tag itself
rather than needing a second head - "pretty important" is tagged B-PRIORITY_P2 directly.
"""

import argparse
import json
import math
import random
import sys
import time
from pathlib import Path

import torch
from torch.utils.data import DataLoader, Dataset

HERE = Path(__file__).resolve().parent
SYNTHETIC = HERE / "synthetic.jsonl"
OUT_DIR = HERE / "model"

# Entity types. PRIORITY carries its class in the label; everything else resolves in Rust.
TYPES = [
    "TITLE", "DATE", "TIME", "DURATION", "RECURRENCE", "PROJECT", "TAG",
    "SCHEDULED", "STEP", "NOTE",
    "PRIORITY_P1", "PRIORITY_P2", "PRIORITY_P3", "PRIORITY_P4",
]
LABELS = ["O"] + [f"{p}-{t}" for t in TYPES for p in ("B", "I")]
LABEL2ID = {l: i for i, l in enumerate(LABELS)}
ID2LABEL = {i: l for l, i in LABEL2ID.items()}


def entity_type(ent):
    """PRIORITY spans become PRIORITY_<class>; everything else keeps its label."""
    if ent["label"] == "PRIORITY":
        canon = (ent.get("canonical") or "").upper()
        return f"PRIORITY_{canon}" if f"PRIORITY_{canon}" in TYPES else None
    return ent["label"] if ent["label"] in TYPES else None


def load_jsonl(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def encode(row, tokenizer, max_len):
    """Tokenizes and projects character spans onto wordpiece tokens as BIO labels."""
    enc = tokenizer(
        row["text"], return_offsets_mapping=True, truncation=True,
        max_length=max_len, padding="max_length",
    )
    offsets = enc.pop("offset_mapping")
    labels = []
    spans = []
    for ent in row["entities"]:
        t = entity_type(ent)
        if t:
            spans.append((ent["start"], ent["end"], t))
    spans.sort()

    for idx, (start, end) in enumerate(offsets):
        # Special tokens and padding carry offset (0, 0) and are excluded from the loss.
        if start == end or enc["attention_mask"][idx] == 0:
            labels.append(-100)
            continue
        tag = "O"
        for s, e, t in spans:
            if start >= s and end <= e:
                tag = f"{'B' if start == s else 'I'}-{t}"
                break
            # A token straddling a boundary belongs to whichever span holds most of it.
            if start < e and end > s:
                overlap = min(end, e) - max(start, s)
                if overlap * 2 >= (end - start):
                    tag = f"{'B' if start <= s else 'I'}-{t}"
                    break
        labels.append(LABEL2ID[tag])
    enc["labels"] = labels
    return enc


class Tagged(Dataset):
    def __init__(self, rows, tokenizer, max_len):
        self.items = [encode(r, tokenizer, max_len) for r in rows]

    def __len__(self):
        return len(self.items)

    def __getitem__(self, i):
        it = self.items[i]
        return {k: torch.tensor(v) for k, v in it.items()}


def decode_spans(text, offsets, tag_ids, mask):
    """BIO tags back to character spans."""
    out, cur = [], None
    for idx, tid in enumerate(tag_ids):
        if idx >= len(offsets) or mask[idx] == 0:
            break
        start, end = offsets[idx]
        if start == end:
            continue
        tag = ID2LABEL.get(int(tid), "O")
        if tag == "O":
            if cur:
                out.append(cur)
                cur = None
            continue
        prefix, _, typ = tag.partition("-")
        if prefix == "B" or cur is None or cur["type"] != typ:
            if cur:
                out.append(cur)
            cur = {"type": typ, "start": start, "end": end}
        else:
            cur["end"] = end
    if cur:
        out.append(cur)
    for s in out:
        s["text"] = text[s["start"]:s["end"]]
    return out


def span_f1(gold_sets, pred_sets):
    """Exact-match span F1, plus a per-type breakdown."""
    from collections import defaultdict
    tp = defaultdict(int)
    fp = defaultdict(int)
    fn = defaultdict(int)
    for gold, pred in zip(gold_sets, pred_sets):
        g = {(s, e, t) for s, e, t in gold}
        p = {(s, e, t) for s, e, t in pred}
        for item in p & g:
            tp[item[2]] += 1
        for item in p - g:
            fp[item[2]] += 1
        for item in g - p:
            fn[item[2]] += 1
    rows = {}
    for t in set(list(tp) + list(fp) + list(fn)):
        prec = tp[t] / (tp[t] + fp[t]) if tp[t] + fp[t] else 0.0
        rec = tp[t] / (tp[t] + fn[t]) if tp[t] + fn[t] else 0.0
        f1 = 2 * prec * rec / (prec + rec) if prec + rec else 0.0
        rows[t] = (prec, rec, f1, tp[t] + fn[t])
    TP, FP, FN = sum(tp.values()), sum(fp.values()), sum(fn.values())
    prec = TP / (TP + FP) if TP + FP else 0.0
    rec = TP / (TP + FN) if TP + FN else 0.0
    micro = 2 * prec * rec / (prec + rec) if prec + rec else 0.0
    return micro, rows


@torch.no_grad()
def evaluate(model, rows, tokenizer, max_len, device, batch_size=64):
    model.eval()
    gold_sets, pred_sets = [], []
    for i in range(0, len(rows), batch_size):
        chunk = rows[i:i + batch_size]
        enc = tokenizer(
            [r["text"] for r in chunk], return_offsets_mapping=True, truncation=True,
            max_length=max_len, padding=True, return_tensors="pt",
        )
        offsets = enc.pop("offset_mapping")
        logits = model(**{k: v.to(device) for k, v in enc.items()}).logits
        preds = logits.argmax(-1).cpu()
        for j, row in enumerate(chunk):
            spans = decode_spans(row["text"], offsets[j].tolist(), preds[j].tolist(), enc["attention_mask"][j].tolist())
            pred_sets.append({(s["start"], s["end"], s["type"]) for s in spans})
            gold = set()
            for ent in row["entities"]:
                t = entity_type(ent)
                if t:
                    gold.add((ent["start"], ent["end"], t))
            gold_sets.append(gold)
    return span_f1(gold_sets, pred_sets)


def main():
    ap = argparse.ArgumentParser(description="Fine-tune a span tagger for Ordo.")
    ap.add_argument("--model", default="sentence-transformers/all-MiniLM-L6-v2")
    ap.add_argument("--data", default=str(SYNTHETIC))
    ap.add_argument("--out", default=str(OUT_DIR))
    ap.add_argument("--epochs", type=int, default=3)
    ap.add_argument("--batch-size", type=int, default=32)
    ap.add_argument("--lr", type=float, default=5e-5)
    ap.add_argument("--max-len", type=int, default=64)
    ap.add_argument("--val-share", type=float, default=0.05)
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--smoke", action="store_true", help="300 examples, 1 epoch")
    args = ap.parse_args()

    from transformers import AutoModelForTokenClassification, AutoTokenizer

    torch.manual_seed(args.seed)
    random.seed(args.seed)

    data_path = Path(args.data)
    if not data_path.exists():
        sys.exit(f"{data_path} not found - run: python eval/generate.py --count 20000")
    rows = load_jsonl(data_path)
    random.shuffle(rows)
    if args.smoke:
        rows, args.epochs = rows[:300], 1

    n_val = max(1, int(len(rows) * args.val_share))
    val_rows, train_rows = rows[:n_val], rows[n_val:]

    print(f"\n  model      {args.model}")
    print(f"  train      {len(train_rows)} examples")
    print(f"  val        {len(val_rows)} examples (held-out synthetic)")
    print(f"  labels     {len(LABELS)} BIO tags over {len(TYPES)} entity types")

    tokenizer = AutoTokenizer.from_pretrained(args.model)
    model = AutoModelForTokenClassification.from_pretrained(
        args.model, num_labels=len(LABELS), id2label=ID2LABEL, label2id=LABEL2ID,
    )
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    model.to(device)
    params = sum(p.numel() for p in model.parameters())
    print(f"  params     {params / 1e6:.1f}M  ({params * 4 / 1e6:.0f} MB fp32)")
    print(f"  device     {device}\n")

    train_ds = Tagged(train_rows, tokenizer, args.max_len)
    loader = DataLoader(train_ds, batch_size=args.batch_size, shuffle=True)

    opt = torch.optim.AdamW(model.parameters(), lr=args.lr, weight_decay=0.01)
    total = len(loader) * args.epochs
    warmup = max(1, int(total * 0.06))

    def lr_at(step):
        if step < warmup:
            return step / warmup
        return max(0.0, (total - step) / max(1, total - warmup))

    sched = torch.optim.lr_scheduler.LambdaLR(opt, lr_at)

    step, started = 0, time.time()
    for epoch in range(args.epochs):
        model.train()
        running = 0.0
        for batch in loader:
            batch = {k: v.to(device) for k, v in batch.items()}
            loss = model(**batch).loss
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            opt.step()
            sched.step()
            opt.zero_grad()
            running += loss.item()
            step += 1
            if step % 50 == 0:
                rate = step / (time.time() - started)
                eta = (total - step) / rate / 60
                print(f"    epoch {epoch + 1}  step {step}/{total}  loss {running / 50:.4f}  eta {eta:.1f}m")
                running = 0.0
        micro, _ = evaluate(model, val_rows, tokenizer, args.max_len, device)
        print(f"  epoch {epoch + 1} done  ·  val span F1 {micro:.4f}")

    micro, per_type = evaluate(model, val_rows, tokenizer, args.max_len, device)
    print(f"\n  Held-out synthetic  ·  micro span F1 {micro:.4f}\n")
    print(f"  {'Type':<16}{'P':>8}{'R':>8}{'F1':>8}{'n':>8}")
    print("  " + "-" * 48)
    for t, (p, r, f, n) in sorted(per_type.items(), key=lambda kv: kv[1][2]):
        print(f"  {t:<16}{p:>8.3f}{r:>8.3f}{f:>8.3f}{n:>8}")

    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    model.save_pretrained(out, safe_serialization=True)
    tokenizer.save_pretrained(out)
    (out / "labels.json").write_text(json.dumps({"labels": LABELS, "types": TYPES}, indent=2), encoding="utf-8")
    mins = (time.time() - started) / 60
    print(f"\n  Saved to {out}  ({mins:.1f} min)")
    print("  Next: python eval/predict.py   then   cargo run --bin bench -- --predictions eval/predictions.jsonl\n")


if __name__ == "__main__":
    main()
