export default function handler(req, res) {
  res.status(200).json({
    prospects: [],
    stages: {
      new: [],
      warming: [],
      contacted: [],
      in_convo: [],
      trial: [],
      customer: [],
      "do-not-contact": [],
      lost: []
    },
    total: 0
  });
}
