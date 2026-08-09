export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const supabaseUrl = process.env.SUPABASE_URL;
  const supabaseKey = process.env.SUPABASE_ANON_KEY || process.env.SUPABASE_SERVICE_ROLE_KEY;

  if (!supabaseUrl || !supabaseKey) {
    return res.status(500).json({ error: 'Missing Supabase credentials' });
  }

  try {
    const data = req.body;
    const prospect = {
      handle: data.handle || '',
      name: data.name || '',
      segment: data.segment || 'founder',
      stage: data.stage || 'new',
      product_url: data.product_url || '',
      notes: data.notes || '',
      score: Number(data.score) || 5,
      updated_at: new Date().toISOString()
    };

    const response = await fetch(`${supabaseUrl}/rest/v1/prospects`, {
      method: 'POST',
      headers: {
        'apikey': supabaseKey,
        'Authorization': `Bearer ${supabaseKey}`,
        'Content-Type': 'application/json',
        'Prefer': 'resolution=merge-duplicates'
      },
      body: JSON.stringify([prospect])
    });

    if (response.ok) {
      return res.status(200).json({ success: true, prospect });
    } else {
      const errText = await response.text();
      return res.status(400).json({ error: errText });
    }
  } catch (err) {
    console.error("Add prospect error:", err);
    return res.status(500).json({ error: err.message });
  }
}
