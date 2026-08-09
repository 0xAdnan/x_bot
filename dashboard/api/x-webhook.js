import crypto from 'crypto';

export default async function handler(req, res) {
  // Account Activity API CRC uses Consumer Secret (X_API_SECRET)
  const consumerSecret = process.env.X_API_SECRET || "X5bBoeop8QDCNzMuVXS2b2uJJ8YPIH2JPRVn0InDXRX1oVNd5l";

  // 1. X Webhook CRC (Challenge Response Check) for Registration
  if (req.method === 'GET') {
    const crcToken = req.query.crc_token;
    if (crcToken) {
      const hmac = crypto.createHmac('sha256', consumerSecret).update(crcToken).digest('base64');
      return res.status(200).json({ response_token: `sha256=${hmac}` });
    }
    return res.status(200).json({ status: "X Webhook Endpoint Active", url: "https://dashboard-blue-five-75.vercel.app/api/x-webhook" });
  }

  // 2. Incoming Mention Webhook Event (POST)
  if (req.method === 'POST') {
    try {
      const body = req.body;

      // Extract tweet create events
      if (body.tweet_create_events && Array.isArray(body.tweet_create_events)) {
        for (const tweet of body.tweet_create_events) {
          const userHandle = `@${tweet.user?.screen_name || ''}`;
          const tweetId = tweet.id_str;
          const text = tweet.text || '';

          // Skip tweets posted by @trypitchdotco itself
          if (userHandle.toLowerCase() === '@trypitchdotco') continue;

          console.log(`[X Webhook] Incoming tweet from ${userHandle}: ${text}`);

          // Extract URL or domain name
          const urlMatch = text.match(/https?:\/\/[^\s]+/) || text.match(/([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:\/[^\s]*)?)/);
          
          if (urlMatch) {
            let targetUrl = urlMatch[0];
            if (!targetUrl.startsWith('http://') && !targetUrl.startsWith('https://')) {
              targetUrl = `https://${targetUrl}`;
            }

            console.log(`[X Webhook] Extracted target URL: ${targetUrl} for user: ${userHandle}`);

            // A. Post Instant Acknowledgement Reply via X API
            const xUserToken = process.env.X_USER_ACCESS_TOKEN || "ZDdibjRaZHJ5Y1BQbDM2ei1jM01tOVZoazR1VVdMVDR4eTVkdjRHRGs0V3lxOjE3ODYzMDY3MzA4Nzg6MTowOmF0OjE";
            const ackText = `Cool ${userHandle}, we're on it! 🚀 Generating your cinematic demo for ${targetUrl} now, we'll get back to you with the video link right here soon!`;
            
            fetch("https://api.twitter.com/2/tweets", {
              method: "POST",
              headers: {
                "Authorization": `Bearer ${xUserToken}`,
                "Content-Type": "application/json"
              },
              body: JSON.stringify({
                text: ackText,
                reply: { in_reply_to_tweet_id: tweetId }
              })
            }).catch(err => console.error("[Ack Error]:", err));

            // B. Call Pitch MCP API with exact schema parameters
            const pitchApiKey = process.env.PITCH_API_KEY || "pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB";
            const mcpResp = await fetch("https://api.trypitch.co/mcp", {
              method: "POST",
              headers: {
                "Authorization": `Bearer ${pitchApiKey}`,
                "Content-Type": "application/json",
                "Accept": "application/json, text/event-stream"
              },
              body: JSON.stringify({
                jsonrpc: "2.0",
                id: Date.now(),
                method: "tools/call",
                params: {
                  name: "create_demo_video",
                  arguments: {
                    url: targetUrl,
                    instructions: `Create a cinematic, polished product demo of ${targetUrl}. Highlight key features, value proposition, and user experience.`,
                    voice: "Charon",
                    subtitles: false,
                    theme: "light",
                    background: "ocean",
                    shape: "rounded",
                    inset: "0.75",
                    browserHeader: "light"
                  }
                }
              })
            });

            if (mcpResp.ok) {
              console.log(`[X Webhook] Pitch MCP video job triggered successfully for ${targetUrl}!`);
            }
          }
        }
      }

      return res.status(200).json({ status: "ok" });
    } catch (err) {
      console.error("[X Webhook Error]:", err);
      return res.status(200).json({ status: "error", error: err.message });
    }
  }

  return res.status(405).json({ error: "Method not allowed" });
}
