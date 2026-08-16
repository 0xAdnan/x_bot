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

  const { to_email, subject, body: emailBody, handle } = body;
  if (!to_email) {
    return res.status(400).json({ error: 'Recipient email is required' });
  }

  const resRust = await fetchRust('/api/outreach/email', {
    method: 'POST',
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ to_email, subject, body: emailBody, handle }),
    timeout: 12000
  });

  if (resRust.ok) {
    return res.status(200).json(resRust.data);
  }

  return res.status(500).json({
    status: "error",
    message: resRust.data?.message || "Failed to dispatch email. Please ensure SMTP credentials (SMTP_HOST, SMTP_USER, SMTP_PASS) are configured in .env."
  });
}
