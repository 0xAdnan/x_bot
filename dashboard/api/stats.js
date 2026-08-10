export default async function handler(req, res) {
  const supabaseUrl = process.env.SUPABASE_URL || "https://jwswpryozfxzaocimadp.supabase.co";
  const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY || process.env.SUPABASE_ANON_KEY;

  let heartbeats = {};

  if (supabaseUrl && supabaseKey) {
    try {
      const headers = { 'apikey': supabaseKey, 'Authorization': `Bearer ${supabaseKey}` };
      const resHb = await fetch(`${supabaseUrl}/rest/v1/activities?action=eq.heartbeat&order=ts.desc&limit=20`, { headers }).then(r => r.json()).catch(() => []);
      
      if (Array.isArray(resHb)) {
        for (const item of resHb) {
          const seg = item.segment;
          if (seg && !heartbeats[seg]) {
            heartbeats[seg] = new Date(item.ts).getTime();
          }
        }
      }
    } catch (e) {}
  }

  const nowEpoch = Date.now();
  const daemonTimeoutMs = 60000; // 60 seconds threshold

  const daemonHeartbeatActive = (seg) => {
    const lastTs = heartbeats[seg];
    if (!lastTs) return false;
    return (nowEpoch - lastTs) <= daemonTimeoutMs;
  };

  const mentionDaemonActive = daemonHeartbeatActive('mention_daemon');
  const workerDaemonActive = daemonHeartbeatActive('mention_mcp_worker');

  const agents = [
    {
      id: "mention_daemon",
      name: "Mention Scanner",
      type: "10s Real-Time Poller",
      target: "x.com/notifications + /search",
      status: mentionDaemonActive ? "active" : "stopped",
      uptime: mentionDaemonActive ? "Continuous 10s Loop" : "Offline",
      last_pulse: heartbeats['mention_daemon'] ? new Date(heartbeats['mention_daemon']).toISOString() : "None",
      description: mentionDaemonActive ? "Scans notifications & search every 10s for @trypitchdotco mentions." : "Daemon process is stopped. Mention scanning paused."
    },
    {
      id: "mention_mcp_worker",
      name: "Pitch MCP Worker",
      type: "Queue & Video Generator",
      target: "Supabase <-> Pitch MCP API",
      status: workerDaemonActive ? "active" : "stopped",
      uptime: workerDaemonActive ? "Continuous 10s Loop" : "Offline",
      last_pulse: heartbeats['mention_mcp_worker'] ? new Date(heartbeats['mention_mcp_worker']).toISOString() : "None",
      description: workerDaemonActive ? "Claims pending jobs, triggers Pitch MCP rendering, and posts X replies." : "Worker daemon is stopped. Video generation queue paused."
    },
    {
      id: "hermes_gateway",
      name: "Hermes Gateway",
      type: "Scheduler Daemon",
      target: "Background Agent Engine",
      status: "active",
      uptime: "Active Gateway",
      last_pulse: new Date().toISOString(),
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

  const activeDaemonsCount = agents.filter(a => a.status === 'active').length;

  return res.status(200).json({
    status: "ok",
    timestamp: new Date().toISOString(),
    agents,
    total_agents: agents.length,
    active_daemons_count: activeDaemonsCount,
    scheduled_cron_count: 2
  });
}
