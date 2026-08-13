import { fetchRust } from './_rust.js';

export default async function handler(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*');
  const resRust = await fetchRust('/api/research');
  if (resRust.ok) {
    return res.status(200).json(resRust.data);
  }
  return res.status(200).json({
    queries: [
      { query: '"loom alternative" OR "tella.tv alternative"', category: 'Competitor Mentions', priority: 'High' },
      { query: '"need a demo video" OR "how to make a product demo"', category: 'High Intent', priority: 'High' },
      { query: 'YC W26 OR "launching on product hunt"', category: 'SaaS Launches', priority: 'Medium' }
    ],
    scoringRules: [
      { rule: 'Founder / CEO / CTO in bio', description: 'Target ICP decision maker', points: '+3' },
      { rule: 'Explicit demo video request', description: 'Immediate high intent', points: '+4' },
      { rule: 'Has live product URL', description: 'Can generate automated video', points: '+3' }
    ],
    contentQueue: []
  });
}
