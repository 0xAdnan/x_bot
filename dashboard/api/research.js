import { fetchRust } from './_rust.js';

export default async function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  const resRust = await fetchRust('/api/research');
  if (resRust.ok) {
    return res.status(200).json(resRust.data);
  }
  return res.status(200).json({
    queries: [
      { query: '(@Polymarket OR Polymarket) (AI OR LLM OR "vibe coding" OR founder)', category: 'Polymarket & AI Discourse', priority: 'Urgent' },
      { query: '(@levelsio OR @rowancheung OR @swyx OR @bindureddy OR @karpathy) (demo OR building OR shipping)', category: 'AI Influencer Threads', priority: 'High' },
      { query: '("drop your link" OR "drop your saas" OR "what are you building" OR "rate my landing page")', category: 'Builder Drop Threads', priority: 'High' },
      { query: '"loom alternative" OR "tella.tv alternative" OR "screen studio alternative"', category: 'Competitor Pain Points', priority: 'High' },
      { query: '"need a demo video" OR "how to make a product demo" OR "recording a demo"', category: 'High Intent Requests', priority: 'Urgent' },
      { query: '(@ProductHunt OR "Product Hunt") (launching OR "live today" OR "product of the day")', category: 'Product Hunt Launches', priority: 'Medium' },
      { query: '("YC W26" OR "YC S26" OR "YC S25") (demo OR launch OR link)', category: 'YC Founder Batches', priority: 'Medium' }
    ],
    trends: [
      {
        topic: 'Polymarket: Vibe Coding vs Manual Video Production',
        volume: 'Viral / High Heat',
        sentiment: 'Sarcastic / Provocative',
        take: 'polymarket 90% probability that devs vibe-code an entire app in 15 mins with claude then spend 6 hours sweating through 45 screen studio takes because they coughed at 0:54. automated narrated walkthroughs on @trypitchdotco solve this completely.',
        type: 'Rage-Bait / Sarcastic'
      },
      {
        topic: 'Screen Studio RAM & Retake Fatigue',
        volume: 'Relatable Pain',
        sentiment: 'Empathetic / Builder Humor',
        take: 'nothing tests founder sanity like doing take #27 of a product demo video and seeing a slack popup mid-zoom. we built @trypitchdotco so you give a prompt/url and get a studio 60s demo with voiceover in 1 minute.',
        type: 'Pain Point'
      },
      {
        topic: 'AI Agent Drops & MCP Workflow Demos',
        volume: 'Trending Tech',
        sentiment: 'Educational / Tech Forward',
        take: 'the hardest part about building an AI agent isn\'t the backend logic, it\'s making a demo video that actually explains what it does without boring people to death in 10 seconds.',
        type: 'Tech Discourse'
      },
      {
        topic: 'Indie Hacker "Drop Your SaaS" Feed Infiltration',
        volume: 'High Conversion',
        sentiment: 'Value-First / Supportive',
        take: 'hopping into weekend "drop your link" threads and delivering free 45s product walkthroughs with @trypitchdotco to show founders how clean their UI looks with auto-zooms and narration.',
        type: 'Growth Play'
      }
    ],
    influencers: [
      {
        handle: '@levelsio',
        name: 'Pieter Levels',
        niche: 'Solopreneur / SaaS',
        followers: '500k+',
        baitAngle: 'Scrappy speed: shipping fast vs wasting hours on video edits',
        warmupHook: 'yo pieter, wild that people still spend days hiring video editors for micro-saas launches when you can generate the entire narrated product walkthrough in 60s from the landing page url'
      },
      {
        handle: '@karpathy',
        name: 'Andrej Karpathy',
        niche: 'AI / Deep Learning',
        followers: '1.2M+',
        baitAngle: 'Agentic evaluation: AI agents making their own visual demo videos',
        warmupHook: 'the logical conclusion of agentic workflows is agents demoing their own software with automated zoom sequences and synthetic narration instead of humans recording screencasts'
      },
      {
        handle: '@rowancheung',
        name: 'Rowan Cheung',
        niche: 'The Rundown AI',
        followers: '400k+',
        baitAngle: 'Curating next-gen generative video devtools',
        warmupHook: 'automated screen demo generation is low-key the most underrated AI productivity unlock this quarter. turning raw URLs into 60s narrated launch videos in one click'
      },
      {
        handle: '@bentossell',
        name: 'Ben Tossell',
        niche: 'AI Products / Ben\'s Bites',
        followers: '150k+',
        baitAngle: 'Reviewing prompt-to-video devtools',
        warmupHook: 'ben you should test prompt-to-demo video pipelines for your newsletter breakdowns. drop an app url and get a 45s polished overview video instantly'
      },
      {
        handle: '@swyx',
        name: 'swyx (Latent Space)',
        niche: 'AI Engineering & MCP',
        followers: '120k+',
        baitAngle: 'MCP video generation agents & programmatic video rendering',
        warmupHook: 'integrating MCP tools directly into automated video render pipelines: agent receives github repo -> reads README -> renders narrated demo video in 60s'
      },
      {
        handle: '@bindureddy',
        name: 'Bindu Reddy',
        niche: 'Abacus AI / LLM Debates',
        followers: '180k+',
        baitAngle: 'Pragmatic AI utility vs marketing fluff',
        warmupHook: 'most AI video tools generate hallucinatory fever dreams. the real enterprise utility is deterministic, pixel-perfect screen walkthroughs for devtools and SaaS'
      }
    ],
    memes: [
      {
        template: 'Drake Hotline Bling / Choice Meme',
        top: 'Spending 4 hours re-recording your screen demo because your dog barked at 1:58',
        bottom: 'Letting @trypitchdotco render a flawless 60s narrated demo while you drink coffee',
        caption: 'founder pain in one picture. why are we still recording screen demos manually in 2026',
        format: 'image_meme'
      },
      {
        template: 'Clown Makeup Progression',
        stages: [
          'Stage 1: I\'ll just record a quick 1-minute Loom for our launch',
          'Stage 2: Take 14... my microphone was on the wrong input',
          'Stage 3: Take 31... Slack notification popped up on the final second',
          'Stage 4: It\'s 3:30 AM, haven\'t shipped yet, still editing keyframes'
        ],
        caption: 'the screen recording pipeline of doom. @trypitchdotco turns any URL into a narrated demo in 60s so you can actually sleep',
        format: 'image_meme'
      },
      {
        template: 'Polymarket Odds Chart',
        top: 'Polymarket: 99.4% Probability',
        bottom: 'Vibe coders generate 15,000 lines of code in 10 minutes then take 3 business days to make a demo video',
        caption: 'odds don\'t lie. tag @trypitchdotco under your product link and stop suffering through screen recordings',
        format: 'chart_meme'
      },
      {
        template: 'Galaxy Brain / Expanding Brain',
        stages: [
          'Small Brain: 2,000 word markdown docs nobody reads',
          'Medium Brain: Uncut 12-minute Loom with 8 awkward pauses',
          'Large Brain: 50 takes on Screen Studio trying to time zooms perfectly',
          'Galaxy Brain: 1 prompt to @trypitchdotco -> instant 60s narrated launch video'
        ],
        caption: 'the evolution of SaaS onboarding. skip straight to galaxy brain',
        format: 'image_meme'
      }
    ],
    scoringRules: [
      { rule: 'Founder / CEO / CTO in bio', description: 'Target ICP decision maker', points: '+3' },
      { rule: 'Explicit demo video request', description: 'Immediate high intent', points: '+4' },
      { rule: 'Has live product URL', description: 'Can generate automated video', points: '+3' },
      { rule: 'Active in Polymarket / AI trend debates', description: 'High engagement viral multiplier', points: '+2' },
      { rule: 'Influencer commenter with >500 followers', description: 'Amplification reach potential', points: '+2' }
    ],
    contentQueue: []
  });
}
