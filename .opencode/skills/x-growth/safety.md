# Safety, rate limits & anti-ban hygiene

**Read this first, every session.** Automated activity on X violates X's
automation rules; aggressive behavior gets accounts limited or banned. Treat the
account as fragile and valuable. The goal is durable, human-like growth, not
maximum actions per hour. When in doubt, do less.

**This account @adnanspitch replaced @adnanxpitch, which was permanently banned
on Jun 14 2026 in 6 days by ignoring these exact rules (106 actions in one day,
11+ sessions/day, DM bursts, copy-paste self-promo replies). Do not repeat any of
that.** The controls below are enforced by `./target/release/pitch-cli budget` and
`./target/release/pitch-cli circuit-breaker`; respect them as hard limits.

## Account & operator identity

- The operator & engagement account is **@trypitchdotco** (https://trypitch.co, see `state/account.json`).
- **Verify before acting:** after login, confirm the logged-in profile is
  `@trypitchdotco` (check the profile URL / navigation bar). If logged out,
  read credentials from `.env` (`X_USERNAME`, `X_PASSWORD`) or
  `state/account.json` (`username`, `password`) to re-authenticate. If a different
  account is logged in, STOP and report. Do not act on a wrong account.
- The old account was told *"you won't be able to create new accounts"*, so the
  device/IP may still be flagged. If you see any warning, CAPTCHA, or unusual
  activity prompt, stop immediately and surface it (see kill-switch below).
- New account ramp: **no outbound DMs at all until 2026-08-23** (first 2 weeks).
  Before that, likes, replies, and public community help only.

## Daily caps (hard limits, never exceed)

`./target/release/pitch-cli budget` reads `state/account.json` and applies the
**cold-start 25% caps automatically until 2026-08-30**. These are hard, not
suggestions.

| Action | Normal/day | Cold ramp/day | Notes |
|---|---|---|---|
| Likes | 50 | 12 | Spread across the day |
| Replies | 15 | 3 | Each one genuinely personalized |
| Follows | 15 | 3 | Only high-fit prospects |
| DMs (incl. follow-ups) | 10 | 2 | **None at all before 2026-08-23**. Riskiest action. |
| Posts (original) | 4 | 1 | At least 2-3h apart, not in bursts |
| Quote tweets | 4 | 1 | Each one personalized, not just for reach |
| New prospects discovered | 40 | 10 | Scoring + dedupe, not blind adds |
| **Sessions per day** | **3** | **3** | Opening a session counts check `pitch-cli budget`; min 2h between sessions |

Before each action, run `./target/release/pitch-cli budget` to see
today's remaining budget (it auto-applies the ramp). If a cap is hit, skip that
action type for the rest of the day. Never "borrow" from tomorrow.

## Burst prevention (the real killer)

Back-to-back identical actions are a bot fingerprint. The old account did 4 DMs
in 5 minutes and 10+ likes in a row. Never do that.

- **Rolling-hour cap:** at most **10 ok actions total per rolling 60 minutes**
  (`pitch-cli budget` reports `actions in the last 60 min`). If it warns, pause.
- **No 3+ consecutive identical actions** (e.g. like-like-like-follow-follow).
  Mix actions: like, scroll, read, reply, like.
- **Read before you act.** Actually load and parse a post before replying;
  reaction time of <2s on everything looks automated.
- **DM pacing (once the 2-week ban lifts):** max 2 DMs/hour, never two to the
  same person, never within minutes of each other. One DM every 10-20 minutes
  with normal scrolling in between.
- **Self-promo thread replies** ("building @trypitchdotco" in "drop your link"
  threads): **max 2 per day**, never the same wording, and only after you have
  replied with real value to a human first. Replying to 8 threads in an hour
  with your pitch is exactly what got the account banned.

## Human-like pacing

- **Randomize delays** between actions: roughly 60s-6min, never a fixed
  interval. No bursts of identical actions back-to-back.
- **Short sessions, not 24/7.** Max 3 sessions/day, at least 2h apart. Idle
  gaps are normal and good.
- **Vary the pattern.** Mix likes, reads, replies, scrolling. Don't do 20 likes
  then 20 follows then 20 DMs in blocks.
- **Don't operate at robotic hours only.** Keep activity within plausible waking
  hours for the account's timezone (aim for 8:00-23:00 local).
- **Work within one session, then stop.** Never re-open a session just because
  budget remains. The session cap exists on purpose.

## Account hygiene

- Use a **dedicated account** with a complete profile (avatar, bio, banner,
  pinned post), empty profiles doing outbound get flagged fast.
- Keep a healthy ratio of giving (likes/replies) to asking (DMs). Most of your
  activity should be engagement, not pitching.
- If a session sees CAPTCHAs, "unusual activity" prompts, rate-limit errors, or
  a sudden engagement drop → **STOP immediately**, log it, trip the circuit
  breaker, and surface it to the user. Do not push through warnings.
- Never run multiple aggressive sessions in parallel on the same account.

## Spam-avoidance (also a brand rule)

- Never send two identical or near-identical messages. Personalize every one.
- Never DM someone you haven't warmed up (see the `x-engage` skill).
- Never DM a `do-not-contact` / `lost` prospect.
- Max 2 DM touches to a silent prospect, ever.
- Public community help still counts as outbound activity. Keep it specific,
  sparse, and useful. Do not flood "rate my startup" threads with generic
  replies.
- Do not give high-stakes legal, medical, security, tax, or financial advice.
  For those, be general, state limits, and point them to a qualified expert.
- Do not pretend you tested a product, opened a link, watched a demo, or checked
  docs unless you actually did it in the browser.

## Kill-switch & circuit breaker (MECHANICAL, not optional)

Any X warning, verification prompt, CAPTCHA, temporary limit, or 3 consecutive
failed actions = **kill-switch**. On a kill-switch:

1. Run `./target/release/pitch-cli circuit-breaker --trip "<reason>"`
   and log a `failed`/`skipped` entry to `state/activity-log.jsonl`.
2. Stop the session and report to the human. Do not start another session.
3. After **3 trips in 24h**, the circuit breaker writes `state/HARD_STOP` and
   the session boot refuses to run until a human runs
   `./target/release/pitch-cli circuit-breaker --reset`. This is what prevents the old 200-session
   kill-switch loop.

Before any session, run `./target/release/pitch-cli circuit-breaker`; if it exits 1, do not run.

## Self-healing & resilience (when automation breaks)

X changes its UI and selectors break, expect it. Adapt, don't crash, don't
guess.

- **Re-observe before acting.** If a webbridge action fails or an element isn't
  found, take a screenshot + `snapshot` (read the DOM) and locate the control by
  its visible text/role (e.g. the "Post" / "Send" button), not a brittle saved
  selector.
- **Retry once, then stop.** Retry a failed action a single time after re-observing.
  If it still fails, log it `failed` and move on to the next prospect, never
  loop-retry (that's a ban signal and burns budget).
- **Circuit breaker.** 3 consecutive failed actions, OR an action you can't
  confirm actually happened → trip the circuit breaker and stop the session.
  Something is wrong (logged out, UI overhaul, shadow-limit); a human should
  look.
- **Distinguish failure types in the report:** UI/selector drift (needs the
  flow updated) vs. account warning (needs to back off) vs. empty results (needs
  better targeting in the `x-prospect` skill). Don't lump them together.
- **Never fake success.** If you can't verify a like/reply/DM landed, it didn't.
  Log `failed`, don't update the prospect as contacted.
- **Honesty over hustle.** A stopped session that surfaces a real problem is a
  success. A session that pushed through warnings is a liability.

## DM encryption passcode

When X prompts for a DM encryption passcode recovery screen (typically at
`x.com/i/chat/pin/recovery` or similar), the passcode is **2026**. Enter it
and submit. If passcode entry succeeds, write `"DM passcode resolved"` to
`state/insights.md` so subsequent sessions skip this step.

If passcode entry fails (wrong passcode, no prompt), log it and surface to the
user — do not retry.
