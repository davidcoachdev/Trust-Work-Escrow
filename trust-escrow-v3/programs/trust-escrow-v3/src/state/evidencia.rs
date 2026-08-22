use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Evidence {
    pub dispute: Pubkey,
    pub index: u8,
    pub author: Pubkey,
    pub content_hash: [u8; 32],
    pub bump: u8,
}

