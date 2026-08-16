export async function fetchRust(path, options = {}) {
  const candidates = [
    process.env.LOCAL_RUST_SERVER_URL,
    "https://moody-earwig-79.loca.lt",
    "https://pitch-bot-adnan.loca.lt",
    "https://heavy-cougar-57.loca.lt",
    "https://perfect-termite-69.loca.lt"
  ].filter(Boolean);

  const sep = path.includes('?') ? '&' : '?';
  const cleanPath = `${path}${sep}bypass-tunnel-reminder=true`;

  for (const baseUrl of candidates) {
    try {
      const controller = new AbortController();
      const timeoutMs = options.timeout || 8000;
      const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
      const resp = await fetch(`${baseUrl}${cleanPath}`, {
        ...options,
        signal: controller.signal,
        headers: {
          "Bypass-Tunnel-Remainder": "true",
          "bypass-tunnel-reminder": "true",
          "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
          ...(options.headers || {})
        }
      });
      clearTimeout(timeoutId);
      const text = await resp.text();
      const trimmed = text.trim();
      if (resp.ok && (trimmed.startsWith('{') || trimmed.startsWith('['))) {
        const data = JSON.parse(trimmed);
        return { ok: true, data, baseUrl };
      }
    } catch (e) {}
  }
  return { ok: false };
}
