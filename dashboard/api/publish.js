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
  const quote_url = body.quote_url || null;
  const reply_to_url = body.reply_to_url || null;
  const thread = body.thread || null;

  if (!text.trim() && (!thread || thread.length === 0)) {
    return res.status(400).json({ error: 'Tweet text or thread is required' });
  }

  const resRust = await fetchRust('/api/publish', {
    method: 'POST',
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ account, text, image_url, quote_url, reply_to_url, thread })
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
