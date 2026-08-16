---
name: yc-antler-outreach
version: 1.0.0
description: >-
  Discover upcoming YC & Antler Global startups, extract contact emails and X handles,
  conduct deep product research, and execute value-first demo/launch video outreach
  for PITCH (trypitch.co). Enforces strict anti-AI human writing guidelines with
  zero em-dashes and zero robotic marketing fluff.
license: MIT
compatibility: claude-code opencode
allowed-tools:
  - read
  - write
  - edit
  - bash
  - webfetch
---

# YC & Antler Startup Scout and Outreach Pipeline

This skill automates the entire end-to-end outbound acquisition funnel for early-stage startups in **Y Combinator** (e.g. Winter 2026, Summer 2025, Winter 2025, Launch HN) and **Antler Global** (US, UK, Singapore, Continental Europe, India, etc.).

## Core Value Proposition for Startups

Founders launching at YC Demo Day or Antler Demo Days need crisp, high-converting product demo videos for:
1. Launch announcements on X, Hacker News, and Product Hunt.
2. Investor pitches and demo day presentation slides.
3. Landing page conversion & interactive onboarding.

**PITCH (trypitch.co)** solves their biggest pain: manually recording, screencasting, and editing demo videos takes days. PITCH turns a plain text walkthrough or URL prompt into a studio-quality, narrated MP4 in minutes, allowing founders to iterate instantly.

---

## Strict Anti-AI & Human Writing Rules (Mandatory)

Before sending or drafting ANY message, enforce these rules:

1. **Zero Em-Dashes (—) and Zero En-Dashes (–)**: These are the #1 tell of generic AI output. Use periods, commas, or parentheses instead.
2. **Zero Banned AI Words**: Never use *delve, elevate, unlock, leverage, seamless, robust, game-changer, revolutionary, landscape, transformative, streamline, navigate, bespoke, beacon, testament*.
3. **Zero Emojis**: Never use 🚀, 🔥, 💡, 🤖, ✨ or generic hype symbols.
4. **No Rule-of-Three Adjective Lists**: Real founders do not write *"fast, intuitive, and powerful"*.
5. **No Robotic Openings**: Never use *"I hope this email finds you well"*, *"I came across your amazing product"*, or fake flattery.
6. **Peer-to-Peer Founder Voice**: Practical, direct, concise, lowercase-friendly where natural, grounded in real software craft.

---

## System Workflow

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        YC & ANTLER OUTREACH SYSTEM WORKFLOW                             │
└────────────────────────────────────────────────────────────────────────────────────────┘

  1. DISCOVERY & SCRAPING
     ├─ YC Directory (Winter 2026, Summer 2025, Winter 2025) via Algolia API
     ├─ Launch HN Stories via Hacker News Algolia
     └─ Antler Global Portfolio (US, UK, Singapore, Europe, India) via Webflow CMS

  2. CONTACT & PRODUCT ENRICHMENT
     ├─ Scrapes founder profiles & socials (X handles, LinkedIn)
     ├─ Crawls startup landing pages & subpages (/contact, /about, /team) for emails
     └─ Analyzes core product workflow & demo hook

  3. ANTI-AI DRAFT GENERATION
     ├─ Personalized Cold Email (Subject + 3-4 sentence value proposition)
     ├─ Tailored X DM (2-3 sentences max)
     └─ Free done-for-you 30s demo offer + link to trypitch.co

  4. CRM PERSISTENCE & DEDUPLICATION
     ├─ Deduplicates against SQLite DB (`data/pitch_bot.db`)
     ├─ Saves to `prospects` table (`segment = 'yc'` or `'antler'`, `stage = 'researched'`)
     └─ Mirrors to `.opencode/skills/x-growth/state/prospects.jsonl`

  5. OUTREACH DISPATCH
     ├─ Email: Send via configured SMTP or export ready-to-send campaigns to CSV
     └─ X DM: Send via `agent-webbridge` (profile `Testing`) respecting safety caps
```

---

## Tooling Commands

All scripts live in `.opencode/skills/yc-antler-outreach/scripts/`:

### 1. Run Complete Discovery & Enrichment Pipeline
```bash
# Scout 10 startups across YC and Antler, enrich contacts, generate anti-AI drafts, save to CRM
python3 .opencode/skills/yc-antler-outreach/scripts/scout_enrich.py --source all --limit 10

# Dry-run without modifying database
python3 .opencode/skills/yc-antler-outreach/scripts/scout_enrich.py --source yc --batch "Winter 2026" --limit 5 --dry-run

# Scout only Antler Global startups
python3 .opencode/skills/yc-antler-outreach/scripts/scout_enrich.py --source antler --limit 10
```

### 2. Review & Export Ready Outreach
```bash
# List researched startups ready for outreach
python3 .opencode/skills/yc-antler-outreach/scripts/send_outreach.py --list --limit 20

# Export researched prospects with drafts to CSV
python3 .opencode/skills/yc-antler-outreach/scripts/send_outreach.py --export-csv data/startup_outreach.csv
```

### 3. Safety & Circuit Breaker Checks
Before performing browser automation or X DMs:
```bash
./target/release/pitch-cli circuit-breaker
./target/release/pitch-cli budget
```
- Daily safety caps: Max 15 DMs/day, burst spacing ≥ 90s.
- All browser X actions must use `agent-webbridge` with `"profile": "Testing"`.
