# frontend/src/api — Integración Backend v3

Zustand es la **fuente de la verdad**; esta carpeta es donde Zustand consume la API para hablar con el backend, que a su vez usa el SDK para interactuar con el contrato.

## Estructura (root métodos + carpeta por método + archivo por endpoint)

```
frontend/src/api/
  client.ts              # root: API_URL, ApiError, apiFetch, apiJson, parseApiError
  types.ts               # tipos compartidos on-chain/off-chain (Job, JobResponse, mapJobResponse, validations)
  index.ts               # barrel que re-exporta todo + namespaces
  jobs/                  # método jobs (on-chain Vec<Job> + off-chain metadata title/description)
    list.ts              # GET /jobs
    get.ts               # GET /jobs/:id
    create.ts            # POST /jobs
    deposit.ts           # POST /jobs/:id/deposit
    cancel.ts            # POST /jobs/:id/cancel
    pause.ts             # POST /jobs/:id/pause
    unpause.ts           # POST /jobs/:id/unpause
    submitWork.ts        # POST /jobs/:id/submit-work
    approveWork.ts       # POST /jobs/:id/approve-work
    rejectWork.ts        # POST /jobs/:id/reject-work
    index.ts
  applications/
    apply.ts             # POST /jobs/:id/apply (proposal_hash on-chain, proposal off-chain)
    accept.ts            # POST /jobs/:id/applications/:index/accept
  milestones/
    create.ts            # POST /jobs/:id/milestones
    submit.ts            # POST /jobs/:id/milestones/:idx/submit
    approve.ts           # POST /jobs/:id/milestones/:idx/approve
    reject.ts            # POST /jobs/:id/milestones/:idx/reject
  disputes/
    raise.ts             # POST /jobs/:id/disputes
    accept.ts            # POST /jobs/:id/disputes/accept
    evidence.ts          # POST /jobs/:id/disputes/evidence (content_hash + content off-chain)
    assignArbiter.ts     # POST /jobs/:id/disputes/assign-arbiter
    resolve.ts           # POST /jobs/:id/disputes/resolve {client_payout_percent}
    platformResolve.ts   # POST /jobs/:id/disputes/platform-resolve
    requestIntervention.ts
    finalize.ts          # POST /jobs/:id/disputes/finalize
  support/
    open.ts              # POST /jobs/:id/support
    resolve.ts           # POST /jobs/:id/support/resolve
  arbiterPool/
    get.ts               # GET /arbiter-pool
    create.ts            # POST /arbiter-pool
    add.ts               # POST /arbiter-pool/arbiters
    remove.ts            # DELETE /arbiter-pool/arbiters/:arbiter
  config/
    get.ts               # GET /config
  auth/
    verify.ts            # POST /auth/verify
  health/
    check.ts             # GET /health | /live | /ready
```

## Flujo on-chain / off-chain

- **On-chain (SDK / programa 7a2Y…)**: `Vec<Job>` indexado por `jobId`, `Application` PDA, `Milestone` PDA, `Dispute` PDA, `ArbiterPool`.
- **Off-chain (backend Postgres/Mongo via MetadataRepository)**: `title`, `description`, `proposal` (texto), `evidence content`. El backend expone título/descripción ya fusionados en `JobResponse`; `api/types.ts#mapJobResponse` normaliza `Created→Open` etc.
- **Frontend**: stores Zustand llaman a `api/<dominio>/<endpoint>.ts` → `apiFetch` → backend Axum → SDK Rust → programa Anchor.

## Stores Zustand (fuente de la verdad)

```
frontend/src/stores/
  useJobStore.ts          # jobs, currentJob, fetchJobs/fetchJob/createJob/deposit/cancel/pause/...
  useApplicationStore.ts  # apply, accept
  useMilestoneStore.ts    # create/submit/approve/reject
  useDisputeStore.ts      # raise/accept/evidence/assign/resolve/...
  useSupportStore.ts      # open/resolve
  useConfigStore.ts       # config + arbiterPool
```

Cada store tiene `{ loading, error, clearError, reset }` + acciones async que delegan a `api/*`. Los componentes (`app/jobs/page.tsx`, `app/create/page.tsx`, `app/jobs/[id]/page.tsx`) consumen el store, no `fetch` directo.

## Stack

Next.js 16 (app router, turbopack) + Zustand 5 + `NEXT_PUBLIC_API_URL` (default `http://127.0.0.1:3000`) + `@solana/wallet-adapter`.
