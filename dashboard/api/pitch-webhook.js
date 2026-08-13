import { fetchRust } from './_rust.js';

export default async function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  if (req.method === 'GET') {
    return res.status(200).json({ status: "Pitch MCP Webhook Receiver Active", url: "https://dashboard-blue-five-75.vercel.app/api/pitch-webhook" });
  }

  let body = req.body || {};
  if (typeof body === 'string') {
    try { body = JSON.parse(body); } catch (e) {}
  }

  const resRust = await fetchRust('/api/trigger', {
    method: 'POST',
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body)
  });

  if (resRust.ok) {
    return res.status(200).json(resRust.data);
  }

  return res.status(200).json({ status: "received", body });
}
