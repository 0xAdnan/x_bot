export default function handler(req, res) {
  res.status(200).json({
    activities: [
      {
        ts: "2026-08-09T00:00:00Z",
        action: "boot",
        handle: "@adnanspitch",
        segment: "",
        variant: "",
        detail: "Fresh start. New account @adnanspitch replaces @adnanxpitch. CRM initialized.",
        result: "ok"
      }
    ],
    total: 1
  });
}
