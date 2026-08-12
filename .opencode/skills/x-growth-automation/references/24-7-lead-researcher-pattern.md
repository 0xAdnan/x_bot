# 24/7 Background Lead Researcher Architecture (`bin/researcher_daemon.py`)

This reference details the 24/7 background lead discovery engine that continuously mines high-intent prospects and competitor mentions without triggering X rate limits or account safety gates.

## 1. Safety & Read-Only Non-Disruptive Architecture
- **Read-Only Operations:** The 24/7 researcher daemon ONLY reads search feeds (`x.com/search?q=...&f=live`). It performs ZERO public write actions (likes, retweets, replies, DMs) during its background loops.
- **0% Safety/Ban Risk:** X anti-automation monitoring targets high-velocity public write actions. Passive reading and lead indexing carry 0% account risk.
- **Gentle Polling Cadence:** Runs background search queries every 45–60 seconds, cycling through target keywords.

## 2. Target Keyword & Competitor Search Queries
The daemon cycles through high-intent queries:
- Competitor Mentions: `tella.tv`, `screen.studio`, `loom.com alternative`, `looking for screen studio alternative`
- Problem / Intent Queries: `need a product demo video`, `how to make a product demo video`, `shipped a new feature video`, `showcase demo video SaaS`

## 3. Lead Scoring & Pre-Crafted Pitch Hook Generation
Each discovered prospect is scored (1–10) and enriched with a pre-crafted personalized outreach hook:
- **Score (+2):** Prospect tweet or bio contains a product URL
- **Score (+2):** Mentions competitor (@Tella, @ScreenStudio, @Loom, @GuiddeApp)
- **Score (+1):** Contains high-intent action phrases ("need", "looking for", "alternative")
- **Pre-Crafted Pitch Hook Example:** *"Hey @username, saw your post regarding [URL]! Created a 60s AI video demo walkthrough for [URL] using PITCH — check it out on pitch.co/demo!"*

## 4. Pre-Cooked CRM Lead Pool & Scheduled Action Windows
- **Supabase CRM Sync:** Discovered prospects are immediately upserted into Supabase `prospects` table (`account = '@adnanspitch'`, `stage = 'new'`).
- **Zero Time Wasted on Action Runs:** When scheduled outreach sessions run (8am / 2pm / 7pm), the agent opens Supabase, retrieves the top pre-cooked leads, and executes outreach in under 30 seconds without wasting time on research during the action window.
