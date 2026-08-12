export default async function handler(req, res) {
  const rustUrl = process.env.LOCAL_RUST_SERVER_URL || "https://fruity-corners-crash.loca.lt";
  try {
    const resp = await fetch(`${rustUrl}/api/mentions`, {
      headers: {
        "Bypass-Tunnel-Remainder": "true",
        "bypass-tunnel-reminder": "true",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
      }
    });
    if (resp.ok) {
      const data = await resp.json();
      return res.status(200).json(data);
    }
  } catch (e) {
    console.error("[Mentions Direct Rust Local SQLite Error]:", e);
  }
  return res.status(200).json({ jobs: [], total: 0 });
}
