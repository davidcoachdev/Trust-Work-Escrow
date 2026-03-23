# Database Schema
## Trust Work Escrow v2

---

## 1. PostgreSQL (Primary Database)

### 1.1 Users

```sql
-- Users cache (sincronizado con on-chain User PDA)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Wallet info (source of truth)
    wallet_principal TEXT UNIQUE NOT NULL,
    active_wallet TEXT,
    
    -- Profile
    username TEXT UNIQUE,
    bio TEXT,
    avatar_url TEXT,
    
    -- Stats (sync from on-chain)
    reputation_score DECIMAL(3,2) DEFAULT 0.00,
    jobs_completed INTEGER DEFAULT 0,
    disputes_won INTEGER DEFAULT 0,
    disputes_lost INTEGER DEFAULT 0,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_reputation CHECK (reputation_score >= 0 AND reputation_score <= 5)
);

CREATE INDEX idx_users_wallet ON users(wallet_principal);
CREATE INDEX idx_users_username ON users(username);
```

### 1.2 User Wallets

```sql
-- Múltiples wallets por usuario
CREATE TABLE user_wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wallet_address TEXT NOT NULL,
    wallet_label TEXT,
    provider TEXT NOT NULL DEFAULT 'phantom', -- phantom, solflare, backpack, ledger
    is_verified BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT unique_wallet_per_user UNIQUE (user_id, wallet_address),
    CONSTRAINT max_wallets_per_user CHECK (
        (SELECT COUNT(*) FROM user_wallets WHERE user_id = users.id) < 10
    )
);

CREATE INDEX idx_user_wallets_user ON user_wallets(user_id);
CREATE INDEX idx_user_wallets_address ON user_wallets(wallet_address);
```

### 1.3 Teams

```sql
-- Equipos/agencias
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- On-chain reference
    pda_address TEXT UNIQUE,
    
    -- Ownership
    owner_id UUID NOT NULL REFERENCES users(id),
    
    -- Info
    name TEXT NOT NULL,
    description TEXT,
    avatar_url TEXT,
    
    -- Stats
    total_earnings BIGINT DEFAULT 0,
    jobs_completed INTEGER DEFAULT 0,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_teams_owner ON teams(owner_id);
```

### 1.4 Team Members

```sql
-- Miembros de equipo
CREATE TABLE team_members (
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- Role in team
    role TEXT NOT NULL DEFAULT 'member', -- owner, lead, pm, developer, designer, qa, member
    department TEXT, -- frontend, backend, design, qa, management
    
    -- Payment
    payout_percentage INTEGER NOT NULL DEFAULT 0,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Primary key
    PRIMARY KEY (team_id, user_id),
    
    -- Constraints
    CONSTRAINT valid_role CHECK (
        role IN ('owner', 'lead', 'pm', 'developer', 'designer', 'qa', 'member')
    ),
    CONSTRAINT valid_percentage CHECK (
        payout_percentage >= 0 AND payout_percentage <= 100
    )
);

-- Check: owners can't have payout
CREATE FUNCTION check_owner_percentage() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.role = 'owner' AND NEW.payout_percentage > 0 THEN
        RAISE EXCEPTION 'Owners cannot have payout percentage';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_owner_percentage
    BEFORE INSERT OR UPDATE ON team_members
    FOR EACH ROW EXECUTE FUNCTION check_owner_percentage();

CREATE INDEX idx_team_members_user ON team_members(user_id);
```

### 1.5 Jobs

```sql
-- Jobs cache (sincronizado con on-chain Job PDA)
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- On-chain reference
    pda_address TEXT UNIQUE,
    
    -- Parties
    client_id UUID NOT NULL REFERENCES users(id),
    freelancer_id UUID REFERENCES users(id),
    team_id UUID REFERENCES teams(id), -- Si es un equipo
    arbiter_id UUID REFERENCES users(id),
    
    -- Job details
    title TEXT NOT NULL,
    description TEXT,
    category TEXT, -- web_development, mobile, design, etc.
    
    -- Amounts (en lamports)
    amount BIGINT NOT NULL,
    fee_amount BIGINT NOT NULL DEFAULT 0,
    deposited_amount BIGINT NOT NULL DEFAULT 0,
    released_amount BIGINT NOT NULL DEFAULT 0,
    
    -- Status (mirrors on-chain JobStatus)
    status TEXT NOT NULL DEFAULT 'draft' 
        CHECK (status IN (
            'draft', 'created', 'funded', 'in_progress', 
            'submitted', 'approved', 'disputed', 'resolved', 'cancelled'
        )),
    
    -- Timeline
    deadline TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    funded_at TIMESTAMPTZ,
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Search
    search_vector TSVECTOR GENERATED ALWAYS AS (
        to_tsvector('spanish', coalesce(title, '') || ' ' || coalesce(description, ''))
    ) STORED
);

CREATE INDEX idx_jobs_client ON jobs(client_id);
CREATE INDEX idx_jobs_freelancer ON jobs(freelancer_id);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_category ON jobs(category);
CREATE INDEX idx_jobs_search ON jobs USING GIN(search_vector);
CREATE INDEX idx_jobs_created ON jobs(created_at DESC);
```

### 1.6 Milestones

```sql
-- Hitos dentro de un job
CREATE TABLE milestones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    
    -- Order
    index_num INTEGER NOT NULL,
    
    -- Details
    title TEXT NOT NULL,
    description TEXT,
    
    -- Amount (en lamports)
    amount BIGINT NOT NULL,
    
    -- Status
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'submitted', 'approved', 'rejected')),
    
    -- Timeline
    deadline TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT unique_milestone_index UNIQUE (job_id, index_num)
);

CREATE INDEX idx_milestones_job ON milestones(job_id);
```

### 1.7 Disputes

```sql
-- Disputas
CREATE TABLE disputes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- References
    job_id UUID NOT NULL REFERENCES jobs(id),
    pda_address TEXT,
    
    -- Parties
    opened_by UUID NOT NULL REFERENCES users(id),
    assigned_to UUID REFERENCES users(id), -- Arbiter
    
    -- Details
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open'
        CHECK (status IN ('open', 'in_review', 'resolved')),
    
    -- Resolution
    freelancer_percentage INTEGER,
    client_percentage INTEGER,
    resolution_notes TEXT,
    resolved_by UUID REFERENCES users(id),
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX idx_disputes_job ON disputes(job_id);
CREATE INDEX idx_disputes_status ON disputes(status);
CREATE INDEX idx_disputes_opened_by ON disputes(opened_by);
```

### 1.8 AI Reports

```sql
-- Reports generados por IA
CREATE TABLE ai_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Reference
    dispute_id UUID REFERENCES disputes(id),
    
    -- Output
    output_json JSONB NOT NULL,
    
    -- Metadata
    model TEXT NOT NULL,
    model_version TEXT,
    confidence_score DECIMAL(3,2),
    
    -- Processing
    processing_time_ms INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_ai_reports_dispute ON ai_reports(dispute_id);
CREATE INDEX idx_ai_reports_confidence ON ai_reports(confidence_score);
```

### 1.9 Notifications

```sql
-- Notificaciones
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- Type
    type TEXT NOT NULL 
        CHECK (type IN (
            'job_created', 'job_funded', 'job_accepted', 
            'work_submitted', 'work_approved', 'work_rejected',
            'dispute_opened', 'dispute_resolved',
            'payment_received', 'system'
        )),
    
    -- Content
    title TEXT NOT NULL,
    message TEXT,
    data JSONB, -- Additional data (job_id, etc.)
    
    -- State
    is_read BOOLEAN DEFAULT FALSE,
    read_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_notifications_user ON notifications(user_id);
CREATE INDEX idx_notifications_user_unread ON notifications(user_id, is_read) 
    WHERE is_read = FALSE;
CREATE INDEX idx_notifications_created ON notifications(created_at DESC);
```

### 1.10 Audit Logs

```sql
-- Logs de auditoría (TODO)
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Transaction
    tx_signature TEXT UNIQUE,
    instruction TEXT NOT NULL,
    
    -- Actor
    actor_wallet TEXT NOT NULL,
    actor_user_id UUID REFERENCES users(id),
    
    -- Resource
    resource_type TEXT NOT NULL, -- job, user, team, etc.
    resource_id TEXT,
    
    -- Change
    action TEXT NOT NULL, -- create, update, delete
    old_value JSONB,
    new_value JSONB,
    
    -- Metadata
    ip_address INET,
    user_agent TEXT,
    
    -- Timestamp
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_tx ON audit_logs(tx_signature);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_wallet);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);
```

### 1.11 Financial Ledger

```sql
-- Ledger contable
CREATE TABLE financial_ledger (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- References
    job_id UUID REFERENCES jobs(id),
    user_id UUID REFERENCES users(id),
    
    -- Transaction
    tx_signature TEXT UNIQUE,
    
    -- Type
    type TEXT NOT NULL 
        CHECK (type IN (
            'job_deposit', 'job_fee', 'milestone_payment',
            'dispute_stake', 'dispute_refund', 'dispute_penalty',
            'treasury_withdrawal', 'rent_refund'
        )),
    
    -- Amounts (en lamports)
    gross_amount BIGINT NOT NULL,
    fee_amount BIGINT DEFAULT 0,
    net_amount BIGINT NOT NULL,
    
    -- Wallets
    from_wallet TEXT,
    to_wallet TEXT,
    
    -- Status
    status TEXT DEFAULT 'confirmed'
        CHECK (status IN ('pending', 'confirmed', 'failed')),
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_ledger_job ON financial_ledger(job_id);
CREATE INDEX idx_ledger_user ON financial_ledger(user_id);
CREATE INDEX idx_ledger_type ON financial_ledger(type);
CREATE INDEX idx_ledger_created ON financial_ledger(created_at DESC);
```

---

## 2. MongoDB (Secondary Database)

### 2.1 Chat Messages

```javascript
// Collection: chat_messages
{
  _id: ObjectId,
  
  // Context
  job_id: "uuid",
  dispute_id: "uuid", // null si no es dispute
  
  // Message
  sender_wallet: "pubkey_string",
  sender_type: "client" | "freelancer" | "system",
  
  // Content (E2EE encrypted)
  content: {
    ciphertext: "base64_string",
    iv: "base64_string",
    auth_tag: "base64_string",
    // La key de descifrado solo se comparte entre partes
  },
  
  // Metadata
  type: "text" | "file" | "system",
  attachments: [
    {
      filename: "report.pdf",
      hash: "Qm...",
      size: 1024000,
      mime_type: "application/pdf"
    }
  ],
  
  // State
  is_read: boolean,
  read_by: ["wallet1", "wallet2"],
  
  // Timestamps
  created_at: ISODate,
  updated_at: ISODate
}

// Indexes
db.chat_messages.createIndex({ job_id: 1, created_at: 1 });
db.chat_messages.createIndex({ dispute_id: 1, created_at: 1 });
```

### 2.2 Dispute Evidence

```javascript
// Collection: dispute_evidence
{
  _id: ObjectId,
  
  // Context
  dispute_id: "uuid",
  
  // Evidence
  type: "chat_snapshot" | "file" | "link" | "screenshot",
  
  // Content
  title: "Screenshot de conversación",
  description: "...",
  
  // Files
  files: [
    {
      filename: "screenshot.png",
      storage_url: "s3://...",
      hash: "Qm...",
      size: 500000,
      mime_type: "image/png"
    }
  ],
  
  // Source
  submitted_by: "wallet_string",
  submitted_at: ISODate,
  
  // For AI processing
  processed_by_ai: boolean,
  ai_summary: "..."
}

// Indexes
db.dispute_evidence.createIndex({ dispute_id: 1 });
```

### 2.3 API Logs

```javascript
// Collection: api_logs
{
  _id: ObjectId,
  
  // Request
  method: "POST",
  path: "/api/jobs/123/fund",
  headers: { ... },
  
  // Auth
  auth_wallet: "pubkey",
  auth_user_id: "uuid",
  
  // Response
  status_code: 200,
  response_time_ms: 145,
  
  // Error
  error: null | { message, stack },
  
  // Timestamps
  created_at: ISODate
}

// Indexes
db.api_logs.createIndex({ created_at: 1 });
db.api_logs.createIndex({ auth_wallet: 1 });
```

---

## 3. Redis (Cache & Sessions)

### 3.1 Sessions

```
# Key: session:{session_id}
# TTL: 7 days
{
  "user_id": "uuid",
  "wallet": "pubkey",
  "roles": ["client", "freelancer"],
  "created_at": "ISO8601"
}
```

### 3.2 Cache Keys

```
# Job cache (5 min TTL)
job:{id} = { ...job_data }

# User cache (5 min TTL)
user:{wallet} = { ...user_data }

# Token prices
price:SOL = 150.25
price:USDC = 1.00

# Rate limiting
ratelimit:{wallet}:{endpoint} = count
```

### 3.3 Pub/Sub Channels

```
# Real-time events
channel:user:{wallet}       # Events for specific user
channel:job:{job_id}       # Events for specific job
channel:notifications:{user_id}  # User notifications
```

---

## 4. Esquema de Relaciones

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            RELACIONES                                      │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  users ─────────────── user_wallets (1:N)                                │
│     │                                                                   │
│     ├── jobs (1:N) ─────────────── milestones (1:N)                     │
│     │                                                                   │
│     ├── teams (1:N) ─────── team_members (N:M) ──► users                │
│     │                                                                   │
│     └── disputes (1:N) ──────── ai_reports (1:N)                        │
│                                                                          │
│  jobs ─────────────── disputes (1:1)                                     │
│     │                                                                   │
│     └── notifications (1:N)                                             │
│                                                                          │
│  jobs ─────────────── financial_ledger (1:N)                             │
│                                                                          │
│  MongoDB:                                                               │
│  jobs ─────────────── chat_messages (1:N)                               │
│  disputes ─────────── dispute_evidence (1:N)                              │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

_Last updated: 2026-03-22_
