export default async function handler(req, res) {
  const supabaseUrl = process.env.SUPABASE_URL;
  const supabaseKey = process.env.SUPABASE_ANON_KEY || process.env.SUPABASE_SERVICE_ROLE_KEY;

  if (supabaseUrl && supabaseKey) {
    try {
      const response = await fetch(`${supabaseUrl}/rest/v1/activities?select=*&order=ts.desc&limit=100`, {
        headers: {
          'apikey': supabaseKey,
          'Authorization': `Bearer ${supabaseKey}`
        }
      });
      if (response.ok) {
        const activities = await response.json();
        return res.status(200).json({ activities, total: activities.length });
      }
    } catch (err) {
      console.error("Supabase fetch error:", err);
    }
  }

  // Fallback if Supabase credentials are not yet configured
  return res.status(200).json({
    activities: [
      {
        ts: new Date().toISOString(),
        action: "boot",
        handle: "@adnanspitch",
        segment: "",
        variant: "",
        detail: "Fresh start. Account @adnanspitch ready. Waiting for Supabase connection.",
        result: "ok"
      }
    ],
    total: 1
  });
}
