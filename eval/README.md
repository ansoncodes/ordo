# Ordo language evaluation

`gold.json` is the hand-written benchmark for how well Ordo understands what people type.
It is the number any replacement model has to beat, and later the seed for training data.

```
cargo run --bin bench                             # the table
cargo run --bin bench -- -v                       # every failure, slot by slot
cargo run --bin bench -- --category soft-priority # one slice
cargo run --bin bench -- --skip semantic-inference
cargo run --bin bench -- --export-spans eval/spans.jsonl
```

## Rules

**Hand-written only.** Never generate cases into this file. Synthetic data goes elsewhere and
is trained on; this is what you measure against. Mixing the two makes the number meaningless.

**`expect` is the ideal, not what the parser can currently do.** The gap is the point. If the
parser leaves "don't forget to" in the title, the case still expects `call Arun` and fails.

**Pin `today` on every case.** Date expectations are absolute (`2026-09-15`), so a relative
reference date would silently rot them. Most cases use `2026-09-14`, a Monday — convenient
because "friday", "next week" and "the weekend" all resolve forward unambiguously.

**Flag product decisions with `"review": true`.** Some expectations encode a choice nobody has
made yet — what "after lunch" means, whether `asap` also implies a due date, whether a title may
be reworded out of passive voice. Mark them and write a `note`. The benchmark reports the count
so a headline number built on unconfirmed guesses is visible as such.

## Spans

Spans carry their **surface text**, not character offsets:

```json
{"label": "DATE", "text": "tmrw"}
```

Offsets in a hand-edited file rot the moment `text` changes — one added word shifts everything
after it, silently. `bench` resolves offsets at load time, and fails loudly if a span is missing
or matches more than once (add `"occurrence": 2` for the second match). `--export-spans` emits the
offset-resolved form as JSONL when the training pipeline needs it.

Labels: `TITLE` `DATE` `TIME` `PRIORITY` `PROJECT` `TAG` `DURATION` `RECURRENCE` `SCHEDULED`
`STEP` `NOTE`.

## Scoring

Per slot, three numbers:

- **Accuracy** — correct across all cases. Correctly producing *nothing* counts, which is how
  hard negatives earn credit.
- **Recall (present)** — correct among cases where the gold value exists. This is the real
  difficulty measure; accuracy alone is flattered by all the cases with no value in that slot.
- **Invented** — gold had no value and the engine produced one anyway. The hallucination counter.
  The deterministic parser scores 0 here. Any model that replaces it has to stay at 0.

**Exact match** requires every slot correct. It is harsh on purpose: a task with a perfect date
and a title full of filler is not a good result.

## Categories

Used to slice the report and to aim synthetic generation at real weaknesses. Current set:

`explicit-syntax` `control` · `hard-negative` `no-metadata` `ambiguous-date` `ambiguous-project`
`ambiguous-duration` `url` · `typo` `abbreviation` `slang` · `conversational` `filler-prefix`
`filler-clause` `passive-voice` `punctuation-emphasis` · `soft-priority` · `vague-time`
`fuzzy-time` `time` `relative-date` `day-of-month` `leading-date` `scheduled` · `duration`
`recurrence` `uncommon-phrasing` · `project` `bare-project-name` `tag` · `checklist` `notes` ·
`very-short` `very-long` · `semantic-inference` `unresolvable-date`

Add freely; a case can carry several. `semantic-inference` marks cases needing world knowledge
("after the client gets back") — exclude that tier with `--skip` when judging whether a tagger
is good enough, since neither the parser nor a small encoder is expected to handle it.

## Synthetic training data

`generate.py` builds training data for the span tagger. Spans are correct **by construction** —
sentences are assembled from labelled components, never annotated after the fact — and every
example is verified before it is written: if `text[start:end]` ever disagrees with the recorded
surface, the run aborts.

```
python eval/generate.py --count 20000 --out eval/synthetic.jsonl
python eval/generate.py --count 12 --preview     # read some
python eval/generate.py --count 5000 --stats     # check the mixture
```

Output is JSONL, the same shape as `bench --export-spans`, with a `canonical` field giving the
normalized form the Rust resolver should receive (`"tmrw"` → `tomorrow`, `"pretty important"` →
`p2`). The generator normalizes; it never resolves. Dates stay as keywords because resolution is
Rust's job, and baking date arithmetic into training data would only teach the model to guess at
something the parser already gets right 91% of the time.

**The mixture is weighted at the measured gaps, not at intuition.** `W` at the top of the file
carries the rationale: heavy on conversational filler (Title 50.7%), soft priority language
(Priority recall 26.7%) and fuzzy times (Time recall 58.8%); light on dates, recurrence and
projects, which the parser already handles at 91–100% and which only need enough presence to stop
the tagger regressing.

**Generation is quota-based, not probabilistic.** An earlier probabilistic version was
silently starved of hard negatives — traps landed at 0.085% against a 8% target — because
deduplication hits the simplest sentences hardest: they have the smallest unique space, so they
collide constantly while metadata-rich sentences sail through. Buckets are filled to explicit
quotas instead, and a bucket that cannot fill says so rather than skewing the mixture quietly.
If you widen `W["no_metadata"]` or `W["trap"]`, add entries to `TITLES` and `TRAPS` to match.

Regenerate rather than commit: `eval/synthetic.jsonl` is reproducible from `--seed`.

## Training and scoring a tagger

```
python eval/train.py --smoke        # 300 examples, 1 epoch - does the plumbing work
python eval/train.py                # the real run, writes eval/model/
python eval/predict.py              # tag the gold set, report span F1
cargo run --bin bench -- --predictions eval/predictions.jsonl
```

Needs `torch` and `transformers`; the training loop is plain torch, no `datasets` or
`accelerate`. PRIORITY is split into `PRIORITY_P1..P4` so the 4-way class falls out of the BIO
tag itself — `"pretty important"` is tagged `B-PRIORITY_P2` — rather than needing a second head.

**Two numbers, measuring different things.** `predict.py` reports **span F1**: how well the model
*identifies*. `bench --predictions` reports **exact match**: the model plus Rust resolution, end
to end, directly comparable to the parser's 47.8%. Keep them apart — a date the tagger finds
correctly but Rust cannot resolve (`"tmrw"`) costs exact match without being a tagging failure,
and the fix for it belongs in the resolver, not the model.

`bench --predictions` takes the title **verbatim from its span** and resolves the other spans
separately, rather than round-tripping the whole sentence back through the parser. That mirrors
what a real pipeline does. Round-tripping was tried first and scored 4.5 points lower for a
spurious reason: fragments the parser could not fully consume (`"around 5"` → `"5"`) leaked back
into the title. That is an artifact of the simulation, not of tagging.

## The oracle: what perfect tagging is worth

Feeding the gold spans back in as predictions measures the ceiling — what a flawless tagger
would score against today's resolvers:

```
cargo run --bin bench -- --export-spans eval/oracle.jsonl
cargo run --bin bench -- --predictions eval/oracle.jsonl
```

```
                 parser    oracle
Title             50.7%     92.5%
Priority (rec.)   26.7%    100.0%
Time (recall)     58.8%     58.8%    <- unchanged
Exact match       47.8%     74.6%
```

Three things follow, and they shape where effort goes:

- **Tagging is worth ~27 points.** Title contamination and soft priority essentially disappear.
- **Time is not a tagging problem.** Perfect spans change nothing, because the resolver cannot
  turn `"around 3ish"` or `"after lunch"` into a time. That is deterministic Rust work, with no
  model involved, and it is the largest remaining slice.
- **74.6% is the ceiling, not 100%.** The 5 missing titles are the rewording cases — a tagger
  selects spans and can never produce `"fix the printer"` from `"need the printer fixed"`. All
  five are already `review`-flagged. If rewording is not a requirement, change those
  expectations and the ceiling rises.

Run the oracle again whenever the resolvers change. It moves the target the model is chasing.

## Resolver work (deterministic, no model)

`parse_date` and `parse_time` were extended to cover what the benchmark showed they could not
resolve. This is plain Rust - no training, no inference - and it moved every engine at once:

```
                    before    after
parser               42.5%    51.3%
v3 tagger            58.5%    67.0%
oracle               74.8%    85.6%

date recall (oracle) 69.3%    87.3%
time recall (oracle) 36.0%    74.0%
```

For the parser: **27 cases fixed, 0 broken** (McNemar exact, p = 1.49e-08). Strictly dominating.

Added: day-of-month ("on the 20th", next occurrence), month and week boundaries ("end of the
month", "beginning of october", and "before <month>" meaning the last day of the one before),
relative phrases ("the day after tomorrow", "a week today", "a fortnight from now", "in ten
days"), spoken clock times ("quarter past two", "ten to six"), spelled-out hours ("at seven"),
space-separated minutes ("at 9 30"), named points in the day ("first thing", "after lunch",
"before bed", "close of play"), and hedges in front of a time ("around 5", "dead on 2",
"no later than 5").

Bare hours 1-8 now read as afternoon and 9-12 as written, overridden by an explicit period
("three in the evening" is 15:00, "seven in the morning" is 07:00). That convention is a
product decision the benchmark forced into the open; it is covered by a test so changing it is
a deliberate act.

Deliberately **not** implemented: idioms with no settled meaning - "knocking off time", "before
the shops shut", "this arvo", "straight after the school run". Inventing a value for each would
be fitting the benchmark rather than the language, and those cases stay `!review` in gold.

Two regressions the benchmark caught during the work, both now tested:

- `"will take about 2 hours"` - the new "about" hedge stole the number before `parse_duration`
  saw it, turning a 2-hour estimate into 2pm. Guarded by refusing a hedge+number when a
  duration unit follows.
- `"a fortnight from now"` resolved to today, because `fortnight` was not in `FUZZY_VOCAB` and
  the typo corrector "fixed" it into `fortnightly`. A real word being corrected into a
  different real word is worth watching for elsewhere in that list.

## Results so far

MiniLM-L6 (22.6M params, full fine-tune, 20k synthetic examples, 3 epochs, ~20 min CPU),
measured on the 306-case gold set:

```
                    parser     v2      oracle
Title                44.8%    84.3%    97.4%
Priority (recall)    12.2%    69.4%   100.0%
Date (recall)        67.3%    68.0%    69.3%
Time (recall)        36.0%    36.0%    36.0%
Exact match          42.5%    58.2%    74.8%
Invented values          5       29        1
Gold span F1             -   0.7481        -
Latency                 ~0   2.0 ms        -
```

The tagger beats the parser decisively: **p = 1.33e-07** (McNemar exact, paired) — 66 cases
fixed against 18 broken.

### v3: a null result, and a misleading metric

v3 added "distractor" clauses - evaluative phrases carrying no priority ("nothing major",
"harder than it looks") - to fix the precision collapse. It did not work:

```
              v2        v3
exact match   58.2%     58.5%      (one case - noise)
span F1      0.7481    0.7536
invented         29        27
```

Two model iterations, no measurable gain. Worth recording as a dead end rather than repeating.

Inspecting the predictions showed why the diagnosis was wrong. The model is far better than
exact-boundary span F1 suggests; most "errors" are annotation convention, not understanding:

```
gold DATE 'tomorrow'   ·  pred DATE 'sometime tomorrow'     boundary convention
pred PRIORITY_P2 'it', PRIORITY_P2 's'                      sub-token fragments of "it's"
```

Scored with overlap matching and a 3-character minimum span:

```
exact boundaries           F1=0.754  P=0.692  R=0.827
overlap + fragment filter  F1=0.894  P=0.835  R=0.962
```

**Recall is 0.96.** The tagger finds nearly every span. About 14 points of the apparent F1
gap was measurement. Note that the fragment filter does *not* change end-to-end exact match
(58.5% either way), so those fragments are a metric artifact, not a product defect - which is
the reason to trust `bench` over span F1 when the two disagree.

### What growing gold from 67 to 306 revealed

The small set was flattering the model. Same weights, measured on both:

```
              67 cases    306 cases
exact match      64.2%        58.2%
span F1         0.8397       0.7481
invented             1           29
```

The regression is not noise — it is the 67-case set happening to sit closer to the training
distribution than real input does. **Precision, not recall, is the problem:**

```
                   P       R
PRIORITY_P4     0.227   0.385
PRIORITY_P1     0.279   0.600
NOTE            0.294   0.625
PRIORITY_P2     0.400   0.533
TIME            0.630   0.680
DATE            0.632   0.843
TITLE           0.829   0.902
```

The model over-labels. It finds most of what is there (recall is respectable) and tags a great
deal that is not. On 67 cases this looked solved; it was not.

The oracle barely moved (74.6% → 74.8%) across a 4.5x larger set, which is a good sign the
expanded gold is comparable in difficulty rather than simply harder.

## Extending to 300+

Current coverage is ~67 cases. The distribution matters more than the count — deliberately keep
roughly a third with no metadata at all, or anything trained on this will learn to label
something in every sentence.

When adding, write the sentence the way you would actually type it, including the typo. Cases
invented to be tidy are the ones that produce a model that works on tidy input.
