export default async function handler(req, res) {
  const supabaseUrl = process.env.SUPABASE_URL || "https://jwswpryozfxzaocimadp.supabase.co";
  const supabaseKey = process.env.SUPABASE_ANON_KEY || "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imp3c3dwcnlvemZ4emFvY2ltYWRwIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODYyNzQzMjQsImV4cCI6MjEwMTg1MDMyNH0.U575XIPsA12Y3JpZJ_gr9T7xH4WZafihkThQJvh8VNo";
  const pitchApiKey = process.env.PITCH_API_KEY || "pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB";

  if (req.method === 'DELETE' || req.method === 'POST') {
    const { type, id } = req.body || req.query || {};

    if (!type || !id) {
      return res.status(400).json({ error: "Missing type or id" });
    }

    let tableName = "";
    if (type === 'activity') tableName = "activities";
    else if (type === 'mention_job') tableName = "mention_jobs";
    else if (type === 'prospect') tableName = "prospects";
    else return res.status(400).json({ error: "Invalid entity type" });

    try {
      // 1. If deleting a mention job, fetch job details first to send cancellation/deletion to Pitch MCP
      if (type === 'mention_job') {
        const fetchUrl = `${supabaseUrl}/rest/v1/mention_jobs?id=eq.${id}`;
        const fetchResp = await fetch(fetchUrl, {
          headers: {
            "apikey": supabaseKey,
            "Authorization": `Bearer ${supabaseKey}`
          }
        });

        if (fetchResp.ok) {
          const jobs = await fetchResp.json();
          if (jobs && jobs.length > 0) {
            const job = jobs[0];
            const jobId = job.editor_job_id;
            const status = (job.status || '').toLowerCase();

            if (jobId) {
              const actionMethod = (status === 'rendering' || status === 'processing' || status === 'pending') ? 'cancel_job' : 'delete_job';
              console.log(`[Pitch MCP] Sending ${actionMethod} for Job ID ${jobId}...`);

              // Send MCP tool call
              fetch("https://api.trypitch.co/mcp", {
                method: "POST",
                headers: {
                  "Authorization": `Bearer ${pitchApiKey}`,
                  "Content-Type": "application/json",
                  "Accept": "application/json, text/event-stream"
                },
                body: JSON.stringify({
                  jsonrpc: "2.0",
                  id: Date.now(),
                  method: "tools/call",
                  params: {
                    name: actionMethod,
                    arguments: { jobId }
                  }
                })
              }).catch(err => console.error(`[Pitch MCP ${actionMethod} Error]:`, err));
            }
          }
        }
      }

      // 2. Direct REST API DELETE call to Supabase
      const deleteUrl = `${supabaseUrl}/rest/v1/${tableName}?id=eq.${id}`;
      const response = await fetch(deleteUrl, {
        method: "DELETE",
        headers: {
          "apikey": supabaseKey,
          "Authorization": `Bearer ${supabaseKey}`,
          "Content-Type": "application/json"
        }
      });

      if (response.ok) {
        return res.status(200).json({ status: "ok", deletedId: id });
      } else {
        const errorText = await response.text();
        console.error(`[Supabase Delete Error]:`, errorText);
        return res.status(500).json({ error: errorText });
      }
    } catch (err) {
      console.error(`[Delete Exception]:`, err);
      return res.status(500).json({ error: err.message });
    }
  }

  return res.status(405).json({ error: "Method not allowed" });
}
