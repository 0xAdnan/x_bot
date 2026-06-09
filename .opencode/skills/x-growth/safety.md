# Safety, rate limits & anti-ban hygiene

**Read this first, every session.** Automated activity on X violates X's
automation rules; aggressive behavior gets accounts limited or banned. Treat the
account as fragile and valuable. The goal is durable, human-like growth, not
maximum actions per hour. When in doubt, do less.

## Daily caps (hard limits, never exceed)

These are conservative starting points for a warm, established account. For a
**new/cold account, use ~25% of these for the first 2-3 weeks** and ramp slowly.

| Action | Per day | Notes |
|---|---|---|---|
| Likes | 50 | Spread across the day |
| Replies | 15 | Each one genuinely personalized |
| Follows | 15 | Only high-fit prospects |
| DMs (incl. follow-ups) | 10 | The riskiest action, stay well under |
| Posts (original) | 4 | At least 2-3h apart, not in bursts |
| Quote tweets | 4 | Each one personalized, not just for reach |
| New prospects discovered | 40 | Scoring + dedupe, not blind adds |

Before each action, check today's remaining budget by running
`bash .opencode/skills/x-growth/scripts/budget.sh` (it tallies today's `ok`
actions from `state/activity-log.jsonl`). If a cap is hit, skip that action type
for the rest of the day. For a new/cold account, run with overrides, e.g.
`CAP_DM=3 CAP_FOLLOW=4 CAP_REPLY=4 CAP_LIKE=12 bash ...budget.sh`.

## Human-like pacing

- **Randomize delays** between actions: roughly 30s-4min, never a fixed
  interval. No bursts of identical actions back-to-back.
- **Work in short sessions**, not 24/7. A few spread-out sessions/day beats one
  marathon. Idle gaps are normal and good.
- **Vary the pattern.** Mix likes, reads, replies, scrolling. Don't do 20 likes
  then 20 follows then 20 DMs in blocks, that's a bot fingerprint.
- **Read before you act.** Actually load and parse a post before replying to it;
  reaction time of <2s on everything looks automated.
- **Don't operate at robotic hours only.** Keep activity within plausible waking
  hours for the account's timezone.

## Account hygiene

- Use a **dedicated account** with a complete profile (avatar, bio, banner,
  pinned post), empty profiles doing outbound get flagged fast.
- Keep a healthy ratio of giving (likes/replies) to asking (DMs). Most of your
  activity should be engagement, not pitching.
- If a session sees CAPTCHAs, "unusual activity" prompts, rate-limit errors, or
  a sudden engagement drop → **STOP immediately**, log it, and surface it to the
  user. Do not push through warnings.
- Never run multiple aggressive sessions in parallel on the same account.

## Spam-avoidance (also a brand rule)

- Never send two identical or near-identical messages. Personalize every one.
- Never DM someone you haven't warmed up ([engagement.md](engagement.md)).
- Never DM a `do-not-contact` / `lost` prospect.
- Max 2 DM touches to a silent prospect, ever.
- Public community help still counts as outbound activity. Keep it specific,
  sparse, and useful. Do not flood "rate my startup" threads with generic
  replies.
- Do not give high-stakes legal, medical, security, tax, or financial advice.
  For those, be general, state limits, and point them to a qualified expert.
- Do not pretend you tested a product, opened a link, watched a demo, or checked
  docs unless you actually did it in the browser.

## Kill-switch conditions, stop the session and report

- Any X warning, verification prompt, CAPTCHA, or temporary limit.
- A daily cap reached early (means pacing is off).
- A prospect reports/blocks you, or you get multiple negative reactions.
- Login/session looks broken or you can't confirm an action actually happened.

Log the reason to `state/activity-log.jsonl` as a `failed`/`skipped` entry and
include it in the session report.

## Self-healing & resilience (when automation breaks)

X changes its UI and selectors break, expect it. Adapt, don't crash, don't
guess.

- **Re-observe before acting.** If a click/type fails or an element isn't found,
  take a screenshot + read the DOM and locate the control by its visible
  text/role (e.g. the "Post" / "Send" button), not a brittle saved selector.
- **Retry once, then stop.** Retry a failed action a single time after re-observing.
  If it still fails, log it `failed` and move on to the next prospect, never
  loop-retry (that's a ban signal and burns budget).
- **Circuit breaker.** 3 consecutive failed actions, OR an action you can't
  confirm actually happened → stop the session and report. Something is wrong
  (logged out, UI overhaul, shadow-limit); a human should look.
- **Distinguish failure types in the report:** UI/selector drift (needs the
  flow updated) vs. account warning (needs to back off) vs. empty results (needs
  better targeting in `prospecting.md`). Don't lump them together.
- **Never fake success.** If you can't verify a like/reply/DM landed, it didn't.
  Log `failed`, don't update the prospect as contacted.
- **Honesty over hustle.** A stopped session that surfaces a real problem is a
  success. A session that pushed through warnings is a liability.
