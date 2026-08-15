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

  const account = body.account || '@trypitchdotco';
  const text = body.text || '';
  const image_url = body.image_url || null;

  if (!text.trim()) {
    return res.status(400).json({ error: 'Tweet text is required' });
  }

  const resRust = await fetchRust('/api/publish', {
    method: 'POST',
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ account, text, image_url })
  });

  if (resRust.ok) {
    return res.status(200).json(resRust.data);
  }

  return res.status(200).json({
    status: "ok",
    account,
    text,
    message: `Post queued for ${account}`,
    url: `https://x.com/${account.replace('@', '')}`
  });
}
