use crate::config::RpcType;
use crate::meteora::AccountsForBuy;
use crate::tx_senders::transaction::{TransactionConfig, build_transaction_with_config};
use crate::tx_senders::{TxResult, TxSender};
use anyhow::Context;
use async_trait::async_trait;
use serde::Serialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::hash::Hash;
use solana_transaction_status::UiTransactionEncoding;
use std::sync::Arc;

#[derive(Clone)]
pub struct GenericRpc {
    pub name: String,
    pub http_rpc: Arc<RpcClient>,
    tx_config: TransactionConfig,
    rpc_type: RpcType,
}

#[derive(Serialize, Debug)]
pub struct TxMetrics {
    pub rpc_name: String,
    pub signature: String,
    pub index: u32,
    pub success: bool,
    pub slot_sent: u64,
    pub slot_landed: Option<u64>,
    pub slot_latency: Option<u64>,
    pub elapsed: Option<u64>, // in milliseconds
}

impl GenericRpc {
    pub fn new(name: String, url: String, config: TransactionConfig, rpc_type: RpcType) -> Self {
        let http_rpc = Arc::new(RpcClient::new(url));
        GenericRpc {
            name,
            http_rpc,
            tx_config: config,
            rpc_type,
        }
    }
}

#[async_trait]
impl TxSender for GenericRpc {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn send_transaction(
        &self,
        _index: u32,
        recent_blockhash: Hash,
        accounts_for_buy: AccountsForBuy,
    ) -> anyhow::Result<TxResult> {
        let transaction =
            build_transaction_with_config(&self.tx_config, &self.rpc_type, recent_blockhash, accounts_for_buy)?;
        let sig = self
            .http_rpc
            .send_transaction_with_config(
                &transaction,
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    preflight_commitment: None,
                    encoding: Some(UiTransactionEncoding::Base64),
                    max_retries: None,
                    min_context_slot: None,
                },
            )
            .await
            .context(format!("Failed to send transaction for {}", self.name))?;
        Ok(TxResult::Signature(sig))
    }
}
