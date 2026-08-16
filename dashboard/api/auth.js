export default async function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization, x-auth-token');

  if (req.method === 'OPTIONS') {
    return res.status(200).end();
  }

  if (req.method === 'GET') {
    const authHeader = req.headers['x-auth-token'] || req.headers['authorization'] || '';
    const cookie = req.headers['cookie'] || '';
    if (authHeader.includes('pitch_auth_token_verified') || cookie.includes('pitch_auth=')) {
      return res.status(200).json({ authenticated: true, status: 'ok' });
    }
    return res.status(200).json({ authenticated: false, status: 'unauthorized' });
  }

  if (req.method === 'POST') {
    let body = req.body || {};
    if (typeof body === 'string') {
      try { body = JSON.parse(body); } catch (e) {}
    }

    const rawPassword = (body.password || '').trim();

    if (rawPassword === 'pitch@123' || rawPassword === 'pitch123') {
      const token = 'pitch_auth_token_verified_100';
      res.setHeader('Set-Cookie', `pitch_auth=${token}; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=2592000`);
      return res.status(200).json({ status: 'ok', authenticated: true, token, message: 'Authenticated successfully' });
    }

    return res.status(401).json({ authenticated: false, error: 'Invalid password' });
  }

  return res.status(405).json({ error: 'Method not allowed' });
}
