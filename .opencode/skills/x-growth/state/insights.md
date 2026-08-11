# Insights & adaptive memory

## Fresh start (Aug 9 2026) — new account @adnanspitch

Previous account @adnanxpitch was **permanently suspended on Jun 14 2026** for
automation-volume spam, 6 days after the account started. The entire old pipeline
(42 prospects) and activity log have been wiped. This is a brand new CRM.

## Why the old account was banned (read this, do not repeat)

1. **Young account, full-speed automation.** ~1 week old, yet the bot ran 106
   actions on Jun 13 and 152 on Jun 14 (caps were 50 likes / 15 replies / 15
   follows / 10 DMs). New accounts get no slack from X. Volume got it flagged.
2. **No session/day cap.** 11+ back-to-back sessions in a single day, all day
   long. Continuous activity is the strongest bot signal there is.
3. **DM bursts.** Multiple DMs sent minutes apart (4 in 5 minutes at one point).
   DMs are the riskiest action on X.
4. **Repeated identical self-promo replies.** "building @trypitchdotco..." pasted
   across many "drop your link" threads within minutes. Classic bot pattern.
5. **No mechanical circuit breaker.** The system logged 200+ consecutive
   "kill-switch" sessions and kept starting new ones instead of pausing.

## The guardrails now in place (see safety.md + scripts)

- `./target/release/pitch-cli budget` reads `account.json`: until `ramp_until`
  (2026-08-30) it applies **25% cold-start caps automatically**.
- **No outbound DMs before 2026-08-23** (first 2 weeks = likes/replies only).
- **Max 3 sessions/day**, min 2-3h between sessions (enforced manually by the
  account; you may run up to 3 sessions per day).
- **Per-hour burst caps**: no more than ~10 total actions/hour, max 2 DMs/hour
  (later), no back-to-back identical actions.
- **Circuit breaker**: after 3 consecutive kill-switch/failed sessions,
  `state/HARD_STOP` is created and automation pauses until a human clears it.
- **Account check**: before any action, confirm the logged-in profile is
  @adnanspitch (not the old account). If it isn't, stop and report.

## Current status

- Pipeline: empty (fresh).
- No actions logged yet for @adnanspitch.
- Budget: cold-start caps until 2026-08-30.

## Things to learn fresh on this account (do not copy old assumptions)

- The old funnel DID get real inbound (a prospect accepted a free demo, another
  replied positive). Value-first demo offers work. What killed the account was
  volume and pacing, not the message.
- Warm up the account manually (real browsing, likes, replies) for at least a
  week before automation ramps. If X flags the device/IP, stop immediately.
