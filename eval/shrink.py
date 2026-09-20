#!/usr/bin/env python3
"""Shrinks the trained tagger so it can ship inside the Ordo binary.

In a model this small the embedding table is most of the weights:

    MiniLM-L6:  30,522 x 384 = 11.7M of 22.6M params

Ordo's domain is to-do sentences, not all of English, so most of those rows are never
touched. This prunes the vocabulary, remaps the embedding matrix, and re-finetunes briefly
so the model settles into the smaller table.

    python eval/shrink.py --prune                 # write the pruned model
    python eval/shrink.py --quantize              # int8 the pruned model
    python eval/shrink.py --prune --quantize

IMPORTANT: the kept vocabulary is chosen from the TRAINING data only, never from gold.
Choosing it from gold would guarantee no unseen token ever appears at evaluation time,
which is not how deployment works. Unseen words are meant to decompose into subwords, so
every single character and every short "##" continuation is kept regardless of frequency -
that is what makes the degradation graceful instead of a wall of [UNK].
"""

import argparse
import json
import shutil
from collections import Counter
from pathlib import Path

import torch

HERE = Path(__file__).resolve().parent
SPECIALS = ["[PAD]", "[UNK]", "[CLS]", "[SEP]", "[MASK]"]


def used_tokens(tokenizer, texts):
    seen = Counter()
    for i in range(0, len(texts), 512):
        for ids in tokenizer(texts[i:i + 512])["input_ids"]:
            seen.update(ids)
    return seen


def prune(src, dst, data_path, keep_floor):
    from transformers import AutoModelForTokenClassification, AutoTokenizer

    tok = AutoTokenizer.from_pretrained(src)
    model = AutoModelForTokenClassification.from_pretrained(src)
    vocab = tok.get_vocab()
    inv = {v: k for k, v in vocab.items()}

    texts = [json.loads(l)["text"] for l in open(data_path, encoding="utf-8") if l.strip()]
    seen = used_tokens(tok, texts)
    print(f"  training corpus      {len(texts)} sentences")
    print(f"  distinct token ids   {len(seen)} of {len(vocab)}")

    keep = set()
    for s in SPECIALS:
        if s in vocab:
            keep.add(vocab[s])
    keep.update(seen)
    # Graceful degradation for words the training data never contained.
    for tokstr, idx in vocab.items():
        body = tokstr[2:] if tokstr.startswith("##") else tokstr
        if len(body) <= keep_floor:
            keep.add(idx)

    keep = sorted(keep)
    print(f"  kept                 {len(keep)} ({len(keep) / len(vocab):.1%} of the table)")

    old_to_new = {old: new for new, old in enumerate(keep)}
    emb = model.bert.embeddings.word_embeddings.weight.data
    new_emb = emb[torch.tensor(keep)].clone()

    model.bert.embeddings.word_embeddings = torch.nn.Embedding(len(keep), emb.shape[1])
    model.bert.embeddings.word_embeddings.weight.data = new_emb
    model.config.vocab_size = len(keep)

    dst = Path(dst)
    if dst.exists():
        shutil.rmtree(dst)
    dst.mkdir(parents=True)
    model.save_pretrained(dst, safe_serialization=True)

    # A WordPiece vocab is just token -> id, so writing vocab.txt in the new id order is
    # the whole remap. The fast tokenizer.json is rebuilt from it on load.
    (dst / "vocab.txt").write_text("\n".join(inv[o] for o in keep) + "\n", encoding="utf-8")
    cfg = {
        "do_lower_case": getattr(tok, "do_lower_case", True),
        "model_max_length": 512,
        "tokenizer_class": "BertTokenizer",
        **{f"{n.lower().strip('[]')}_token": n for n in SPECIALS},
    }
    (dst / "tokenizer_config.json").write_text(json.dumps(cfg, indent=2), encoding="utf-8")
    for extra in ("labels.json",):
        if (Path(src) / extra).exists():
            shutil.copy(Path(src) / extra, dst / extra)

    params = sum(p.numel() for p in model.parameters())
    print(f"  params               {params / 1e6:.2f}M  ({params * 4 / 1e6:.0f} MB fp32)")
    print(f"  written to           {dst}")
    return len(keep), old_to_new


def quantize(src, dst):
    """Dynamic int8 over the linear layers - the standard CPU-inference shrink."""
    from transformers import AutoModelForTokenClassification

    model = AutoModelForTokenClassification.from_pretrained(src)
    model.eval()
    q = torch.quantization.quantize_dynamic(model, {torch.nn.Linear}, dtype=torch.qint8)
    dst = Path(dst)
    if dst.exists():
        shutil.rmtree(dst)
    dst.mkdir(parents=True)
    torch.save(q.state_dict(), dst / "model-int8.pt")
    for f in Path(src).iterdir():
        if f.suffix in (".json", ".txt"):
            shutil.copy(f, dst / f.name)
    size = (dst / "model-int8.pt").stat().st_size
    print(f"  int8 state dict      {size / 1e6:.1f} MB  ->  {dst}")


def main():
    ap = argparse.ArgumentParser(description="Shrink the Ordo span tagger.")
    ap.add_argument("--src", default=str(HERE / "model-v3"))
    ap.add_argument("--pruned", default=str(HERE / "model-pruned"))
    ap.add_argument("--quantized", default=str(HERE / "model-int8"))
    ap.add_argument("--data", default=str(HERE / "synthetic.jsonl"))
    ap.add_argument("--keep-floor", type=int, default=2,
                    help="keep every token whose body is this short, for graceful [UNK] avoidance")
    ap.add_argument("--prune", action="store_true")
    ap.add_argument("--quantize", action="store_true")
    args = ap.parse_args()

    if args.prune:
        print("\n  Pruning vocabulary")
        prune(args.src, args.pruned, args.data, args.keep_floor)
        print("\n  Re-finetune before measuring - the embedding rows moved:")
        print(f"    python eval/train.py --model {args.pruned} --out {args.pruned}-tuned --epochs 2\n")
    if args.quantize:
        print("\n  Quantizing")
        quantize(args.src, args.quantized)


if __name__ == "__main__":
    main()
