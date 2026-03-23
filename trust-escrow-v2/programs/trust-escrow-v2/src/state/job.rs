use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Created,
    ApplicationsOpen,
    InProgress,
    Submitted,
    Approved,
    Disputed,
    Cancelled,
}

impl anchor_lang::Space for JobStatus {
    const INIT_SPACE: usize = 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ApplicationStatus {
    Pending,
    Accepted,
    Rejected,
    Withdrawn,
}

impl anchor_lang::Space for ApplicationStatus {
    const INIT_SPACE: usize = 1;
}

#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,
    #[max_len(64)]
    pub title: String,
    #[max_len(1024)]
    pub description: String,
    pub amount: u64,
    pub entry_fee: u64,
    pub total_deposited: u64,
    pub deadline: i64,
    pub status: JobStatus,
    pub freelancer: Option<Pubkey>,
    pub team: Option<Pubkey>,
    #[max_len(50)]
    pub applications: Vec<Application>,
    pub bump: u8,
    pub created_at: i64,
    pub updated_at: i64,
    pub submitted_at: Option<i64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct Application {
    pub applicant: Pubkey,
    pub is_team: bool,
    #[max_len(512)]
    pub proposal: String,
    pub applied_at: i64,
    pub status: ApplicationStatus,
}

impl Default for Job {
    fn default() -> Self {
        Self {
            client: Pubkey::default(),
            title: String::new(),
            description: String::new(),
            amount: 0,
            entry_fee: 0,
            total_deposited: 0,
            deadline: 0,
            status: JobStatus::Created,
            freelancer: None,
            team: None,
            applications: Vec::new(),
            bump: 0,
            created_at: 0,
            updated_at: 0,
            submitted_at: None,
        }
    }
}
