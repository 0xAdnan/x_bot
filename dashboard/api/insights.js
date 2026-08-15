import { fetchRust } from './_rust.js';

export default async function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  const resRust = await fetchRust('/api/insights');
  if (resRust.ok) {
    return res.status(200).json(resRust.data);
  }
  return res.status(200).json({ content: "No insights recorded yet." });
}
