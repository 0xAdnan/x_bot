# PITCH, X/Twitter Growth Agent (OpenCode)

An autonomous OpenCode agent that runs the full X/Twitter growth-to-sales funnel
for **PITCH** (https://trypitch.co), the AI video editor that turns a task
description into a studio-quality narrated demo MP4.

It discovers ICP prospects, warms them with likes & genuine replies, then DMs to
convert. It also develops a practical tech-founder personality, comments on
broader startup/AI/devtool topics, and helps builders publicly so @trypitchdotco
can build a real community around the product. It drives a real logged-in X
session via a browser MCP, fully autonomously, inside hard safety limits.

## Layout

Hybrid design: a thin **agent** (the persona) that runs a discoverable **skill**
(the playbooks). The agent loads on its own; the skill is surfaced to any agent
by its `description` and loaded on demand.

```
.opencode/
  agent/x-growth.md             thin persona, defers to the skill
  skills/x-growth/
    SKILL.md                    discoverable entry: session loop, CRM, hard rules
    prospecting.md              ICP, search queries, qualify/disqualify, scoring
    engagement.md               like/reply warm-up cadence + reply recipes
    community.md                founder personality, help-first community loops
    outreach.md                 DM sequences, the "free demo" play, objections
    dm-templates.md             per-segment opening-DM bank (scaffolds)
    learn.md                    learning loop, how it evolves its approach
    voice.md                    brand voice, approved claims, compliance
    safety.md                   daily caps, pacing, anti-ban, self-healing
    scripts/budget.sh           today's remaining action budget from the log
    scripts/stats.sh            reply/conversion rates per segment & variant
    scripts/runner.sh           always-on orchestrator (jittered bounded sessions)
opencode.jsonc                  browser MCP wiring (Playwright)
state/
  prospects.jsonl               the CRM pipeline
  activity-log.jsonl            every action + outcome (rate-limiting + learning)
  insights.md                   adaptive memory, what's working (self-edited)
  README.md                     state schema
```

## Setup

1. **Install OpenCode** and a browser MCP. This config uses Playwright MCP via
   `npx @playwright/mcp@latest` (auto-downloaded on first run; needs Node).
2. **Use a dedicated X account** with a complete profile. Don't point this at
   your personal/main account.
3. **Persist login** (recommended): in `opencode.jsonc`, swap `--isolated` for
   `--user-data-dir=./.browser-profile`, run once, log into X manually in that
   browser, then close. Future sessions reuse the cookies.
4. *(Optional)* set the agent's `model` in `.opencode/agent/x-growth.md`
   frontmatter to your preferred provider/model.

## Run

```bash
cd /home/adnan/Documents/x_bot
opencode
```

Then select/invoke the **x-growth** agent and tell it to run a session, e.g.:

> Run a prospecting + engagement session for the founder and growth segments.

Or just: **"run a session"**, it boots safety limits, pulls the pipeline,
scans community conversations, discovers, warms, converts, follows up, logs
everything, and reports back.

### Always-on (the safe way)

A session is **bounded**, it ends when caps are hit or the kill-switch trips.
For "always running", use the orchestrator, which keeps the *process* alive but
keeps the *X activity* human-like (jittered gaps, waking hours only, backs off
on caps):

```bash
bash .opencode/skills/x-growth/scripts/runner.sh &   # start
tail -f state/runner.log                             # watch
touch state/PAUSE                                     # pause (stays up; rm to resume)
touch state/STOP                                      # stop gracefully
kill "$(cat state/runner.pid)"                        # stop now
```

> ⚠️ Do **not** make it run actions continuously/24-7, that's the fastest way to
> get banned. The runner deliberately sleeps, throttles, and restricts to waking
> hours. Tune with env vars (`WAKE_START`, `WAKE_END`, `MIN_GAP`, `MAX_GAP`).
> Dry-run the schedule without touching X: `DRY=1 RUN_ONCE=1 bash …/runner.sh`.

## How it works (the loop)

1. Boot → load the `x-growth` skill, read `safety.md`, run
   `scripts/budget.sh` to check today's remaining caps.
2. Pull pipeline from `state/prospects.jsonl`; advance warm leads first.
3. Discover new qualified prospects (scored + deduped).
4. Scan broader tech/founder conversations and answer useful community asks.
5. Warm prospects with likes + genuine replies (no pitching).
6. DM only once the warm-up bar is met, lead with value (offer a free
   @trypitchdotco demo of *their* product), never a cold pitch.
7. Follow up within limits; honor opt-outs instantly.
8. Log every action; end with a session report.

## Guardrails baked in

- **Hard daily caps** (likes/replies/follows/DMs) + randomized human-like pacing.
- **No spam**: every message personalized; no identical blasts; warm-up required
  before any DM.
- **Instant opt-out**: "no"/"stop" → `do-not-contact`, forever.
- **Approved claims only**: no invented features, pricing, or metrics.
- **Kill-switch**: stops on any CAPTCHA / rate-limit / X warning and reports.

## Learns & heals over time

- **Evolving (safe):** every DM is tagged with its opener variant; replies and
  conversions are logged as outcomes. `scripts/stats.sh` shows reply/conversion
  rates per segment & variant, and the agent updates `state/insights.md` to favor
  what's winning and retire what's losing, keeping a ~10-20% exploration slot so
  it keeps discovering. **Tactics evolve; hard rules, brand voice, and rate caps
  never do.** See [learn.md](.opencode/skills/x-growth/learn.md).
- **Self-healing:** when X's UI shifts and a click/selector fails, the agent
  re-observes the page (screenshot + DOM) and finds the control by visible
  text/role, retries once, and trips a circuit breaker (stop + report) after 3
  consecutive failures or any unverifiable action, rather than crashing or
  faking success. See `safety.md`.

> ⚠️ Automating X violates X's automation rules and can get accounts limited or
> banned. Run on a dedicated account, start at ~25% of the caps for new
> accounts, and ramp slowly. The agent optimizes for a durable account, not max
> actions/hour.
