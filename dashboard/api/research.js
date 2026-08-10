export default async function handler(req, res) {
  const supabaseUrl = process.env.SUPABASE_URL || "https://jwswpryozfxzaocimadp.supabase.co";
  const supabaseKey = process.env.SUPABASE_ANON_KEY || "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imp3c3dwcnlvemZ4emFvY2ltYWRwIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODYyNzQzMjQsImV4cCI6MjEwMTg1MDMyNH0.U575XIPsA12Y3JpZJ_gr9T7xH4WZafihkThQJvh8VNo";

  try {
    const prospectsUrl = `${supabaseUrl}/rest/v1/prospects?select=*&order=score.desc`;
    const resp = await fetch(prospectsUrl, {
      headers: {
        "apikey": supabaseKey,
        "Authorization": `Bearer ${supabaseKey}`
      }
    });

    const prospects = resp.ok ? await resp.json() : [];
    const account = req.query.account || 'all';

    let filteredProspects = prospects || [];
    if (account !== 'all') {
      filteredProspects = filteredProspects.filter(p => p.account === account || !p.account);
    }

    // Active 24/7 AI Trend & Competitor Research Queries
    const researchQueries = [
      { query: 'tella.tv OR screen.studio OR loom', category: 'Viral Competitor Mentions', priority: 'High', status: 'Scanning Live' },
      { query: 'AI video generation trend 2026', category: 'Trending Tech Topic', priority: 'Urgent', status: 'Scanning Live' },
      { query: 'SaaS demo video meme', category: 'Relatable Meme & Hook', priority: 'High', status: 'Scanning Live' },
      { query: 'looking for screen studio alternative', category: 'High Intent Lead Search', priority: 'Urgent', status: 'Scanning Live' },
      { query: 'Product Hunt top SaaS launch video', category: 'Viral Launch Format', priority: 'High', status: 'Scanning Live' },
      { query: 'YC W26 / S26 AI demo video', category: 'ICP Founder Research', priority: 'Urgent', status: 'Scanning Live' }
    ];

    // Algorithmic Prioritization Scoring Matrix (from twitter-algorithm-optimizer skill)
    const scoringRules = [
      { rule: 'Relatable SaaS/AI Builder Pain Point (+3 pts)', points: '+3 Points', description: 'Has high potential for retweets and bookmarks' },
      { rule: 'Explicit Competitor Comparison (@Tella, @Loom) (+3 pts)', points: '+3 Points', description: 'Triggers active debate and community replies' },
      { rule: 'Live Product/Website URL Present (+2 pts)', points: '+2 Points', description: 'Allows instant 60s AI video demo generation' },
      { rule: 'High-Intent Founder / ICP Bio Match (+2 pts)', points: '+2 Points', description: 'High probability of trial conversion' }
    ];

    // Non-Generic Viral Content Queue (Algorithmic Triggers & Memes)
    const generatedContentQueue = [
      {
        account: '@trypitchdotco',
        type: '🔥 Relatable AI Builder Meme / Hot Take',
        hook: 'Spent 3 days recording a product demo in Loom vs Spent 60 seconds pasting a URL into PITCH 🎬\n\nWhich one are you choosing in 2026? Drop your hot take below 👇',
        status: 'Queued for Next Run',
        preset: 'Algorithm Signal: High Reply & Debate Trigger'
      },
      {
        account: '@trypitchdotco',
        type: '🚀 Viral Competitor Comparison',
        hook: 'Stop spending $500 on manual video editors for every new feature launch. PITCH automatically turns any live URL into a 1080p narrated video pitch in 60s.',
        status: 'Queued for Next Run',
        preset: 'Algorithm Signal: High Retweet & Bookmark Value'
      },
      {
        account: '@adnanspitch',
        type: '🎯 High-Converting Personalized Outreach Hook',
        hook: 'Hey @founder, saw you shipped a major update! Created a 60s AI video demo walkthrough for your landing page using PITCH — check out the rendered video link here!',
        status: 'Pre-cooked in CRM',
        preset: 'Algorithm Signal: Direct Lead Conversion'
      }
    ];

    return res.status(200).json({
      status: "ok",
      queries: researchQueries,
      scoringRules: scoringRules,
      prospects: filteredProspects,
      contentQueue: generatedContentQueue
    });
  } catch (err) {
    console.error("[Research API Error]:", err);
    return res.status(500).json({ error: err.message });
  }
}
