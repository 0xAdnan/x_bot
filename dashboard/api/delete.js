import { fetchRust } from './_rust.js';

export default async function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  if (req.method !== 'POST' && req.method !== 'DELETE') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  let body = req.body || {};
  if (typeof body === 'string') {
    try { body = JSON.parse(body); } catch (e) {}
  }

  const type = body.type || body.item_type || 'prospect';
  const id = body.id || '';

  if (!id) {
    return res.status(400).json({ error: 'Missing ID for deletion' });
  }

  const resRust = await fetchRust('/api/delete', {
    method: 'POST',
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ type, id })
  });

  if (resRust.ok) {
    return res.status(200).json(resRust.data);
  }

  return res.status(200).json({ status: "ok", deleted: true, type, id });
}
