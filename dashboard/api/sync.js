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

  const makeSignature = (a) => {
    const ts = (a.ts || '').slice(0, 19); // Compare up to seconds
    const action = a.action || '';
    const handle = a.handle || '';
    const detail = (a.detail || '').replace(/\s+/g, ' ').trim();
    return `${ts}_${action}_${handle}_${detail}`;
  };

  // 1. GET: Clean/Deduplicate existing Supabase activities
  if (req.method === 'GET') {
    try {
      const fetchResp = await fetch(`${supabaseUrl}/rest/v1/activities?select=id,ts,action,handle,detail&order=id.asc`, { headers });
      if (!fetchResp.ok) {
        return res.status(500).json({ error: "Failed to fetch activities from Supabase" });
      }

      const activities = await fetchResp.json();
      const seen = new Set();
      const duplicateIds = [];

      for (const a of activities) {
        const key = makeSignature(a);
        if (seen.has(key)) {
          duplicateIds.push(a.id);
        } else {
          seen.add(key);
        }
      }

      // Delete duplicates in batches of 50
      let deletedCount = 0;
      if (duplicateIds.length > 0) {
        for (let i = 0; i < duplicateIds.length; i += 50) {
          const batch = duplicateIds.slice(i, i + 50);
          const delResp = await fetch(`${supabaseUrl}/rest/v1/activities?id=in.(${batch.join(',')})`, {
            method: 'DELETE',
            headers
          });
          if (delResp.ok) {
            deletedCount += batch.length;
          }
        }
      }

      return res.status(200).json({
        status: "ok",
        total_records_scanned: activities.length,
        duplicate_records_found: duplicateIds.length,
        deleted_count: deletedCount
      });
    } catch (err) {
      console.error("[Sync Clean Error]:", err);
      return res.status(500).json({ error: err.message });
    }
  }

  // 2. POST: Deduplicate and Sync incoming activities/prospects
  if (req.method === 'POST') {
    try {
      const body = req.body || {};
      const incomingActivities = Array.isArray(body) ? body : (body.activities || []);
      const incomingProspects = body.prospects || [];

      let syncedActivitiesCount = 0;
      let skippedActivitiesCount = 0;

      if (incomingActivities.length > 0) {
        const existingResp = await fetch(`${supabaseUrl}/rest/v1/activities?select=ts,action,handle,detail`, { headers });
        let existingSignatures = new Set();
        if (existingResp.ok) {
          const existingData = await existingResp.json();
          for (const item of existingData) {
            existingSignatures.add(makeSignature(item));
          }
        }

        const newActivities = [];
        for (const item of incomingActivities) {
          const sig = makeSignature(item);
          if (!existingSignatures.has(sig)) {
            newActivities.push(item);
            existingSignatures.add(sig);
          } else {
            skippedActivitiesCount++;
          }
        }

        if (newActivities.length > 0) {
          const insertResp = await fetch(`${supabaseUrl}/rest/v1/activities`, {
            method: 'POST',
            headers,
            body: JSON.stringify(newActivities)
          });
          if (insertResp.ok) {
            syncedActivitiesCount = newActivities.length;
          } else {
            const errText = await insertResp.text();
            console.error("[Sync Insert Error]:", errText);
          }
        }
      }

      // Sync Prospects if present
      let prospectsSynced = 0;
      if (incomingProspects.length > 0) {
        const prospectHeaders = { ...headers, 'Prefer': 'resolution=merge-duplicates' };
        const prospectResp = await fetch(`${supabaseUrl}/rest/v1/prospects`, {
          method: 'POST',
          headers: prospectHeaders,
          body: JSON.stringify(incomingProspects)
        });
        if (prospectResp.ok) {
          prospectsSynced = incomingProspects.length;
        }
      }

      return res.status(200).json({
        status: "ok",
        synced_activities: syncedActivitiesCount,
        skipped_duplicates: skippedActivitiesCount,
        prospects_synced: prospectsSynced
      });
    } catch (err) {
      console.error("[Sync POST Error]:", err);
      return res.status(500).json({ error: err.message });
    }
  }

  return res.status(405).json({ error: "Method not allowed" });
}
