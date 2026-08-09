export default async function handler(req, res) {
  const supabaseUrl = process.env.SUPABASE_URL;
  const supabaseKey = process.env.SUPABASE_ANON_KEY || process.env.SUPABASE_SERVICE_ROLE_KEY;

  if (supabaseUrl && supabaseKey) {
    try {
      const response = await fetch(`${supabaseUrl}/rest/v1/mention_jobs?select=*&order=updated_at.desc&limit=50`, {
        headers: {
          'apikey': supabaseKey,
          'Authorization': `Bearer ${supabaseKey}`
        }
      });
      if (response.ok) {
        const jobs = await response.json();
        return res.status(200).json({ jobs, total: jobs.length });
      }
    } catch (err) {
      console.error("Supabase mention jobs fetch error:", err);
    }
  }

  // Fallback demo data
  return res.status(200).json({
    jobs: [
      {
        id: 1,
        tweet_id: "1821949182310123847",
        user_handle: "@l0xding",
        target_url: "https://trypitch.co",
        editor_job_id: "cms36syrs0015ph4dkxpvl1d7",
        status: "delivered",
        s3_video_url: "https://s3.trypitch.co/trypitch/pitch/user_3DRBLAInFjCbFpzLUKZpcixGSye/try_pitch_co/videos/1785418492656-mv4ep0/final_with_cards.mp4",
        x_reply_id: "1821950123849102931",
        updated_at: new Date().toISOString()
      },
      {
        id: 2,
        tweet_id: "1821952000000000000",
        user_handle: "@l0xding",
        target_url: "https://tella.com",
        editor_job_id: "cms36tella0015ph4dkxpvl999",
        status: "rendering",
        s3_video_url: "",
        x_reply_id: "",
        updated_at: new Date().toISOString()
      }
    ],
    total: 2
  });
}
