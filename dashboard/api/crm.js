export default async function handler(req, res) {
  const supabaseUrl = process.env.SUPABASE_URL;
  const supabaseKey = process.env.SUPABASE_ANON_KEY || process.env.SUPABASE_SERVICE_ROLE_KEY;

  if (supabaseUrl && supabaseKey) {
    try {
      const response = await fetch(`${supabaseUrl}/rest/v1/prospects?select=*&order=updated_at.desc`, {
        headers: {
          'apikey': supabaseKey,
          'Authorization': `Bearer ${supabaseKey}`
        }
      });
      if (response.ok) {
        const prospects = await response.json();
        const stages = {
          new: [],
          warming: [],
          contacted: [],
          in_convo: [],
          trial: [],
          customer: [],
          "do-not-contact": [],
          lost: []
        };
        for (const p of prospects) {
          const st = p.stage || "new";
          if (stages[st]) {
            stages[st].push(p);
          } else {
            stages["new"].push(p);
          }
        }
        return res.status(200).json({ prospects, stages, total: prospects.length });
      }
    } catch (err) {
      console.error("Supabase CRM fetch error:", err);
    }
  }

  // Fallback
  return res.status(200).json({
    prospects: [],
    stages: {
      new: [],
      warming: [],
      contacted: [],
      in_convo: [],
      trial: [],
      customer: [],
      "do-not-contact": [],
      lost: []
    },
    total: 0
  });
}
