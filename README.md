# Next-Gen University ERP-LMS — Newgate University Minna
Lead Engineer & Architect: Abdullahi Baba Garba
Dept. of Cyber Security Science, Faculty of Computing and IT.

100% Rust (Axum) microservices mesh + Vue 3 + TypeScript frontend + local Ollama SLMs.

## Layout
```text
crates/common/            # shared domain logic: cohort, progression, RBAC, JWT, money, matric numbers
services/api-gateway/     # JWT auth + Redis sliding-window rate limit + reverse proxy routing
services/academic-service/# NUC engine: cohorts, carryover, prerequisites, CGPA/spillover
services/admissions-service/
services/financial-service/# rust_decimal ledgers + Paystack/Remita HMAC-SHA512 webhooks
services/hospital-service/# isolated PG + AES-256-GCM field encryption
services/ai-secops-service/# llama3.2:3b tutor/copilot + qwen2.5:1.5b SecOps JSON scoring
frontend/                 # Vue 3 + TS (scaffold in Week 3)
docker-compose.yml        # postgres-main, postgres-hospital, redis, rabbitmq, mongodb, ollama
```

## Quickstart
```bash
cp .env.example .env
docker compose up -d postgres-main postgres-hospital redis rabbitmq mongodb
cargo test --workspace
cargo run -p api-gateway
docker compose --profile ai up -d ollama  # pulls llama3.2:3b + qwen2.5:1.5b on 8GB box
```

## NUC rules encoded in `crates/common`
- Cohort binding `YYYY-Semester1|Semester2`
- Carryover F-lock, prerequisite block
- CGPA >= 1.00 good standing, < 1.00 repeat/probation, spillover after normal duration, 1.5x cap → Senate review
