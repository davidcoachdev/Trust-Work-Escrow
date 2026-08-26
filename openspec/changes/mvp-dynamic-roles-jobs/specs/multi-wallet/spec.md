# multi-wallet Specification

## Purpose
Wallet set `user_wallets {email, pubkey, purpose: publish|apply|general, label, created_at}` with default-auto (1 wallet) vs picker (2+ wallets) per action. Funds check via `getBalance` before relay.

## Requirements

### Requirement: Wallet Set with Purpose
The system SHALL store 1..N wallets per email with `purpose ∈ {publish, apply, general}`. Legacy `wallet_pubkey` SHALL be aliased to `publish` wallet for backward compat. The system SHALL validate each pubkey as 32-byte bs58.

#### Scenario: First wallet auto-migrated
- GIVEN existing user with single `wallet_pubkey=ABC`
- WHEN migration runs
- THEN `user_wallets` contains one row `{ABC, purpose=publish, label="Principal"}`

#### Scenario: Second wallet added with purpose
- GIVEN user already has publish wallet `ABC`
- WHEN user links wallet `XYZ` with `purpose=apply` via `POST /users/:email/wallets`
- THEN repository stores second row and `GET /users/:email/wallets` returns 2 wallets

#### Scenario: Invalid pubkey rejected
- GIVEN wallet string not 32-byte bs58
- WHEN linking wallet
- THEN API returns 400 ValidationError

### Requirement: Default vs Picker Signing
The system SHALL auto-select wallet when `wallets.len()==1`; when `len>=2` the system SHALL present picker and require explicit `signer_purpose` matching the action (publish wallet for `create_job`, apply wallet for `apply_to_job`). SIWS `x-pubkey` SHALL match chosen wallet or backend rejects.

#### Scenario: Single wallet auto-sign
- GIVEN user has 1 wallet `ABC`
- WHEN user creates job
- THEN frontend auto-uses `ABC` without picker and relay succeeds

#### Scenario: Two wallets picker required
- GIVEN user has `ABC(publish)` and `XYZ(apply)`
- WHEN user applies to job without selecting wallet
- THEN UI forces picker; API returns 400 `MissingWalletPurpose` if `signer_purpose` absent

### Requirement: Funds Check Before Relay
The system SHALL call `getBalance` on chosen wallet before `relay_signed_job_transaction` and block with insufficient-funds error if `balance < amount + fee_amount`.

#### Scenario: Wallet without funds blocked
- GIVEN chosen wallet balance 0.01 SOL, job amount 1 SOL + fee 0.025 SOL
- WHEN user confirms create
- THEN `getBalance` check returns 400 `InsufficientFunds` and transaction is not relayed

#### Scenario: Publish wallet used for apply is warned
- GIVEN user selects publish wallet `ABC` to apply to foreign job
- WHEN frontend detects `purpose=publish` for `apply` action
- THEN UI shows warning "Usarás wallet de publicación para postularte" but allows if confirmed (no hard block beyond self-apply rule)
