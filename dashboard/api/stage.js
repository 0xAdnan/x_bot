import { fetchRust } from './_rust.js';

export default async function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  let body = req.body || {};
  if (typeof body === 'string') {
    try { body = JSON.parse(body); } catch (e) {}
  }

  const resRust = await fetchRust('/api/prospect/stage', {
    method: 'POST',
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body)
  });

  if (resRust.ok) {
    return res.status(200).json(resRust.data);
  }

  return res.status(200).json({ status: "ok", stage: body.stage, id: body.id });
}
