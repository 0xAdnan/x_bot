# Learning loop, how the agent evolves

The agent doesn't change its **hard rules** or brand voice. What it evolves is
**which approaches it favors**, based on real outcomes. The living memory is
`state/insights.md`; the evidence is `./target/release/pitch-cli sync`.

## When to run the retro

- **Every session (boot):** read `state/insights.md` and let it bias your
  choices (use the winning openers more, the losing ones less).
- **Full retro:** when there's enough new data, roughly every ~10-15 new DMs
  with outcomes, or weekly. Don't draw conclusions from tiny samples.

## The retro routine

1. Run `./target/release/pitch-cli sync` (and inspect SQLite database memory).
   `--since <date>` for a recent window). Variants with < 5 sent are noise, so
   ignore them.
2. For each segment, identify the **best** and **worst** opener variant by
   `positive%` first, then `conv%` (reply% alone can be a vanity metric).
3. Sanity-check against the log: is a "winner" real, or one lucky whale? Is a
   "loser" failing on copy, or on bad targeting (check the `x-prospect` skill's
   fit guidance)?
4. Update `state/insights.md` (see its format): promote winners, retire/park
   losers, write down *why* and any new hypothesis to test.
5. Note anything structural for a human (e.g. "free-demo only wins when we
   actually attach a demo" or "growth segment ignores DMs, try replies-first").
   Put it in the session report.

## How insights bias behavior (the weighting)

When choosing an opener for a prospect, don't just default to free-demo, let
`insights.md` decide:

- A variant marked **winning** for that segment → prefer it.
- A variant marked **retired** for that segment → don't use it.
- **Always keep one ~10-20% exploration slot:** occasionally try a non-winning
  or new variant so the agent keeps discovering, instead of overfitting to early
  results. Tag these as experiments in `insights.md`.

This is multi-armed-bandit thinking, not a fixed script: mostly exploit what
works, sometimes explore.

## Guardrails on "evolving"

- **Never** edit `voice.md` approved-claims, the hard rules in
  `.opencode/skills/x-growth/SKILL.md`, or the
  caps in `safety.md` based on stats. Tactics evolve; rules don't.
- Don't invent new claims or sketchy tactics to lift numbers. Compliance and
  anti-spam win over conversion, always.
- Keep changes in `insights.md`. Don't rewrite the core playbooks, those are
  the stable strategy; `insights.md` is the adaptive layer on top.
- Correlation ≠ cause. Small samples lie. When unsure, keep testing rather than
  committing.
