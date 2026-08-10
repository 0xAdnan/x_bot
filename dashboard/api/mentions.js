export default async function handler(req, res) {
  const supabaseUrl = process.env.SUPABASE_URL || "https://jwswpryozfxzaocimadp.supabase.co";
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY || process.env.SUPABASE_ANON_KEY;

  if (!supabaseUrl || !supabaseKey) {
    return res.status(500).json({ error: "Supabase credentials missing on server" });
  }

  const headers = {
    'apikey': supabaseKey,
    'Authorization': `Bearer ${supabaseKey}`,
    'Content-Type': 'application/json'
  };

  if (req.method === 'GET') {
    try {
      const response = await fetch(`${supabaseUrl}/rest/v1/mention_jobs?select=*&order=updated_at.desc&limit=50`, { headers });
      if (response.ok) {
        const jobs = await response.json();
        return res.status(200).json({ jobs, total: jobs.length });
      }
    } catch (err) {
      console.error("Supabase mention jobs fetch error:", err);
    }

    return res.status(200).json({ jobs: [], total: 0 });
  }

  if (req.method === 'POST') {
    try {
      let body = req.body;
      if (typeof body === 'string') {
        try { body = JSON.parse(body); } catch (e) {}
      }
      const jobs = Array.isArray(body) ? body : [body];

      const upsertHeaders = { ...headers, 'Prefer': 'resolution=merge-duplicates' };
      const response = await fetch(`${supabaseUrl}/rest/v1/mention_jobs`, {
        method: 'POST',
        headers: upsertHeaders,
        body: JSON.stringify(jobs)
      });

      if (response.ok) {
        return res.status(200).json({ status: "ok", synced: jobs.length });
      } else {
        const errText = await response.text();
        return res.status(500).json({ error: errText });
      }
    } catch (err) {
      console.error("Supabase mention jobs post error:", err);
      return res.status(500).json({ error: err.message });
    }
  }

  return res.status(405).json({ error: "Method not allowed" });
}
