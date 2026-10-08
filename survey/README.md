# Stakeholder Survey — hosting guide

Single-file app: `survey/index.html` (no build step). Responses save to
browser `localStorage`; coordinators export CSV/JSON and send to the team.

## Option A — instant on campus (recommended pilot)
```bash
python3 -m http.server 8000 --directory survey
# share http://<your-laptop-ip>:8000 with labs / classes
```

## Option B — free public link
- **GitHub Pages:** push repo → Settings → Pages → deploy `/survey`.
- **Netlify Drop:** drag the `survey/` folder to app.netlify.com/drop.
- **Cloudflare Pages / Vercel:** import repo, publish directory `survey`.

## Option C — later: POST to backend
`index.html` has a hook ready: POST `collect()` JSON to
`POST /api/v1/survey` (add table `survey_responses` + route when
`whatsapp-service` lands). Until then CSV/JSON export is the pipeline.

## Operations
- QR the hosted URL on department notice boards for 200L/300L rush coverage.
- Target: 100+ students, 20+ staff, 10+ stakeholder units before design freeze.
- Phone numbers are pilot opt-in only; keep warm CSVs 90–180d per retention plan.
