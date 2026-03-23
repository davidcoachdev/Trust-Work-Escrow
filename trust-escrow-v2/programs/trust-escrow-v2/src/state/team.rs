use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MemberRole {
    Owner,
    ProjectManager,
    Contributor,
}

impl anchor_lang::Space for MemberRole {
    const INIT_SPACE: usize = 1;
}

#[account]
#[derive(InitSpace)]
pub struct Team {
    pub owner: Pubkey,
    #[max_len(32)]
    pub name: String,
    #[max_len(256)]
    pub description: String,
    #[max_len(20)]
    pub members: Vec<Member>,
    pub total_percentage: u8,
    pub bump: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Member {
    pub user: Pubkey,
    pub role: MemberRole,
    pub percentage: u8,
    pub joined_at: i64,
}

impl anchor_lang::Space for Member {
    const INIT_SPACE: usize = 32 + 1 + 1 + 8;
}
