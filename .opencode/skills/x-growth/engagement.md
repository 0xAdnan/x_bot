# Engagement, warming up before you ever pitch

Nobody buys from a stranger who slides into DMs cold. Earn recognition first.
Engagement is how a prospect goes from `new` → `warming` → ready-to-DM.

## The warm-up bar (gate before any DM)

A prospect is DM-ready only when **all** are true:
- You've liked **≥ 2** of their posts across **≥ 2 different days**, AND
- You've left **≥ 1 genuine, value-adding reply** they could have seen, AND
- It's been at least ~24h since your first interaction (no same-minute
  like→reply→DM bursts, that reads as a bot).

Until then, keep engaging. Track touches in the prospect's `touches` field.

## The engagement ladder (low to high commitment)

1. **Like** a recent, relevant post. Cheapest signal. Always start here.
2. **Follow** any qualified prospect you're genuinely warming (`score >= 4`).
   Following is a real relationship signal: it says "I'm interested in what you
   build", and it puts their posts in your feed so you can keep showing up. Do it
   early, once you've decided someone is a real prospect, not as a mass-follow.
3. **Reply** with something genuinely useful (see recipes). This is the move
   that gets you noticed.
4. **Quote tweet** when it adds value (see `content.md` for full playbook). A
   thoughtful quote tweet on a prospect's post is stronger than a plain reply —
   it puts @trypitchdotco in their mentions and shows up in their quote-tweet
   feed. Use for top prospects or when you have a genuinely useful take.
   Original posting and product posting are handled separately in `content.md`.

### Community help mode

For `community` prospects and public help asks, the goal is trust first, not DM
conversion. Answer the question in public when possible.

- Give one concrete suggestion based on what you actually saw.
- No product mention unless demo/video/onboarding/launch content is directly
  relevant.
- If they ask a follow-up, keep helping. If they ask for private review or a
  demo, then move to DM with disclosure that you work on @trypitchdotco.
- Add useful builders to the pipeline even if they are not buyer-ready. Set
  `segment` to `community`, use `why` to note the ask, and schedule a light
  follow-up engagement in 2-4 days.

### Inbound replies first

If someone has replied to you, quote-tweeted you, followed up in a thread, or
sent a DM, prioritize answering them before starting a fresh pitch or outbound
message. The next touch should feel like a real continuation, not a queued
campaign step.

- Address their actual message first.
- Match their tone and level of interest.
- If they asked a question, answer it before mentioning @trypitchdotco.
- If they shared context, react to that context before asking for anything.

### Following the right way (genuine, not growth-hacky)

- **Follow people you actually want a relationship with.** A qualified prospect,
  a builder whose work is relevant, someone in the @trypitchdotco orbit. Not
  random accounts to pad numbers.
- **No follow/unfollow churn, ever.** Following then unfollowing to game people
  is spammy, against the spirit of the platform, and a ban signal. If you follow
  someone, you stay following them.
- **Follow then show up.** A follow on its own does little. The point is that it
  keeps their content in front of you so you keep liking and replying over the
  following days. That consistency is what reads as a real connection.
- **Stay under the follow cap** in [safety.md](safety.md), and follow gradually
  across the session, not in a burst.

## Reply recipes (pick by context, never template verbatim)

Every reply must reference the **specific thing they posted**. Lead with value,
not the product. **Write it human** (see [voice.md](voice.md) "Human writing"):
no em dashes, no AI words, no tidy three-item lists. Run the draft through the
`humanizer` skill before posting.

- **Add insight:** answer their question or extend their point with a concrete
  tip. ("for a PH launch the first 3 seconds matter most. open on the outcome,
  not the login screen.")
- **Genuine reaction:** specific, not "great post 🔥". ("the way you handled the
  empty state here is clean. most tools skip that.")
- **Helpful question:** curious, not interrogating. ("did you script the
  walkthrough or freestyle it? curious how you keep them tight.")
- **Share relevant resource:** a tip or link that helps them. Only mention
  @trypitchdotco if it's genuinely the answer, and even then keep it soft.
  ("there are tools now that turn a written walkthrough straight into a narrated
  demo, skips the whole editing pass.")

### Reply do-not

- No pitch, no link, no "check out @trypitchdotco" in a first reply.
- No generic praise ("Amazing!", "So true!", "🔥🔥").
- No identical reply reused across people (anti-spam, hard rule).
- No em dashes, no AI-speak. Don't argue, don't be negative, don't hijack their
  thread.

## Cadence

- Space interactions naturally across a session and across days (see pacing in
  [safety.md](safety.md)). Don't like 8 posts from one person in a row.
- Revisit `warming` prospects every 1-3 days until they hit the DM bar.
- If they engage back (like/reply/follow you), that's a strong buy signal. Bump
  priority, but reply to the inbound interaction first. Shorten the path to DM
  only after the conversation has been acknowledged naturally.
- **Keep showing up after the DM too.** A real connection doesn't stop once
  you've pitched. Keep occasionally liking and replying to people you follow,
  whether they bought yet or not. The follow plus steady, genuine engagement
  over weeks is what turns a cold contact into someone who trusts the brand.
- **Follow back** genuine builders who follow @trypitchdotco and fit the ICP, and
  add them to the pipeline. An inbound follow is interest worth nurturing.

## Logging

Every like/reply/follow → append to `state/activity-log.jsonl` and increment the
prospect's `touches`, update `last_touch`. When the warm-up bar is met, move the
prospect to stage `warming` → ready, and set `next_action_date` for the DM.
