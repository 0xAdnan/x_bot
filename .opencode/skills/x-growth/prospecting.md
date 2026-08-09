# Prospecting, who to target & how to find them

Goal: find people on X who plausibly need product demo / marketing videos and
could buy PITCH. Quality over volume. A good prospect is worth ten random
follows.

## ICP and community segments

| Segment | Who | Core pain PITCH solves | Buying trigger |
|---|---|---|---|
| `founder` | Indie hackers, solo & small-team SaaS founders | No time/skill/budget for demo videos; launches need video | Shipping a feature, launching, posting on Product Hunt |
| `growth` | Growth / PMM / marketing at devtools & SaaS | Need constant demo, changelog, launch clips at volume | Hiring for "video/content", complaining about prod time |
| `agency` | Marketing agencies, freelance demo/video producers | Production is their bottleneck; demos eat margin | Talking about client load, scaling, turnaround time |
| `creator` | Tech YouTubers, course creators, devrel, educators | Tutorials/walkthroughs need polish without edit time | Publishing tutorials, "spent X hours editing" posts |
| `community` | Builders asking for feedback, help, launch advice, or product critique | They may not be buyers yet, but they build the trusted orbit | Asking for help, posting a link, engaging with @trypitchdotco |

## Where / how to search (browser)

Run searches on `x.com/search`. Rotate queries; don't hammer one. Examples:

**Founders / shippers**
- `"just shipped" (saas OR app OR tool) min_faves:5`
- `"launching" (on product hunt OR PH) -is:retweet`
- `"built this" demo lang:en min_faves:10`
- `#buildinpublic` recent + has a product link in bio

**Growth / SaaS marketing**
- `("product demo" OR "demo video" OR "launch video") (hard OR expensive OR "took forever")`
- `("changelog" OR "release video") saas`
- hiring posts: `"hiring" ("video" OR "content" OR "demo") startup`

**Agencies / freelancers**
- `(agency OR freelance) ("product demo" OR "explainer" OR "saas video")`
- `"client" ("turnaround" OR "revisions" OR "editing") video`

**Creators / educators**
- `("tutorial" OR "walkthrough") (editing OR "hours editing") dev`
- `("course" OR "devrel") (demo OR screencast)`

**Community / tech-founder conversations**
- `"rate my landing page" startup -is:retweet`
- `"roast my startup" OR "roast my landing page"`
- `"I need help" (SaaS OR startup OR launch OR demo)`
- `("AI agent" OR "browser automation" OR "codegen") (shipping OR building)`
- `("onboarding" OR "activation") (SaaS OR devtool) min_faves:5`

### "Drop your link" / "What are you building" thread mining

Some accounts post threads asking followers to share their products in
replies — these threads are goldmines of founders actively seeking exposure.

**Accounts to watch** (look for ones with engaged followings):
- Launch Llama (45k founders), maker growth accounts, build-in-public curators
- Anyone running recurring "share your startup" threads with real engagement
- Product Hunt launch announcement replies
- "Rate my landing page" / "review my SaaS" threads
- Posts with hooks like `"drop your link"`, `"share your website"`,
  `"what are you building"`, `"post your product"`, `"promote yourself"`

**How to mine them:**
1. Search X for recent "drop your link" / "what are you building" / "share your
   startup" posts with high engagement (lots of replies).
2. Navigate to the post and scroll through the replies.
3. For each reply that includes a product link, quickly evaluate:
   - Does the product need a demo video? (SaaS, app, devtool, etc.)
   - Is the profile active and real? (pinned post, bio, recent activity)
   - Is the founder personally building it? (solo/small team = better fit)
4. Score, dedupe, and add qualified ones to the pipeline at stage `new`.

**Why this works:** Someone posting their product in a discovery thread is
actively looking for ways to grow — they're more receptive to a demo video
that helps them convert visitors. They're also used to getting DMs from these
threads, so outreach feels natural. These prospects tend to warm up faster
because they're already in a "promote my product" mindset.

### YC & Antler Competitors & Adjacent Product Demo Startups

Watch these accounts and target users engaging with or complaining about their tools:

**1. Primary Competitor Accounts (YC & Antler Cohorts + Adjacent Tools):**
- **YC Alumni:** `@supademo` (YC S22), `@tangohq` (YC W21), `@guidde_io` (YC S22), `@hyperbound` (YC W24), `@tolstoyhq` (YC W22)
- **Interactive & Screen Recording Tools:** `@arcade_dev`, `@tella_edu`, `@screenstudio`, `@storylane_io`, `@guideflow`, `@demostack`, `@loom`, `@descript`, `@synthesiaIO`, `@heygen_ai`
- **Antler Cohort Video Startups:** `@quickvid_ai`, `@fable_demo`, `@guideflow`

**2. Specialized Search Queries:**
- `("Supademo" OR "Tango" OR "Guidde" OR "Storylane") (demo OR walkthrough OR "interactive demo")`
- `("Screen Studio" OR "Tella" OR "Descript") (editing OR voiceover OR narration OR "took 2 hours")`
- `("YC demo" OR "YC launch") (video OR "need a video" OR "explainer")`

**3. Engagement Rule:**
When founders complain about manual editing or recording overhead on these tools, reply with `@trypitchdotco`'s contrast play: *"Automate browser capture + AI narration directly from plain text walkthroughs — zero editing required."*



For community discovery, the bar is not "can we sell immediately?" The bar is
"can we be genuinely useful and would this builder belong in the
@trypitchdotco orbit?" If yes, help first and add them as `community` or the
best-fit buyer segment.

## Qualify, strong signals (raise score)

- Has a real product / SaaS with a link in bio or pinned post (+3)
- Recently shipped/launched something demoable in last 14 days (+3)
- Explicitly mentions needing/struggling with demo or marketing video (+4)
- Active account (posts in last 7 days, real engagement) (+2)
- Sounds like a buyer/decision-maker (founder, growth lead, owns the product) (+2)
- Audience/reach that makes them worth converting or partnering (+1)

## Disqualify, skip or `do-not-contact`

- No product, no link, pure shitposting/personal account
- Competitor or works at a directly competing video-demo tool
- Big brand with an obvious in-house video team (low fit, low intent)
- Account looks like a bot, dormant (no posts in 30+ days), or anonymous troll
- Anti-AI / anti-automation stance in bio or recent posts
- Already a customer or already in our pipeline (dedupe!)

## Lead scoring

Sum the signals above. Set `score` on the prospect row.

- **≥ 8** → high priority. Warm up fast, prioritize for DM.
- **4-7** → standard. Normal warm-up cadence.
- **< 4** → park at `new` with low priority or drop. Don't waste DMs.

## Recording a new prospect

Before adding, **dedupe**: grep `state/prospects.jsonl` for the handle. If
present, update instead of duplicating. New qualified prospects are appended at
stage `new` with `score`, `segment`, `product_url`, and a one-line `why` (the
specific reason they fit, referenced later in outreach so it's personal).
