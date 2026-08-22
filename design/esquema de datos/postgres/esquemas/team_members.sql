-- team_members: miembros de un equipo
CREATE TABLE team_members (
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member'
        CHECK (role IN ('owner','lead','pm','developer','designer','qa','member')),
    department TEXT,                                   -- frontend, backend, design, qa, management
    payout_percentage INTEGER NOT NULL DEFAULT 0
        CHECK (payout_percentage >= 0 AND payout_percentage <= 100),
    is_active BOOLEAN DEFAULT TRUE,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    left_at TIMESTAMPTZ,
    PRIMARY KEY (team_id, user_id)
);
CREATE INDEX idx_team_members_user ON team_members(user_id);
