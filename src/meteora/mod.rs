use crate::bench::Bench;
use crate::core::extract_instructions;
use crate::tx_senders::constants::METEORA_POOLS_PROGRAM;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::TransactionStatusMeta;
use tracing::log::info;
// 30 95 dc 82 3d 0b 09 b2
pub const CREATE_IX_DISC: [u8; 8] = [0x30, 0x95, 0xdc, 0x82, 0x3d, 0x0b, 0x09, 0xb2];
pub const IX_DISCRIMINATOR_SIZE: usize = 8;

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone)]
pub struct CreateIxData {
    pub token_a: u64,
    pub token_b: u64,
}

#[derive(Debug, Copy, Clone)]
pub struct AccountsForBuy {
    pub pool: Pubkey,
    pub a_token_mint: Pubkey,
    pub a_vault: Pubkey,
    pub b_vault: Pubkey,
    pub a_token_vault: Pubkey,
    pub b_token_vault: Pubkey,
    pub a_vault_lp_mint: Pubkey,
    pub b_vault_lp_mint: Pubkey,
    pub a_vault_lp: Pubkey,
    pub b_vault_lp: Pubkey,
    pub protocol_token_fee: Pubkey,
}

pub struct MeteoraController {
    bench: Bench,

    is_buy: bool,
}

impl MeteoraController {
    pub fn new(bench: Bench) -> Self {
        MeteoraController {
            bench,
            is_buy: false,
        }
    }

    pub async fn transaction_handler(
        &mut self,
        _signature: Signature,
        transaction: VersionedTransaction,
        meta: TransactionStatusMeta,
        _is_vote: bool,
        _slot: u64,
    ) -> anyhow::Result<()> {
        let instructions: Vec<solana_sdk::instruction::Instruction> = extract_instructions(meta, transaction.clone())?;

        if !self.is_buy {
            for instruction in instructions {
                if instruction.program_id == METEORA_POOLS_PROGRAM {
                    let ix_discriminator: [u8; 8] = instruction.data[0..IX_DISCRIMINATOR_SIZE].try_into()?;

                    let mut ix_data = &instruction.data[IX_DISCRIMINATOR_SIZE..];

                    if ix_discriminator == CREATE_IX_DISC {

                        let create_ix_data: CreateIxData = BorshDeserialize::deserialize(&mut ix_data)?;
                        info!("create ix: {:?}", create_ix_data);

                        let pool = instruction.accounts[0].pubkey;
                        let a_token_mint = instruction.accounts[3].pubkey;
                        let a_vault = instruction.accounts[5].pubkey;
                        let b_vault = instruction.accounts[6].pubkey;
                        let a_token_vault = instruction.accounts[7].pubkey;
                        let b_token_vault = instruction.accounts[8].pubkey;
                        let a_vault_lp_mint = instruction.accounts[9].pubkey;
                        let b_vault_lp_mint = instruction.accounts[10].pubkey;
                        let a_vault_lp = instruction.accounts[11].pubkey;
                        let b_vault_lp = instruction.accounts[12].pubkey;
                        let protocol_token_fee = instruction.accounts[17].pubkey;

                        let recent_blockhash: Hash = *transaction.message.recent_blockhash();
                        self.is_buy = true;
                        self.bench
                            .clone()
                            .send_buy_tx(
                                recent_blockhash,
                                AccountsForBuy {
                                    pool,
                                    a_token_mint,
                                    a_vault,
                                    b_vault,
                                    a_token_vault,
                                    b_token_vault,
                                    a_vault_lp_mint,
                                    b_vault_lp_mint,
                                    a_vault_lp,
                                    b_vault_lp,
                                    protocol_token_fee,
                                },
                            )
                            .await;
                    }
                }
            }
        }

        Ok(())
    }
}
