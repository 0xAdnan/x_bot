import crypto from 'crypto';

const CORRECT_PASSWORD = process.env.DASHBOARD_PASSWORD || "pitch@123";

export default async function handler(req, res) {
  const isHttps = req.headers['x-forwarded-proto'] === 'https' || req.headers['referer']?.startsWith('https:');
  const cookieFlags = `Path=/; HttpOnly; SameSite=Lax; ${isHttps ? 'Secure;' : ''} Max-Age=2592000`;

  if (req.method === 'POST') {
    let body = req.body;

    // Parse body if stream or string
    if (!body) {
      try {
        body = await new Promise((resolve) => {
          let data = '';
          req.on('data', chunk => data += chunk);
          req.on('end', () => {
            try { resolve(JSON.parse(data)); } catch (e) { resolve({}); }
          });
        });
      } catch (e) {
        body = {};
      }
    } else if (typeof body === 'string') {
      try {
        body = JSON.parse(body);
      } catch (e) {
        body = {};
      }
    }

    const inputPassword = body?.password || req.query?.password || req.headers['x-dashboard-password'];
    
    if (inputPassword === CORRECT_PASSWORD) {
      const token = crypto.createHmac('sha256', CORRECT_PASSWORD).update("authenticated_user_session").digest('hex');
      
      res.setHeader('Set-Cookie', `pitch_auth_session=${token}; ${cookieFlags}`);
      
      return res.status(200).json({ status: "ok", authenticated: true, token });
    }
    
    return res.status(401).json({ status: "error", message: "Invalid Password" });
  }

  if (req.method === 'GET') {
    const cookies = req.headers.cookie || '';
    const authHeader = req.headers.authorization || req.headers['x-auth-token'] || req.query?.token || '';
    const expectedToken = crypto.createHmac('sha256', CORRECT_PASSWORD).update("authenticated_user_session").digest('hex');
    
    if (cookies.includes(`pitch_auth_session=${expectedToken}`) || authHeader.includes(expectedToken)) {
      return res.status(200).json({ authenticated: true });
    }
    
    return res.status(200).json({ authenticated: false });
  }

  return res.status(405).json({ error: "Method not allowed" });
}
