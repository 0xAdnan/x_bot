export default async function handler(req, res) {
  const supabaseUrl = process.env.SUPABASE_URL;
  const supabaseKey = process.env.SUPABASE_ANON_KEY || process.env.SUPABASE_SERVICE_ROLE_KEY;

  if (supabaseUrl && supabaseKey) {
    try {
      const response = await fetch(`${supabaseUrl}/rest/v1/insights?select=content&id=eq.1`, {
        headers: {
          'apikey': supabaseKey,
          'Authorization': `Bearer ${supabaseKey}`
        }
      });
      if (response.ok) {
        const data = await response.json();
        if (data && data.length > 0) {
          return res.status(200).json({ content: data[0].content });
        }
      }
    } catch (err) {
      console.error("Supabase Insights fetch error:", err);
    }
  }

  return res.status(200).json({
    content: "# Adaptive Insights\n\nWaiting for Supabase sync..."
  });
}
