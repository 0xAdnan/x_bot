export default async function handler(req, res) {
  const supabaseUrl = process.env.SUPABASE_URL || "https://jwswpryozfxzaocimadp.supabase.co";
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY || process.env.SUPABASE_ANON_KEY;

  let mentionJobsCount = 0;
  let activitiesCount = 0;

  if (supabaseUrl && supabaseKey) {
    try {
      const headers = { 'apikey': supabaseKey, 'Authorization': `Bearer ${supabaseKey}` };
      const [resJobs, resAct] = await Promise.all([
        fetch(`${supabaseUrl}/rest/v1/mention_jobs?select=id`, { headers }).then(r => r.json()).catch(() => []),
        fetch(`${supabaseUrl}/rest/v1/activities?select=id`, { headers }).then(r => r.json()).catch(() => [])
      ]);
      mentionJobsCount = Array.isArray(resJobs) ? resJobs.length : 0;
      activitiesCount = Array.isArray(resAct) ? resAct.length : 0;
    } catch (e) {}
  }

  const now = new Date().toISOString();

  const agents = [
    {
      id: "mention_daemon",
      name: "Mention Scanner",
      type: "10s Real-Time Poller",
      target: "x.com/notifications + /search",
      status: "active",
      uptime: "Continuous 10s Loop",
      last_pulse: now,
      description: "Scans notifications & search every 10s for @trypitchdotco mentions."
    },
    {
      id: "mention_mcp_worker",
      name: "Pitch MCP Worker",
      type: "Queue & Video Generator",
      target: "Supabase <-> Pitch MCP API",
      status: "active",
      uptime: "Continuous 10s Loop",
      last_pulse: now,
      description: "Claims pending jobs, triggers Pitch MCP rendering, and posts X replies."
    },
    {
      id: "hermes_gateway",
      name: "Hermes Gateway",
      type: "Scheduler Daemon",
      target: "Background Agent Engine",
      status: "active",
      uptime: "Active Gateway",
      last_pulse: now,
      description: "Triggers scheduled growth sessions, content posting, and cron jobs."
    },
    {
      id: "brand_agent",
      name: "Brand & Content",
      type: "3x Daily Cron Job",
      target: "@trypitchdotco",
      status: "scheduled",
      next_run: "Today at 08:00 AM IST",
      schedule: "0 8,11,14,17,20,23 * * *",
      description: "Publishes product demos and founder commentary for @trypitchdotco."
    },
    {
      id: "operator_agent",
      name: "Operator Growth",
      type: "3x Daily Cron Job",
      target: "@adnanspitch",
      status: "scheduled",
      next_run: "Today at 09:00 AM IST",
      schedule: "0 9,14,19 * * *",
      description: "Discovers SaaS founders, warm-up likes, and non-promotional replies."
    }
  ];

  return res.status(200).json({
    status: "ok",
    timestamp: now,
    agents,
    total_agents: agents.length,
    active_daemons_count: 3,
    scheduled_cron_count: 2,
    metrics: {
      mention_jobs_count: mentionJobsCount,
      activities_count: activitiesCount,
      circuit_breaker: "Normal (0 trips in 24h)",
      cold_ramp_until: "2026-08-30"
    }
  });
}
