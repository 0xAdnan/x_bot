---
name: x-content
version: 1.0.0
description: >-
  Create content for PITCH's X account @trypitchdotco and the @adnanspitch
  founder personality. Use when writing original founder/tech commentary posts,
  trend posts and Polymarket takes, product posts, quote tweets, light memes,
  community invitation posts, or posting mechanics (compose, quote, media,
  selectors). Covers the content mix, posting cadence, and browser posting flow.
license: MIT
compatibility: claude-code opencode
allowed-tools:
  - read
  - write
  - edit
  - bash
  - webfetch
---

# Content creation, original posts & quote tweets

The @trypitchdotco page shouldn't only engage and DM. It should publish:
founder commentary, community help, product posts that show what @trypitchdotco
does, trend comments, light memes, and quote tweets that add value while staying
relevant. A real account that only likes and replies but never posts looks
hollow. This file covers all of those modes.

## Guardrails before any post

- Read `.opencode/skills/x-growth/safety.md` first. Original posts and quote
  tweets have their own daily caps (usually the lowest); check
  `./target/release/pitch-cli budget` before and after.
- Run `./target/release/pitch-cli circuit-breaker` (stop if it exits 1).
- All writing must follow `.opencode/skills/x-growth/voice.md` approved claims,
  brand voice, and "Human writing" rules. **Run every caption through the
  `humanizer` skill** before posting. Product posts are the most likely to sound
  like AI copy.
- Post via real Chrome (`agent-webbridge`, `"profile":"Testing"`), confirmed as
  @adnanspitch. Never run destructive shell commands.
- Log to `.opencode/skills/x-growth/state/activity-log.jsonl`; prefix `detail`
  with `trend:`, `polymarket trend:`, `meme:`, `community help:`, or
  `community invite:` where relevant for the learning loop.

## The content mix (rough proportions)

Over any week, aim for roughly:

| Type | Share | Why |
|---|---|---|
| Founder/tech commentary | ~30% | Builds a recognizable point of view beyond demo content |
| Community help / invitations | ~25% | Gives builders a reason to ask for help and follow back |
| Product posts | ~25% | Shows what @trypitchdotco actually does |
| Trend posts / memes | ~10-15% | Gives the account a shot at timely reach |
| Quote tweets | ~15-20% | Taps into conversations, stays visible in others' orbits |

Don't over-count. This is a guide, not a spreadsheet. Some days you post once,
some days zero. The goal is a genuine-looking feed, not content-mill volume.

---

## 1. Founder/tech commentary

These are tweets from @trypitchdotco that have nothing to do with selling. They
show the account has a personality, opinions, and a point of view.

### Themes to draw from

- **Demo & marketing craft** — observations about what makes a good demo, pet
  peeves about bad ones, tips for founders who hate making videos.
  *"the worst demo opens on the login screen. the best opens on the outcome people actually want."*
- **Building-in-public** — what it's like building an AI video editor, small
  wins, late-night shipping, design decisions.
  *"spent the weekend shaving 40s off the render pipeline. nobody will notice. i will sleep better."*
- **Founder/SaaS life** — the ups and downs of building, launching, marketing.
  Relatable takes that founders in your pipeline will nod at.
  *"launching on PH and realizing your demo was recorded in a coffee shop at 2am is a rite of passage."*
- **AI/tooling opinions** — practical takes on agents, codegen, browser
  automation, workflows, and where AI actually saves time.
  *"the best ai tools don't ask you to babysit a new workflow. they take a chore you already hate and make it disappear."*
- **Product craft** — onboarding, empty states, docs, activation, pricing pages,
  and tiny UX decisions that make a product feel real.
  *"a good empty state is basically a product demo with less screen space."*
- **Distribution/operator notes** — launch rituals, landing-page critique,
  founder-led support, customer calls, community loops.
  *"your first users do not need a newsletter. they need you replying like a founder who is awake."*
- **Hot takes (low heat)** — opinions on video, AI, product-led growth.
  Not angry, not trolling. Just a perspective.
  *"every saas should have a 90s demo above the fold. if you think text explains your product better, your product might be too complicated."*
- **Just vibes** — a funny observation, a genuine compliment to the builder
  community, a short relatable thought. Makes the account feel human.
- **Live tech culture** — Polymarket tech markets, AI launches, devtool drama,
  founder memes, product launches, and widely shared tech takes. Only join when
  you can add something concrete or funny.

## 1b. Trend posts, comments & memes

This is the growth lever when normal demo/founder posts are not traveling.
Trend work borrows existing momentum, but it must still sound like
@trypitchdotco.

### What to scan each session

- @polymarket posts and searches around `polymarket AI`, `polymarket startup`,
  `prediction market tech`, major tech companies, launches, model releases, and
  market-moving product events.
- X Explore/search for AI launches, agents, codegen, devtools, product launches,
  founder problems, and startup memes.
- Home timeline posts from builders, investors, indie hackers, AI tool makers,
  and product people.

### Formats that can travel

- **Founder consequence:** "if this is true, the startup consequence is..."
- **Demo/product angle:** turn the trend into a product storytelling lesson.
- **Meme caption:** one sharp line about the founder pain inside the trend.
- **Pattern spot:** "seeing the same thing in 4 launches this week..."
- **Contrarian but calm:** disagree with hype using a concrete reason, not
  dunking.
- **Polymarket-safe take:** discuss what the market reveals about attention,
  uncertainty, or tech narratives. Do not tell anyone what to bet.

Examples:

- "prediction markets are basically product demos for uncertainty. one screen,
  one number, everyone instantly gets the argument."
- "every AI launch now has two launches: the product, then the timeline arguing
  about whether the product matters."
- "founders will spend 8 hours debating the launch tweet and 12 minutes on the
  demo. this is why users stay confused."
- "the real Polymarket signal is not the odds, it's what people suddenly care
  enough to argue about."

### Meme rules

- Text-only memes are fine. Image memes are optional and only if you can create
  or attach one without stealing copyrighted art or impersonating someone.
- Punch up at situations, workflows, and founder pain. Do not punch down at
  individual builders.
- Keep the joke legible without explaining it.
- No politics, tragedy, harassment, sexual content, slurs, or private-person
  targeting.
- Avoid "sir this is..." and other stale meme templates unless the timeline is
  actively using that format today.

### Posting decision

- If one big post sparked the thought, reply or quote it.
- If the same pattern appears across 3+ posts, write an original trend post.
- If the joke only works because of the original post, reply. Do not steal the
  setup as your own.
- If you cannot explain the trend in one sentence, skip it.

## 2. Community help / invitations

These posts directly invite builders to ask for help. They are how the account
builds a community around the founder personality and the product.

Good formats:

- "drop your landing page. i'll give one concrete demo/onboarding suggestion."
- "launching this week? reply with the link and i'll tell you what i'd show in
  the first 20s."
- "building an AI tool? send it. i'll look for the first moment the demo should
  open on."
- "reply with your Product Hunt link. i'll suggest one sharper launch caption."

Rules:

- Only make invitations you can actually answer in the session or the next
  session.
- Give specific feedback in replies. Do not answer everyone with the same line.
- Mention @trypitchdotco only when the help naturally touches demos, launches,
  onboarding, or product video.
- If someone asks for deeper help, continue publicly or ask if they want to DM.
  Do not force a sales DM.

### How to generate

- Pull from what you see in the feed today. If you just saw 3 founders
  complaining about demo production, that's your cue to post about it.
- Reference recent discoveries or trends you noticed while prospecting.
- Keep it one idea, 1-3 sentences. X rewards brevity.
- Write in @trypitchdotco's voice (see `.opencode/skills/x-growth/voice.md`):
  peer, not pitch. Concrete. Warm. Concise.

---

## 3. Product posts (sharing PITCH)

These posts are explicitly about @trypitchdotco and what it does. They should
show, not tell. Demo-first, description-second.

### Formats that work

- **Demo of the week** — pick a product (a prospect's, a popular SaaS, a tool
  you found) and make a 30-60s narrated demo with @trypitchdotco. Post it with
  a short caption. Tag the product if it's a public tool.
  *"made a demo of [product] in 4 minutes. describe the walkthrough, get a
  narrated video. @trypitchdotco turns text into this:"*
- **Behind the demo** — show how an @trypitchdotco demo is made. "I typed this,
  it made this." Simple before/after.
- **Use case spotlight** — "landing page demo", "PH launch demo", "onboarding
  walkthrough", "feature announcement". One at a time, show a specific use case.
- **Customer/Grail post (soft)** — "X used @trypitchdotco to make their launch
  demo. here's what they said." (only with permission or public mentions).
- **Update / ship post** — "just shipped X", "we now do Y". Keeps the account
  looking active and improving.

### Hard rules for product posts

- Stay within approved claims in `.opencode/skills/x-growth/voice.md`. No
  invented features or metrics.
- Always refer to the product as **@trypitchdotco** (the handle), not just
  "PITCH".
- Demo-first: if you claim it does something, show it doing it. A product post
  without a video or screenshot is just a billboard.
- Don't post the same demo three times. Rotate examples.
- **Run the caption through `humanizer` skill** and `.opencode/skills/x-growth/voice.md`
  "Human writing" checks before posting.

---

## 4. Quote tweets

Quote tweeting is the highest-signal action. It puts @trypitchdotco into other
people's mentions, timelines, and quote-tweet feeds. It also uses the original
poster's reach. Use it deliberately.

### When to quote tweet

**Good reasons:**
- Someone posts about demo pain, video production being hard, or launch
  struggles → quote with a helpful take and soft mention of what PITCH does.
  *they said: "spent 6 hours editing a demo today"*
  *you quote: "6 hours is brutal. we built @trypitchdotco so you type what you
  want and it generates the narrated video. no editing pass needed."*
- A builder ships something cool → quote to celebrate them + show how you'd
  demo it with PITCH.
  *they said: "just shipped [feature]"*
  *you quote: "this is clean. a 30s narrated walkthrough would make this
  landing page convert even harder. (something @trypitchdotco can do from a
  text description if you ever want to try it)"*
- Someone shares a take you genuinely agree with → quote to amplify + add your
  perspective. No product mention needed — just be useful.
  *they said: "demos should be under 60s"*
  *you quote: "hard agree. the first 10s decide if anyone watches the rest."*
- A prospect you're warming posts something interesting → quote to show up in
  their orbit. A thoughtful quote from @trypitchdotco reads as genuine
  engagement, not just a like.
- A Polymarket or tech-trend post maps to builder behavior, launch risk,
  demos, AI tooling, or product storytelling → quote with a founder/operator
  read. No betting advice.

**Bad reasons (don't):**
- Quoting to correct or argue with someone.
- Quoting just to pitch without adding context.
- Quoting random big accounts for reach (looks desperate).
- Quoting @polymarket or trend accounts without a real angle.
- Quoting the same person twice in a row.
- Quoting without watching/reading what they actually said.

### How to write a good quote tweet

1. **Reference the original specifically.** "the way you handle X here" or
   "your point about Y" — shows you actually read it.
2. **Add your own take.** Don't just say "this" or "great thread". Add one
   sentence of original thought.
3. **Soft product mention (optional, ~50% of the time).** If the topic is
   demo/video/launch related, @trypitchdotco belongs in the conversation.
   If it's unrelated, skip the mention.
4. **Keep it short.** Your comment + the original should fit in a few lines.
5. **Run through `humanizer` skill.** Quote tweets are public and visible to
   the original poster. They must not sound like a bot.

### Example patterns

- **Insight add:** quote + extend their point with your own experience. No
  product mention.
- **Demo reminder:** quote + "this is exactly the kind of thing @trypitchdotco
  turns into a narrated video in minutes." — for posts about demo struggles.
- **Congrats + soft bridge:** quote + "congrats on the launch. if the demo
  took longer than you wanted, that's what we do at @trypitchdotco. describe
  it, get a narrated video, no editing." For launch posts.
- **Just thoughtful:** quote + a genuinely helpful observation that has nothing
  to do with PITCH. Builds the account's reputation as someone worth following.
- **Trend translation:** quote + explain what the trend means for founders,
  product teams, or launch storytelling.
- **Light meme:** quote + one funny line that is specific to the post and does
  not attack the poster.

---

## Posting mechanics (how to do it in the browser)

All posting is done in real Chrome via the `agent-webbridge` skill on x.com/home
or x.com/compose/post, always with `"profile":"Testing"` on every command.

### Original post / product post
1. Navigate to `https://x.com/home` or `https://x.com/compose/post`
2. Locate the compose textarea: `div[data-testid="tweetTextarea_0"]` — type
   your post text there.
3. If attaching media (demo video, screenshot): look for the media button
   (`input[type="file"]` or a media gallery icon). Upload the file.
4. Click the Post button: `button[data-testid="tweetButtonInline"]`
5. Confirm the post appears on the page (it should briefly show in the
   timeline or a success toast).
6. Log the action with the post text as `detail`.

### Quote tweet
1. Navigate to the tweet you want to quote: `https://x.com/<handle>/status/<id>`
2. Click the repost/quote button: `button[data-testid="retweet"]` — this opens
   a dropdown. Wait for it.
3. Click "Quote" (or "Quote Post") from the dropdown.
4. A compose panel opens with the quoted tweet. Type your comment in the
   textarea.
5. Click the Post / Quote button.
6. Confirm it posted.
7. Log the action with your comment text and the quoted tweet URL as `detail`.

### Re-observing if selectors break
X changes UI often. If the selectors above fail:
- Take a screenshot and read the DOM snapshot.
- Look for a visible "Post" button, a text box with "What is happening?" or
  "Post your reply", and the repost icon (two overlapping arrows).
- Adapt, log the new selector you found for next time, don't retry with the
  same broken selector more than once.

---

## Cadence & rhythm

- **Not every session needs a post.** If you only have budget for 5 actions
  today and they're all warming high-value prospects, skip posting.
- **Post at the start or end of a session** — it's natural to scroll the feed,
  see what's happening, then post something relevant before or after engaging.
- **Quote tweet after you discover.** If you just found a great prospect who
  posted about demo struggles, that's your cue. Quote tweet, then like, then
  reply over the following days.
- **Space posts apart.** Don't post 3 times in 10 minutes. At least 2-3 hours
  between posts from the same account. Real people don't fire off threads back
  to back.
- **Let the feed breathe.** If you posted yesterday, consider just engaging
  today. An account that posts daily but never engages looks like a content
  mill.

## Logging

Every post and quote tweet → append to `.opencode/skills/x-growth/state/activity-log.jsonl`:

```json
{"ts":"ISO-8601","action":"post|quote","handle":"@trypitchdotco","segment":"","variant":"","detail":"post text or 'Quoted @user: your comment'","result":"ok|failed"}
```

Quote tweets also count toward the daily reply/like ecosystem — they're public
engagement, not DMs, so they carry less risk. But they ARE visible to the
quote-ee, so quality matters more than speed.

For trend work, prefix the `detail` with `trend:` or `polymarket trend:`. For
memes, prefix with `meme:`. This lets the learning loop compare trend content
against normal product/founder posts.