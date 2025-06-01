use crate::config::{PingThingsArgs, RpcType};
use crate::meteora::AccountsForBuy;
use crate::tx_senders::constants::{JITO_TIP, TOKEN_PROGRAM};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::VersionedMessage;
use solana_sdk::message::v0::Message;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::VersionedTransaction;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use std::sync::Arc;

use super::constants::{BLOXROUTE_TIP, METEORA_POOLS_PROGRAM, METEORA_VAULT_PROGRAM, NEXTBLOCK_TIP, WSOL_MINT};

#[derive(Clone)]
pub struct TransactionConfig {
    pub keypair: Arc<Keypair>,
    pub compute_unit_limit: u32,
    pub compute_unit_price: u64,
    pub tip: u64,
    pub buy_amount: u64,
    pub min_amount_out: u64,
}

impl From<PingThingsArgs> for TransactionConfig {
    fn from(args: PingThingsArgs) -> Self {
        let keypair = Keypair::from_base58_string(args.private_key.as_str());

        let tip: u64 = (args.tip * LAMPORTS_PER_SOL as f64) as u64;
        let buy_amount: u64 = (args.buy_amount * LAMPORTS_PER_SOL as f64) as u64;
        let min_amount_out: u64 = (args.min_amount_out * 1_000_000 as f64) as u64;

        TransactionConfig {
            keypair: Arc::new(keypair),
            compute_unit_limit: args.compute_unit_limit,
            compute_unit_price: args.compute_unit_price,
            tip,
            buy_amount,
            min_amount_out,
        }
    }
}
pub fn build_transaction_with_config(
    tx_config: &TransactionConfig,
    rpc_type: &RpcType,
    recent_blockhash: Hash,
    accounts_for_buy: AccountsForBuy,
) -> anyhow::Result<VersionedTransaction> {
    let mut instructions = Vec::new();

    if tx_config.compute_unit_limit > 0 {
        let compute_unit_limit = ComputeBudgetInstruction::set_compute_unit_limit(tx_config.compute_unit_limit);
        instructions.push(compute_unit_limit);
    }

    if tx_config.compute_unit_price > 0 {
        let compute_unit_price = ComputeBudgetInstruction::set_compute_unit_price(tx_config.compute_unit_price);
        instructions.push(compute_unit_price);
    }

    if tx_config.tip > 0 {
        let tip_instruction: Option<Instruction> = match rpc_type {
            RpcType::Jito => Some(system_instruction::transfer(
                &tx_config.keypair.pubkey(),
                &JITO_TIP,
                tx_config.tip,
            )),
            RpcType::Bloxroute => Some(system_instruction::transfer(
                &tx_config.keypair.pubkey(),
                &BLOXROUTE_TIP,
                tx_config.tip,
            )),
            RpcType::Nextblock => Some(system_instruction::transfer(
                &tx_config.keypair.pubkey(),
                &NEXTBLOCK_TIP,
                tx_config.tip,
            )),
            _ => None,
        };

        if tip_instruction.is_some() {
            instructions.push(tip_instruction.unwrap());
        }
    }

    let AccountsForBuy {
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
    } = accounts_for_buy;

    let owner = tx_config.keypair.pubkey();
    let user_source_token = get_associated_token_address(&owner, &WSOL_MINT);
    let user_destination_token = get_associated_token_address(&owner, &a_token_mint);
    
    // Create WSOL account if needed
    let wsol_account_instruction = create_associated_token_account(&owner, &owner, &WSOL_MINT, &TOKEN_PROGRAM);
    instructions.push(wsol_account_instruction);
    
    // Wrap SOL to WSOL by transferring SOL to the WSOL account
    let wrap_sol_instruction = system_instruction::transfer(&owner, &user_source_token, tx_config.buy_amount);
    instructions.push(wrap_sol_instruction);
    
    // Sync native instruction for WSOL to update the token balance
    let sync_native_instruction = spl_token::instruction::sync_native(&TOKEN_PROGRAM, &user_source_token)?;
    instructions.push(sync_native_instruction);
    
    // Create destination token account
    let token_account_instruction = create_associated_token_account(&owner, &owner, &a_token_mint, &TOKEN_PROGRAM);
    instructions.push(token_account_instruction);

    // Swap instruction data
    let buy: u64 = 0xf8c69e91e17587c8;
    let mut data = vec![];
    data.extend_from_slice(&buy.to_le_bytes());
    data.extend_from_slice(&tx_config.min_amount_out.to_le_bytes());
    data.extend_from_slice(&tx_config.buy_amount.to_le_bytes());

    let accounts = vec![
        AccountMeta::new(pool, false),
        AccountMeta::new(user_source_token, false),
        AccountMeta::new(user_destination_token, false),
        AccountMeta::new(a_vault, false),
        AccountMeta::new(b_vault, false),
        AccountMeta::new(a_token_vault, false),
        AccountMeta::new(b_token_vault, false),
        AccountMeta::new(a_vault_lp_mint, false),
        AccountMeta::new(b_vault_lp_mint, false),
        AccountMeta::new(a_vault_lp, false),
        AccountMeta::new(b_vault_lp, false),
        AccountMeta::new(protocol_token_fee, false),
        AccountMeta::new_readonly(owner, true),
        AccountMeta::new_readonly(METEORA_VAULT_PROGRAM, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM, false),
    ];

    let swap_instruction = Instruction {
        program_id: METEORA_POOLS_PROGRAM,
        accounts,
        data,
    };

    instructions.push(swap_instruction);

    let message_v0 = Message::try_compile(&owner, instructions.as_slice(), &[], recent_blockhash)?;

    let versioned_message = VersionedMessage::V0(message_v0);

    Ok(VersionedTransaction::try_new(versioned_message, &[&tx_config.keypair])?)
}
