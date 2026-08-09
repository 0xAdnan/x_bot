#!/usr/bin/env bash
# stats.sh — outcome analytics per segment & opener variant, from the activity log.
#
# Powers the learning loop: shows which approaches actually get replies and
# conversions so the agent can weight its choices (see learn.md / insights.md).
#
# Attribution model:
#   - Each "dm"/"followup" action is logged with "segment" and "variant".
#   - When a reply/conversion is observed, the agent logs an "outcome" action
#     with the same "segment"+"variant" and a "detail" of:
#       ignored | replied | positive | declined | trial | customer
#
# Rates (per segment|variant):
#   sent      = # of dm/followup
#   replied   = outcomes in {replied,positive,declined,trial,customer}
#   positive  = outcomes in {positive,trial,customer}
#   converted = outcomes in {trial,customer}
#
# Usage:
#   bash .opencode/skills/x-growth/scripts/stats.sh                 # all-time
#   bash .opencode/skills/x-growth/scripts/stats.sh --since 2026-06-01
#   bash .opencode/skills/x-growth/scripts/stats.sh --json
#   bash .opencode/skills/x-growth/scripts/stats.sh --min 5         # hide variants with <5 sent
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../" && pwd)"          # scripts/ -> skill root (contains state/)
LOG="$ROOT/state/activity-log.jsonl"

SINCE=""; JSON=0; MIN=1
while [ $# -gt 0 ]; do
  case "$1" in
    --since) SINCE="$2"; shift 2 ;;
    --json) JSON=1; shift ;;
    --min) MIN="$2"; shift 2 ;;
    *) shift ;;
  esac
done

[ -f "$LOG" ] || { echo "No log at $LOG yet — no data to analyze." >&2; exit 0; }
command -v python3 >/dev/null 2>&1 || { echo "stats.sh needs python3." >&2; exit 1; }

SINCE="$SINCE" JSON="$JSON" MIN="$MIN" python3 - "$LOG" <<'PY'
import json, os, sys
from collections import defaultdict

log = sys.argv[1]
since = os.environ.get("SINCE", "")
as_json = os.environ["JSON"] == "1"
min_sent = int(os.environ["MIN"])

REPLIED = {"replied", "positive", "declined", "trial", "customer"}
POSITIVE = {"positive", "trial", "customer"}
CONVERTED = {"trial", "customer"}

# key = (segment, variant) -> counters
sent = defaultdict(int)
out = defaultdict(lambda: defaultdict(int))  # key -> detail -> n

with open(log) as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        day = str(e.get("ts", "")).split("T")[0]
        if since and day < since:
            continue
        seg = e.get("segment", "?")
        var = e.get("variant", "?")
        key = (seg, var)
        a = e.get("action")
        if a in ("dm", "followup") and e.get("result") == "ok":
            sent[key] += 1
        elif a == "outcome":
            out[key][e.get("detail", "")] += 1

def rates(key):
    s = sent[key]
    o = out[key]
    rep = sum(o[d] for d in REPLIED)
    pos = sum(o[d] for d in POSITIVE)
    conv = sum(o[d] for d in CONVERTED)
    pct = lambda n: (100.0 * n / s) if s else 0.0
    return s, rep, pos, conv, pct(rep), pct(pos), pct(conv)

keys = sorted(set(sent) | set(out), key=lambda k: (-sent[k], k))

if as_json:
    data = []
    for k in keys:
        s, rep, pos, conv, rr, pr, cr = rates(k)
        if s < min_sent:
            continue
        data.append({"segment": k[0], "variant": k[1], "sent": s,
                     "replied": rep, "positive": pos, "converted": conv,
                     "reply_rate": round(rr, 1), "positive_rate": round(pr, 1),
                     "conversion_rate": round(cr, 1)})
    print(json.dumps({"since": since or "all-time", "rows": data}, indent=2))
    sys.exit(0)

print(f"Outcome stats ({since or 'all-time'})  [variants with >= {min_sent} sent]")
print(f"{'segment':<10}{'variant':<16}{'sent':>5}{'rep':>5}{'pos':>5}{'conv':>5}"
      f"{'rep%':>7}{'pos%':>7}{'conv%':>7}")
print("-" * 73)
shown = 0
for k in keys:
    s, rep, pos, conv, rr, pr, cr = rates(k)
    if s < min_sent:
        continue
    shown += 1
    print(f"{k[0]:<10}{k[1]:<16}{s:>5}{rep:>5}{pos:>5}{conv:>5}"
          f"{rr:>6.0f}%{pr:>6.0f}%{cr:>6.0f}%")
if not shown:
    print("(not enough data yet — keep running sessions)")
PY
