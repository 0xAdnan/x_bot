import { fetchRust } from './_rust.js';

export default async function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  const resRust = await fetchRust('/api/stats');
  if (resRust.ok) {
    return res.status(200).json(resRust.data);
  }
  return res.status(200).json({
    status: 'ok',
    mention_jobs_total: 0,
    delivered: 0,
    rendering: 0,
    prospects_total: 0,
    agents: []
  });
}
