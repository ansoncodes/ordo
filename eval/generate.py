#!/usr/bin/env python3
"""Synthetic training data for Ordo's span tagger.

Builds sentences from labelled components, so the character spans are correct by
construction rather than by annotation. Every example is verified before it is written:
if `text[start:end]` ever disagrees with the recorded surface, the run aborts.

Weighted at the gaps the parser baseline actually showed (cargo run --bin bench):

    Title        50.7%   conversational filler stays in the title
    Priority     26.7%   recall - soft priority language unrecognised
    Time         58.8%   recall - fuzzy and spelled-out times unrecognised

Everything else the parser already handles at 91-100%, so those patterns appear only
often enough to stop the tagger regressing on them.

    python eval/generate.py --count 20000 --out eval/synthetic.jsonl
    python eval/generate.py --count 12 --preview
    python eval/generate.py --count 5000 --stats

The output is TRAINING data. It never touches eval/gold.json, which stays hand-written
and is the only thing you measure against.
"""

import argparse
import json
import random
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
GOLD = HERE / "gold.json"

# ---------------------------------------------------------------------------
# Mixture weights - tuned to the baseline above, not to intuition.
# ---------------------------------------------------------------------------

W = {
    "no_metadata": 0.28,      # hard negatives: a title and nothing else
    "trap": 0.14,             # month / weekday / project / tag words as ordinary words
    "filler_prefix": 0.45,    # "don't forget to", "remind me to"
    "filler_suffix": 0.14,    # "when i get time"
    "clause_suffix": 0.16,    # "once the client confirms" - not metadata, not title
    "distractor": 0.30,       # evaluative but NOT priority - teaches the discrimination
    "date": 0.55,
    "time": 0.38,
    "priority": 0.40,
    "duration": 0.18,
    "recurrence": 0.16,
    "project": 0.14,
    "tag": 0.10,
    "note": 0.08,
    "steps": 0.06,
    "scheduled": 0.04,
    "date_first": 0.07,       # "tomorrow: call Arun"
    "comma_before_priority": 0.55,
    "typo": 0.22,             # extra noise on top of the misspelled surfaces below
}

# Within a slot, how often to pick from the hard half of its surface list.
HARD_SHARE = {"priority": 0.75, "time": 0.65, "date": 0.45}

# ---------------------------------------------------------------------------
# Components. Each entry is (surfaces, canonical form for the Rust resolver).
# Surfaces are split (easy, hard): "hard" means the phrasing the parser currently
# misses - conversational, fuzzy or abbreviated.
# ---------------------------------------------------------------------------

TITLES = [
    "call Arun", "call mom", "call the bank", "call the plumber", "email the landlord",
    "send the invoice to acme", "send the report to maya", "reply to sir",
    "reply to the recruiter", "book dentist", "book the venue", "book flights",
    "fix the printer", "fix the login bug", "review the PR", "review the deck",
    "water the plants", "pay rent", "pay the electricity bill", "check the server",
    "back up the database", "renew the car insurance", "renew the domain",
    "order the tiles", "clean the garage", "write the summary", "draft the roadmap",
    "prep the talk", "finish the slides", "ship v2", "deploy the hotfix", "file taxes",
    "submit the form", "chase the council about the permit", "grab stamps", "buy milk",
    "pick up the parcel", "sort out the docs", "update the changelog",
    "merge the release branch", "rotate the API keys", "cancel the subscription",
    "return the headphones", "top up the oyster card", "defrost the freezer",
    "sign the lease", "post the birthday card", "refill the prescription",
    "take the car in for a service", "reconcile last month's expenses",
    "follow up with the supplier", "send Sarah the presentation", "ping the design team",
    "close the open tickets", "write the retro notes", "confirm the venue booking",
    "get a quote for the roof", "swap the smoke alarm batteries", "tidy the spare room",
]

FILLER_PREFIX = [
    "don't forget to", "dont forget to", "remind me to", "make sure I", "make sure to",
    "i need to", "need to", "gotta", "i have to", "have to", "can you remind me to",
    "hey can you remind me to", "hey don't let me forget to", "don't let me forget to",
    "please remember to", "remember to", "i should", "i should probably", "note to self:",
    "todo:", "i must", "we need to", "someone needs to", "i keep forgetting to",
    "really need to", "must remember to",
]

FILLER_SUFFIX = [
    "when i get time", "when you can", "at some point", "if possible", "if i get a chance",
    "sometime", "whenever", "when i can", "at some stage", "if there's time",
]

DATES = [
    ((["today"], ["2day", "todya"]), "today"),
    ((["tomorrow"], ["tmrw", "tmw", "2moro", "2morrow", "tommorow", "tommorrow", "tomorow", "tmmrw"]), "tomorrow"),
    ((["monday"], ["mon", "mondy"]), "monday"),
    ((["tuesday"], ["tue", "tues"]), "tuesday"),
    ((["wednesday"], ["wed", "wensday", "wendsday"]), "wednesday"),
    ((["thursday"], ["thu", "thurs", "thrusday"]), "thursday"),
    ((["friday"], ["fri", "fridy"]), "friday"),
    ((["saturday"], ["sat"]), "saturday"),
    ((["sunday"], ["sun"]), "sunday"),
    ((["next monday"], ["nxt monday", "next mon"]), "next monday"),
    ((["next friday"], ["nxt friday", "next fri"]), "next friday"),
    ((["next week"], ["nxt week", "sometime next week"]), "next week"),
    ((["next month"], ["nxt month", "sometime next month"]), "next month"),
    ((["in 3 days"], ["in three days", "in a couple of days"]), "in 3 days"),
    ((["in 2 weeks"], ["in two weeks", "in a fortnight"]), "in 2 weeks"),
    ((["this weekend", "the weekend"], ["over the weekend", "at the weekend"]), "weekend"),
    ((["tonight"], ["tonite", "later tonight"]), "tonight"),
    ((["eod"], ["end of day", "by end of day", "before the day is out"]), "eod"),
    ((["eow"], ["end of the week", "by the end of the week"]), "eow"),
    ((["eom"], ["end of the month", "by the end of the month"]), "eom"),
    ((["on the 20th", "on the 3rd"], ["the 20th", "the 15th"]), "day-of-month"),
    ((["sep 15", "oct 1"], ["september 15", "15 sep", "1st of october"]), "explicit-date"),
    ((["2026-10-01"], []), "2026-10-01"),
]

DATE_CONNECTORS = ["", "", "", "by", "on", "before", "due"]
LEADING_PREPOSITIONS = {"in", "on", "by", "over", "at", "before", "sometime", "the", "this", "later", "end"}

TIMES = [
    ((["3pm", "at 3pm", "15:00"], ["at 3", "around 3", "arnd 3", "around 3ish", "3ish", "at about 3", "3 in the afternoon", "around three"]), "15:00"),
    ((["9:30", "at 9:30", "09:30"], ["9 30", "at 9 30", "half nine", "9.30", "around 9:30", "9ish"]), "09:30"),
    ((["noon", "at noon"], ["midday", "around midday", "lunchtime"]), "12:00"),
    ((["5pm", "at 5pm", "17:00"], ["at 5", "around 5", "5 in the evening", "about 5"]), "17:00"),
    ((["6am", "at 6am"], ["6 in the morning", "first thing", "first thing in the morning", "early"]), "06:00"),
    ((["8pm", "at 8pm"], ["at 8", "around 8", "8 in the evening"]), "20:00"),
    ((["morning"], ["in the morning", "sometime in the morning", "am"]), "morning"),
    ((["afternoon"], ["in the afternoon", "after lunch", "post lunch", "sometime in the afternoon"]), "afternoon"),
    ((["evening"], ["in the evening", "this evening", "after work", "later on"]), "evening"),
]

PRIORITIES = [
    ((["!p1", "!critical", "!!!"], ["urgent", "super urgent", "really urgent", "very urgent",
      "this is critical", "critical", "asap", "as soon as possible", "top priority",
      "highest priority", "drop everything", "this one's urgent", "needs doing now",
      "make it urgent", "high alert"]), "p1"),
    ((["!p2", "!high", "!!"], ["high priority", "high prio", "important", "pretty important",
      "fairly important", "quite important", "make it important", "make it a priority",
      "kind of important", "reasonably important", "this matters", "bit important",
      "fairly high priority", "mark it important"]), "p2"),
    ((["!p3"], ["normal priority", "medium priority", "normal prio", "middling"]), "p3"),
    ((["!p4", "!low"], ["low priority", "low prio", "not important", "not a big deal",
      "no rush", "someday", "nice to have", "not urgent", "when convenient",
      "no hurry", "doesn't matter much", "low prio honestly", "back burner"]), "p4"),
]

DURATIONS = [
    ((["~5m"], ["5 min", "5 mins", "takes 5 min", "about 5 minutes"]), 5),
    ((["~15m"], ["15 mins", "15 minutes", "quarter of an hour", "takes 15 min"]), 15),
    ((["~30m"], ["30 mins", "half an hour", "about half an hour", "30 minutes"]), 30),
    ((["~45m"], ["45 mins", "45 minutes", "about 45 mins", "takes 45 min"]), 45),
    ((["~1h", "1h"], ["1 hour", "an hour", "about an hour", "takes an hour", "probably an hour"]), 60),
    ((["~90m", "1h30m"], ["an hour and a half", "90 mins"]), 90),
    ((["~2h"], ["2 hours", "2 hrs", "about 2 hours", "will take 2 hours", "couple of hours"]), 120),
]

RECURRENCES = [
    ((["every day", "daily"], ["evry day", "each day"]), "every day"),
    ((["every weekday"], ["on weekdays", "evry weekday", "each weekday"]), "every weekday"),
    ((["every monday"], ["on mondays", "evry monday", "each monday"]), "every monday"),
    ((["every friday"], ["on fridays", "each friday"]), "every friday"),
    ((["every 2 weeks"], ["every other week", "fortnightly", "every two weeks"]), "every 2 weeks"),
    ((["every 3 days"], ["evry 3 days", "every three days"]), "every 3 days"),
    ((["monthly", "every month"], ["evry month", "each month", "once a month"]), "every month"),
    ((["yearly", "every year"], ["annually", "once a year"]), "every year"),
]

PROJECTS = ["Hacking Challenge", "Workshop", "Personal", "SMEC Technologies", "Home Renovation"]
TAGS = ["finance", "admin", "health", "errands"]

NOTES = [
    "ask for the PO number", "remember eggs", "blocked on design review",
    "ask about the projector", "check the old thread first", "bring the receipts",
    "they close at 5", "use the corporate card", "cc the finance team",
]

STEPS = [
    ["outline", "draft", "send"], ["outline", "design"], ["research", "write", "edit"],
    ["book", "confirm"], ["pack", "post"], ["call", "email"],
]

# Words that ARE real project names, tag names, months or weekdays, used here as ordinary
# English. The tagger over-labelled every one of these until they were generated: "deal with
# that invoice once finance approves it" came back with finance tagged @finance.
AMBIGUOUS = [
    "personal", "workshop", "finance", "admin", "health", "errands",
    "march", "may", "august", "april", "june", "sun", "mon", "wed", "fri", "sat",
]

# Templates, not sentences: the pattern needs coverage even when a specific sentence using it
# is held out in gold.json, and a fixed list cannot survive deduplication at volume.
TRAP_TEMPLATES = [
    "{w} email", "sort the {w} stuff", "file the {w} paperwork", "the {w} folder needs tidying",
    "move the {w} notes into the wiki", "print the {w} report", "archive the {w} threads",
    "deal with that invoice once {w} approves it", "check with {w} before sending",
    "ask {w} about the quote", "{w} said they would call back", "reply to {w} about the deposit",
    "chase {w} for the paperwork", "the {w} team needs an answer", "update the {w} spreadsheet",
    "forward the {w} thread", "book a room for the {w} review", "tidy up the {w} inbox",
]

# Evaluative-sounding trailing phrases that are NOT priority. The 306-case gold set showed
# PRIORITY precision collapsing to 0.23-0.40: the model had learned "trailing opinion clause
# => priority" because every evaluative phrase it ever saw in training was one. These teach
# the discrimination. They also fix NOTE precision, which failed the same way.
DISTRACTOR = [
    "which is fine", "no problem", "should be easy", "it's a big job", "that's the tricky bit",
    "shouldn't take long", "nothing complicated", "bit of a faff", "worth doing properly",
    "same as last time", "could be messy", "straightforward enough", "harder than it looks",
    "nothing major", "easier said than done", "not as bad as it sounds", "a bit fiddly",
    "should be fine", "nothing to it", "more effort than expected",
]

# Trailing clauses that are not metadata and not part of the title. Unlabelled, like the
# shorter fillers - they carry the words that were provoking phantom priorities.
CLAUSE_SUFFIX = [
    "budget is tight so check a few sites first", "they said they'd get back to us",
    "once the client confirms", "after finance approves it", "if the parts arrive in time",
    "assuming the weather holds", "unless something changes", "if they reply in time",
    "once the invoice clears", "provided the quote holds", "as soon as they confirm",
    "while the team is still around", "if nothing else comes up",
]

# Sentences whose month / weekday / number words must NOT be read as metadata.
TRAPS = [
    "buy sun cream", "march the band down the street", "call mum about the may wedding",
    "book a table for four", "personal email", "read https://example.com/docs",
    "check the march figures", "order 2 packs of paper", "call 3 suppliers",
    "find a may bank holiday deal", "wednesday addams costume", "sunday roast recipe",
    "sort the april invoices", "the june accounts need signing off",
    "watch the sun set from the pier", "buy a fri sticker for the van",
    "email august about the quote", "pick up the may issue", "print 30 copies",
    "book 2 rooms", "may needs the file", "april from accounts called",
]

KEYBOARD = {
    "a": "qs", "b": "vn", "c": "xv", "d": "sf", "e": "wr", "f": "dg", "g": "fh",
    "h": "gj", "i": "uo", "j": "hk", "k": "jl", "l": "k", "m": "n", "n": "bm",
    "o": "ip", "p": "o", "q": "wa", "r": "et", "s": "ad", "t": "ry", "u": "yi",
    "v": "cb", "w": "qe", "x": "zc", "y": "tu", "z": "x",
}

# ---------------------------------------------------------------------------
# Generation
# ---------------------------------------------------------------------------


def pick(rng, entry, slot):
    """Choose a surface, favouring the hard variants at the slot's configured rate."""
    (easy, hard), canonical = entry
    share = HARD_SHARE.get(slot, 0.5)
    pool = hard if (hard and rng.random() < share) else easy
    return rng.choice(pool or easy), canonical


def typo(word, rng):
    if len(word) < 5 or not word.isalpha():
        return word
    i = rng.randrange(1, len(word) - 1)
    r = rng.random()
    if r < 0.30:
        return word[:i] + word[i + 1:]
    if r < 0.55:
        return word[:i] + word[i] + word[i:]
    if r < 0.80:
        return word[:i] + word[i + 1] + word[i] + word[i + 2:]
    sub = KEYBOARD.get(word[i])
    return word[:i] + rng.choice(sub) + word[i + 1:] if sub else word


def noisy(text, rng, rate):
    return " ".join(typo(w, rng) if rng.random() < rate else w for w in text.split())


def build(rng, mode):
    """Returns (segments, profile). A segment is (surface, label|None, canonical|None).

    `mode` is chosen by the caller rather than rolled here, so the mixture is a quota
    rather than an outcome - deduplication would otherwise starve exactly the simplest
    buckets, which are the hard negatives that matter most.
    """
    head, tail, profile = [], [], []
    bare = mode in ("bare", "trap")

    if mode == "trap":
        # Half fixed sentences, half composed from templates, so the pattern stays covered
        # at volume and survives deduplication against gold.
        if rng.random() < 0.5:
            title = rng.choice(TRAP_TEMPLATES).format(w=rng.choice(AMBIGUOUS))
        else:
            title = rng.choice(TRAPS)
        profile.append("trap")
    else:
        title = rng.choice(TITLES)
    # Hard negatives carry more typo noise: their only job is to teach the model that a
    # month-ish or number-ish word in an ordinary sentence is still just the title.
    if rng.random() < (W["typo"] * 2 if bare else W["typo"]):
        title = noisy(title, rng, 0.3)

    if rng.random() < W["filler_prefix"]:
        head.append((rng.choice(FILLER_PREFIX), None, None))
        profile.append("filler-prefix")

    date_seg = None
    if not bare and rng.random() < W["date"]:
        surface, canon = pick(rng, rng.choice(DATES), "date")
        # Surfaces that already carry their own preposition ("in 3 days", "over the
        # weekend", "by the end of the week") must not collect a second one.
        leads = surface.split()[0]
        conn = "" if leads in LEADING_PREPOSITIONS else rng.choice(DATE_CONNECTORS)
        date_seg = (f"{conn} {surface}".strip(), "DATE", canon)

    # "tomorrow: call Arun" - the date leads and the title follows.
    if date_seg and rng.random() < W["date_first"]:
        head.append((date_seg[0] + ":", "DATE", date_seg[2]))
        date_seg = None
        profile.append("leading-date")

    head.append((title, "TITLE", None))

    if date_seg:
        tail.append(date_seg)
    if not bare and rng.random() < W["time"]:
        surface, canon = pick(rng, rng.choice(TIMES), "time")
        tail.append((surface, "TIME", canon))
        profile.append("time")
    if not bare and rng.random() < W["recurrence"]:
        surface, canon = pick(rng, rng.choice(RECURRENCES), "recurrence")
        tail.append((surface, "RECURRENCE", canon))
    if not bare and rng.random() < W["duration"]:
        surface, canon = pick(rng, rng.choice(DURATIONS), "duration")
        tail.append((surface, "DURATION", canon))
    if not bare and rng.random() < W["project"]:
        name = rng.choice(PROJECTS)
        if rng.random() < 0.6:
            tail.append((f"#{name}", "PROJECT", name))
        else:
            tail.append((f"{rng.choice(['for', 'in'])} {name}", "PROJECT", name))
    if not bare and rng.random() < W["tag"]:
        t = rng.choice(TAGS)
        tail.append((f"@{t}", "TAG", t))
    if not bare and rng.random() < W["scheduled"]:
        surface, canon = pick(rng, rng.choice(DATES), "date")
        tail.append((f"^{surface}", "SCHEDULED", canon))

    if rng.random() < W["filler_suffix"]:
        tail.append((rng.choice(FILLER_SUFFIX), None, None))
        profile.append("filler-suffix")
    if rng.random() < W["clause_suffix"]:
        tail.append((rng.choice(CLAUSE_SUFFIX), None, None))
        profile.append("clause-suffix")
    if rng.random() < W["distractor"]:
        tail.append((rng.choice(DISTRACTOR), None, None))
        profile.append("distractor")

    if not bare and rng.random() < W["priority"]:
        surface, canon = pick(rng, rng.choice(PRIORITIES), "priority")
        if rng.random() < W["comma_before_priority"] and tail:
            last, label, c = tail[-1]
            tail[-1] = (last + ",", label, c)
        tail.append((surface, "PRIORITY", canon))
        profile.append("priority")

    if not bare and rng.random() < W["steps"]:
        for s in rng.choice(STEPS):
            tail.append((f"+{s}", "STEP", s))
    if not bare and rng.random() < W["note"]:
        note = rng.choice(NOTES)
        tail.append(("//", None, None))
        tail.append((note, "NOTE", note))

    if mode == "bare":
        profile.append("no-metadata")
    return head + tail, (profile or ["plain"])


def assemble(segments, case_id, today, profile):
    """Joins segments and records each label's character span."""
    text, entities = "", []
    for surface, label, canonical in segments:
        if not surface:
            continue
        if text:
            text += " "
        start = len(text)
        text += surface
        if label:
            # A trailing comma added for phrasing is punctuation, not part of the value.
            end = len(text) - 1 if surface.endswith(",") else len(text)
            ent = {"start": start, "end": end, "label": label, "text": text[start:end]}
            if canonical is not None:
                ent["canonical"] = canonical
            entities.append(ent)
    return {"id": case_id, "text": text, "today": today, "entities": entities, "profile": profile}


def verify(ex):
    """Every span must still name exactly the text it points at."""
    for e in ex["entities"]:
        actual = ex["text"][e["start"]:e["end"]]
        if actual != e["text"]:
            raise AssertionError(f"{ex['id']}: span {e['label']} says {e['text']!r} but text has {actual!r}")


def gold_texts():
    if not GOLD.exists():
        return set()
    with GOLD.open(encoding="utf-8") as f:
        return {c["text"].strip().lower() for c in json.load(f)["cases"]}


# Mondays through Sundays, so nothing downstream is locked to one weekday.
TODAYS = ["2026-09-14", "2026-09-15", "2026-09-16", "2026-09-17", "2026-09-18", "2026-09-19", "2026-09-20"]


def fill(rng, mode, target, seen, held):
    """Fills one bucket to `target`, or as close as its unique space allows."""
    out, attempts, cap = [], 0, max(target * 60, 5000)
    while len(out) < target and attempts < cap:
        attempts += 1
        today = rng.choice(TODAYS)
        segments, profile = build(rng, mode)
        ex = assemble(segments, "", today, profile)
        key = ex["text"].strip().lower()
        if key in seen or key in held:
            continue
        verify(ex)
        seen.add(key)
        out.append(ex)
    if len(out) < target:
        print(
            f"warning: bucket {mode!r} produced {len(out)} of {target} - its unique space is "
            f"exhausted. Add more entries to TITLES/TRAPS to widen it.",
            file=sys.stderr,
        )
    return out


def generate(count, seed):
    """Generates each bucket to an explicit quota so the mixture is what W says it is."""
    rng = random.Random(seed)
    held, seen = gold_texts(), set()
    quotas = {
        "bare": round(count * W["no_metadata"]),
        "trap": round(count * W["trap"]),
    }
    quotas["rich"] = count - quotas["bare"] - quotas["trap"]

    rows = []
    for mode in ("rich", "bare", "trap"):
        rows.extend(fill(rng, mode, quotas[mode], seen, held))
    rng.shuffle(rows)
    for i, r in enumerate(rows):
        r["id"] = f"syn-{i:06d}"
    return rows


def stats(rows):
    from collections import Counter
    labels, profiles = Counter(), Counter()
    bare = 0
    for r in rows:
        ls = {e["label"] for e in r["entities"]}
        labels.update(ls)
        profiles.update(r["profile"])
        if ls <= {"TITLE"}:
            bare += 1
    n = len(rows)
    print(f"\n  {n} examples\n")
    print("  Label            Share")
    print("  " + "-" * 30)
    for label, c in labels.most_common():
        print(f"  {label:<16} {c / n:6.1%}  ({c})")
    print("\n  Profile          Share")
    print("  " + "-" * 30)
    for p, c in profiles.most_common():
        print(f"  {p:<16} {c / n:6.1%}  ({c})")
    print(f"\n  Title-only (hard negatives): {bare / n:.1%}")
    avg = sum(len(r['text'].split()) for r in rows) / n
    print(f"  Mean length: {avg:.1f} words\n")


def main():
    ap = argparse.ArgumentParser(description="Synthetic training data for Ordo's span tagger.")
    ap.add_argument("--count", type=int, default=20000)
    ap.add_argument("--out", default="eval/synthetic.jsonl")
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--preview", action="store_true", help="print examples instead of writing")
    ap.add_argument("--stats", action="store_true", help="print the mixture and exit")
    args = ap.parse_args()

    if Path(args.out).resolve() == GOLD:
        sys.exit("refusing to write over the hand-written gold set")

    rows = generate(args.count, args.seed)

    if args.preview:
        for r in rows:
            print(f"\n{r['text']}")
            for e in r["entities"]:
                canon = f"  -> {e['canonical']}" if "canonical" in e else ""
                print(f"    {e['label']:<11} {e['text']!r}{canon}")
        return
    if args.stats:
        stats(rows)
        return

    path = Path(args.out)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"Wrote {len(rows)} examples to {path}")
    stats(rows)


if __name__ == "__main__":
    main()
