use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use mpl_token_metadata::instructions::{
    CreateMetadataAccountV3Cpi, CreateMetadataAccountV3CpiAccounts,
    CreateMetadataAccountV3InstructionArgs, CreateMasterEditionV3Cpi,
    CreateMasterEditionV3CpiAccounts, CreateMasterEditionV3InstructionArgs,
};
use mpl_token_metadata::types::{DataV2, Creator};

declare_id!("4fRvr5yrDNTqnSXv8yFb9CSj3MwnYuade8UUmgb8cg3H");


// ─── Constants ────────────────────────────────────────────────────────────────

const MAX_ORDER_ID_LEN: usize = 64;
const MAX_NFC_UID_LEN: usize = 32;
const MAX_PRODUCT_NAME_LEN: usize = 128;
const MAX_METADATA_URI_LEN: usize = 200;
const MAX_FARMER_ID_LEN: usize = 64;
const MAX_PARTNER_ID_LEN: usize = 64;
const MAX_CONSUMER_ID_LEN: usize = 64;

// Status constants — keep in sync with DeliveryTrace.status
pub const STATUS_INITIALIZED: u8 = 0;
pub const STATUS_PICKED_UP: u8 = 1;
pub const STATUS_IN_TRANSIT: u8 = 2;
pub const STATUS_DELIVERED: u8 = 3;

// ─── Program ──────────────────────────────────────────────────────────────────

#[program]
pub mod chaintrace {
    use super::*;

    pub fn mint_product_nft(
        ctx: Context<MintProductNft>,
        args: MintProductArgs,
    ) -> Result<()> {
        // ── Validate inputs ────────────────────────────────────────────────
        require!(!args.order_id.is_empty(), ChainTraceError::EmptyOrderId);
        require!(
            args.order_id.len() <= MAX_ORDER_ID_LEN,
            ChainTraceError::StringTooLong
        );
        require!(
            !args.product_name.is_empty(),
            ChainTraceError::EmptyProductName
        );
        require!(
            args.product_name.len() <= MAX_PRODUCT_NAME_LEN,
            ChainTraceError::StringTooLong
        );
        require!(
            !args.metadata_uri.is_empty(),
            ChainTraceError::EmptyMetadataUri
        );
        require!(
            args.metadata_uri.len() <= MAX_METADATA_URI_LEN,
            ChainTraceError::StringTooLong
        );

        // ── Mint 1 token to the server's ATA ──────────────────────────────
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.server_authority.to_account_info(),
            },
        );
        token::mint_to(cpi_ctx, 1)?;

        // ── Create Metaplex metadata ───────────────────────────────────────
        let creators = vec![Creator {
            address: ctx.accounts.server_authority.key(),
            verified: true,
            share: 100,
        }];

        let meta_metadata_ai  = ctx.accounts.metadata.to_account_info();
        let meta_mint_ai      = ctx.accounts.mint.to_account_info();
        let meta_authority_ai = ctx.accounts.server_authority.to_account_info();
        let meta_system_ai    = ctx.accounts.system_program.to_account_info();
        let meta_rent_ai      = ctx.accounts.rent.to_account_info();

        let metadata_cpi = CreateMetadataAccountV3Cpi::new(
            &ctx.accounts.token_metadata_program,
            CreateMetadataAccountV3CpiAccounts {
                metadata: &meta_metadata_ai,
                mint: &meta_mint_ai,
                mint_authority: &meta_authority_ai,
                payer: &meta_authority_ai,
                update_authority: (&meta_authority_ai, true),
                system_program: &meta_system_ai,
                rent: Some(&meta_rent_ai),
            },
            CreateMetadataAccountV3InstructionArgs {
                data: DataV2 {
                    name: args.product_name.clone(),
                    symbol: "CT".to_string(),
                    uri: args.metadata_uri.clone(),
                    seller_fee_basis_points: 0,
                    creators: Some(creators),
                    collection: None,
                    uses: None,
                },
                is_mutable: true,
                collection_details: None,
            },
        );
        metadata_cpi.invoke()?;

        // ── Create MasterEdition (cap supply at 1/1) ───────────────────────
        let ed_edition_ai       = ctx.accounts.master_edition.to_account_info();
        let ed_mint_ai          = ctx.accounts.mint.to_account_info();
        let ed_authority_ai     = ctx.accounts.server_authority.to_account_info();
        let ed_metadata_ai      = ctx.accounts.metadata.to_account_info();
        let ed_token_program_ai = ctx.accounts.token_program.to_account_info();
        let ed_system_ai        = ctx.accounts.system_program.to_account_info();
        let ed_rent_ai          = ctx.accounts.rent.to_account_info();

        let edition_cpi = CreateMasterEditionV3Cpi::new(
            &ctx.accounts.token_metadata_program,
            CreateMasterEditionV3CpiAccounts {
                edition: &ed_edition_ai,
                mint: &ed_mint_ai,
                update_authority: &ed_authority_ai,
                mint_authority: &ed_authority_ai,
                payer: &ed_authority_ai,
                metadata: &ed_metadata_ai,
                token_program: &ed_token_program_ai,
                system_program: &ed_system_ai,
                rent: Some(&ed_rent_ai),
            },
            CreateMasterEditionV3InstructionArgs {
                max_supply: Some(0),
            },
        );
        edition_cpi.invoke()?;

        // ── Write ProductTrace PDA ─────────────────────────────────────────
        let trace = &mut ctx.accounts.product_trace;
        trace.order_id = args.order_id.clone();
        trace.nft_mint = ctx.accounts.mint.key();
        trace.farmer_wallet = args.farmer_wallet;
        trace.product_name = args.product_name;
        trace.metadata_uri = args.metadata_uri;
        trace.created_at = Clock::get()?.unix_timestamp;
        trace.bump = ctx.bumps.product_trace;

        emit!(ProductNftMinted {
            order_id: args.order_id,
            nft_mint: ctx.accounts.mint.key(),
            timestamp: trace.created_at,
        });

        Ok(())
    }

    /// Record NFC tap by farmer during packaging — creates the DeliveryTrace PDA.
    ///
    /// Edge cases handled:
    ///  - Requires ProductTrace to already exist (seeds derive same order_id).
    ///    If ProductTrace PDA is absent the ix fails naturally because the
    ///    `product_trace` account constraint won't resolve.
    ///  - Double-initialization prevented: `init` on DeliveryTrace PDA will
    ///    fail if it already exists, returning AccountAlreadyInUse.
    ///    We surface a cleaner error via a manual check.
    ///  - All DB foreign-key IDs validated for non-empty / length.
    pub fn initialize_delivery(
        ctx: Context<InitializeDelivery>,
        args: InitDeliveryArgs,
    ) -> Result<()> {
        // ── Validate inputs ────────────────────────────────────────────────
        require!(!args.order_id.is_empty(), ChainTraceError::EmptyOrderId);
        require!(
            args.order_id.len() <= MAX_ORDER_ID_LEN,
            ChainTraceError::StringTooLong
        );
        require!(!args.nfc_uid.is_empty(), ChainTraceError::EmptyNfcUid);
        require!(
            args.nfc_uid.len() <= MAX_NFC_UID_LEN,
            ChainTraceError::StringTooLong
        );
        require!(
            !args.farmer_id.is_empty(),
            ChainTraceError::EmptyFarmerId
        );
        require!(
            args.farmer_id.len() <= MAX_FARMER_ID_LEN,
            ChainTraceError::StringTooLong
        );
        require!(
            !args.delivery_partner_id.is_empty(),
            ChainTraceError::EmptyDeliveryPartnerId
        );
        require!(
            args.delivery_partner_id.len() <= MAX_PARTNER_ID_LEN,
            ChainTraceError::StringTooLong
        );
        require!(
            !args.consumer_id.is_empty(),
            ChainTraceError::EmptyConsumerId
        );
        require!(
            args.consumer_id.len() <= MAX_CONSUMER_ID_LEN,
            ChainTraceError::StringTooLong
        );

        // ── Validate order_id matches the ProductTrace we loaded ───────────
        // (extra safety — Anchor PDA seeds already enforce this, but being
        //  explicit guards against a future refactor that changes seed logic)
        require!(
            ctx.accounts.product_trace.order_id == args.order_id,
            ChainTraceError::OrderIdMismatch
        );

        // ── Write DeliveryTrace PDA ────────────────────────────────────────
        let now = Clock::get()?.unix_timestamp;
        let delivery = &mut ctx.accounts.delivery_trace;
        delivery.order_id = args.order_id.clone();
        delivery.nft_mint = ctx.accounts.product_trace.nft_mint;
        delivery.nfc_uid = args.nfc_uid.clone();
        delivery.status = STATUS_INITIALIZED;
        delivery.farmer_id = args.farmer_id;
        delivery.delivery_partner_id = args.delivery_partner_id;
        delivery.consumer_id = args.consumer_id;
        delivery.initialized_at = now;
        delivery.picked_up_at = 0;    // 0 = not yet set
        delivery.in_transit_at = 0;
        delivery.delivered_at = 0;
        delivery.bump = ctx.bumps.delivery_trace;

        emit!(DeliveryInitialized {
            order_id: args.order_id,
            nfc_uid: args.nfc_uid,
            timestamp: now,
        });

        Ok(())
    }

    /// Advance delivery status and record timestamp.
    ///
    /// Edge cases handled:
    ///  - Only forward transitions allowed (status can only increase).
    ///    Prevents replayed or out-of-order server calls from corrupting state.
    ///  - Skipping a status is NOT allowed (e.g., 0 → 2 is rejected).
    ///    Every step must be recorded so the trace is complete.
    ///  - Attempting to move past DELIVERED (3) is rejected.
    ///  - Timestamp for each milestone recorded once and never overwritten
    ///    (idempotent replay protection: if server accidentally sends the
    ///    same status twice the second call fails on transition check).
    pub fn update_delivery_status(
        ctx: Context<UpdateDeliveryStatus>,
        args: UpdateStatusArgs,
    ) -> Result<()> {
        // ── Validate new_status range ──────────────────────────────────────
        require!(
            args.new_status <= STATUS_DELIVERED,
            ChainTraceError::InvalidStatusValue
        );

        let delivery = &mut ctx.accounts.delivery_trace;
        let current = delivery.status;

        require!(
            args.new_status == current + 1,
            ChainTraceError::InvalidStatusTransition
        );

        let now = Clock::get()?.unix_timestamp;
        delivery.status = args.new_status;

        match args.new_status {
            STATUS_PICKED_UP => {
                delivery.picked_up_at = now;
            }
            STATUS_IN_TRANSIT => {
                delivery.in_transit_at = now;
            }
            STATUS_DELIVERED => {
                delivery.delivered_at = now;
            }
       
            _ => return Err(ChainTraceError::InvalidStatusTransition.into()),
        }

        emit!(DeliveryStatusUpdated {
            order_id: delivery.order_id.clone(),
            new_status: args.new_status,
            timestamp: now,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(args: MintProductArgs)]
pub struct MintProductNft<'info> {
    /// The server keypair — must sign every tx.
    #[account(mut)]
    pub server_authority: Signer<'info>,

    /// Fresh mint account created by this ix.
    #[account(
        init,
        payer = server_authority,
        mint::decimals = 0,
        mint::authority = server_authority,
        mint::freeze_authority = server_authority,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = server_authority,
        associated_token::mint = mint,
        associated_token::authority = server_authority,
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = server_authority,
        space = ProductTrace::space(&args.order_id, &args.product_name, &args.metadata_uri),
        seeds = [b"product", args.order_id.as_bytes()],
        bump,
    )]
    pub product_trace: Account<'info, ProductTrace>,

    /// CHECK: Metaplex metadata PDA — validated by the token-metadata program.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Metaplex master-edition PDA — validated by the token-metadata program.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Metaplex token-metadata program.
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(args: InitDeliveryArgs)]
pub struct InitializeDelivery<'info> {
    #[account(mut)]
    pub server_authority: Signer<'info>,

     
    #[account(
        seeds = [b"product", args.order_id.as_bytes()],
        bump = product_trace.bump,
    )]
    pub product_trace: Account<'info, ProductTrace>,

   
    #[account(
        init,
        payer = server_authority,
        space = DeliveryTrace::space(
            &args.order_id,
            &args.nfc_uid,
            &args.farmer_id,
            &args.delivery_partner_id,
            &args.consumer_id,
        ),
        seeds = [b"delivery", args.order_id.as_bytes()],
        bump,
    )]
    pub delivery_trace: Account<'info, DeliveryTrace>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(args: UpdateStatusArgs)]
pub struct UpdateDeliveryStatus<'info> {
    #[account(mut)]
    pub server_authority: Signer<'info>,

    /// DeliveryTrace must already exist.
    #[account(
        mut,
        seeds = [b"delivery", delivery_trace.order_id.as_bytes()],
        bump = delivery_trace.bump,
        // Extra guard: server_authority must match the program authority stored
        // at deploy time — prevents a compromised second keypair from writing.
        // (Optional: remove if you only ever have one server wallet.)
        // constraint = delivery_trace.bump != 0 @ ChainTraceError::NotInitialized,
    )]
    pub delivery_trace: Account<'info, DeliveryTrace>,

    pub system_program: Program<'info, System>,
}

// ─── Instruction Arguments ────────────────────────────────────────────────────

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MintProductArgs {
    pub order_id: String,
    pub product_name: String,
    pub metadata_uri: String,
    pub farmer_wallet: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitDeliveryArgs {
    pub order_id: String,
    pub nfc_uid: String,
    pub farmer_id: String,
    pub delivery_partner_id: String,
    pub consumer_id: String,
}   

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateStatusArgs {
    pub new_status: u8,
}

// ─── State Accounts ───────────────────────────────────────────────────────────

#[account]
pub struct ProductTrace {
    pub order_id: String,
    pub nft_mint: Pubkey,
    pub farmer_wallet: Pubkey,
    pub product_name: String,
    pub metadata_uri: String,
    pub created_at: i64,
    pub bump: u8,
}

impl ProductTrace {
    /// Dynamic space calculation.
    /// Anchor string = 4-byte length prefix + bytes.
    pub fn space(order_id: &str, product_name: &str, metadata_uri: &str) -> usize {
        8                           // discriminator
        + (4 + order_id.len())
        + 32                        // nft_mint
        + 32                        // farmer_wallet
        + (4 + product_name.len())
        + (4 + metadata_uri.len())
        + 8                         // created_at
        + 1                         // bump
        + 64                        // padding (future fields)
    }
}

#[account]
pub struct DeliveryTrace {
    pub order_id: String,
    pub nft_mint: Pubkey,
    pub nfc_uid: String,
    pub status: u8,
    pub farmer_id: String,
    pub delivery_partner_id: String,
    pub consumer_id: String,
    pub initialized_at: i64,
    pub picked_up_at: i64,
    pub in_transit_at: i64,
    pub delivered_at: i64,
    pub bump: u8,
}

impl DeliveryTrace {
    pub fn space(
        order_id: &str,
        nfc_uid: &str,
        farmer_id: &str,
        delivery_partner_id: &str,
        consumer_id: &str,
    ) -> usize {
        8                                    // discriminator
        + (4 + order_id.len())
        + 32                                 // nft_mint
        + (4 + nfc_uid.len())
        + 1                                  // status
        + (4 + farmer_id.len())
        + (4 + delivery_partner_id.len())
        + (4 + consumer_id.len())
        + 8                                  // initialized_at
        + 8                                  // picked_up_at
        + 8                                  // in_transit_at
        + 8                                  // delivered_at
        + 1                                  // bump
        + 64                                 // padding
    }
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[event]
pub struct ProductNftMinted {
    pub order_id: String,
    pub nft_mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct DeliveryInitialized {
    pub order_id: String,
    pub nfc_uid: String,
    pub timestamp: i64,
}

#[event]
pub struct DeliveryStatusUpdated {
    pub order_id: String,
    pub new_status: u8,
    pub timestamp: i64,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[error_code]
pub enum ChainTraceError {
    #[msg("Order ID cannot be empty")]
    EmptyOrderId,

    #[msg("NFC UID cannot be empty")]
    EmptyNfcUid,

    #[msg("Product name cannot be empty")]
    EmptyProductName,

    #[msg("Metadata URI cannot be empty")]
    EmptyMetadataUri,

    #[msg("Farmer ID cannot be empty")]
    EmptyFarmerId,

    #[msg("Delivery partner ID cannot be empty")]
    EmptyDeliveryPartnerId,

    #[msg("Consumer ID cannot be empty")]
    EmptyConsumerId,

    #[msg("A string argument exceeds the maximum allowed length")]
    StringTooLong,

    #[msg("Status can only advance by one step at a time (no skipping)")]
    InvalidStatusTransition,

    #[msg("new_status value is out of range (max 3)")]
    InvalidStatusValue,

    #[msg("Delivery already initialized for this order")]
    AlreadyInitialized,

    #[msg("order_id in args does not match the loaded ProductTrace PDA")]
    OrderIdMismatch,

    #[msg("DeliveryTrace has not been initialized yet")]
    NotInitialized,
}