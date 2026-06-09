# state/, the agent's CRM

Two append-only JSONL files. The agent reads and updates these on every action.
Don't hand-edit while a session is running.

## prospects.jsonl

One JSON object per prospect (the pipeline). Schema:

| field | meaning |
|---|---|
| `handle` | `@name`, unique key, used for dedupe |
| `name` | display name |
| `url` | link to their X profile |
| `segment` | `founder` \| `growth` \| `agency` \| `creator` \| `community` |
| `score` | lead score from prospecting.md |
| `stage` | `new` → `warming` → `contacted` → `in_convo` → `trial` → `customer`; dead ends `lost`, `do-not-contact` |
| `last_touch` | YYYY-MM-DD of last action |
| `next_action_date` | YYYY-MM-DD the agent should act next |
| `touches` | count of interactions so far |
| `product_url` | their product/SaaS link (used to make the free demo) |
| `last_variant` | which opener variant was used (for outcome attribution) |
| `outcome` | `ignored` \| `replied` \| `positive` \| `declined` \| `trial` \| `customer` |
| `notes` | free text, conversation memory |
| `why` | one-line reason they fit (used to personalize outreach) |

## activity-log.jsonl

One JSON object per action taken. Schema:

| field | meaning |
|---|---|
| `ts` | ISO-8601 timestamp |
| `action` | `like` \| `reply` \| `follow` \| `dm` \| `followup` \| `outcome` \| `discover` \| `post` \| `quote` \| `failed` |
| `handle` | who it was directed at |
| `segment` | ICP segment (on `dm`/`followup`/`outcome`, for learning) |
| `variant` | opener variant used (on `dm`/`followup`/`outcome`, for learning) |
| `detail` | short description / text sent; for `outcome`: `ignored`\|`replied`\|`positive`\|`declined`\|`trial`\|`customer` |
| `result` | `ok` \| `failed` \| `skipped` |

Used for (a) daily rate-limit accounting via `budget.sh`, (b) outcome analytics
via `stats.sh` (the learning loop), and (c) honesty, every logged action must
have actually happened in the browser.

## insights.md

The agent's adaptive memory: which approaches are winning/losing per segment. The
agent reads it every session and updates it during the learning retro (see
`learn.md`). It's the **only** strategy file the agent self-edits, hard rules,
brand voice, and rate caps stay fixed.
