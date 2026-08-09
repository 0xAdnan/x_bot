export default function handler(req, res) {
  res.status(200).json({
    budget_raw: "X action budget for 2026-08-09\naction                used   cap  left\n--------------------------------------\nLikes                    0    12    12\nReplies                  0     3     3\nFollows                  0     3     3\nDMs (dm+followup)        0     2     2\nPosts                    0     1     1\nQuotes                   0     1     1\nDiscoveries              0    10    10\n\nactions in the last 60 min: 0 (burst cap 10)\n  COLD-START RAMP ACTIVE until 2026-08-30: caps are 25% of the normal limits.\n",
    circuit_breaker: "circuit-breaker: 0 trip(s) in the last 24h (limit 3).\nSTATUS: OK to run.",
    account: {
      handle: "@adnanspitch",
      product_handle: "@trypitchdotco",
      site: "https://trypitch.co",
      created: "2026-08-09",
      ramp_until: "2026-08-30"
    }
  });
}
