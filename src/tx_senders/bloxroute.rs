use std::str::FromStr;

use crate::config::RpcType;
use crate::meteora::AccountsForBuy;
use crate::tx_senders::transaction::{TransactionConfig, build_transaction_with_config};
use crate::tx_senders::{TxResult, TxSender};
use anyhow::Context;
use async_trait::async_trait;
use base64;
use reqwest::Client;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::json;
use solana_sdk::hash::Hash;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use tracing::debug;

pub struct BloxrouteTxSender {
    url: String,
    name: String,
    client: Client,
    auth: String,
    tx_config: TransactionConfig,
}

impl BloxrouteTxSender {
    pub fn new(name: String, url: String, tx_config: TransactionConfig, client: Client, auth: String) -> Self {
        Self {
            url,
            name,
            tx_config,
            client,
            auth,
        }
    }

    pub fn build_transaction_with_config(
        &self,
        _index: u32,
        recent_blockhash: Hash,
        accounts_for_buy: AccountsForBuy,
    ) -> VersionedTransaction {
        build_transaction_with_config(&self.tx_config, &RpcType::Jito, recent_blockhash, accounts_for_buy)
    }
}

#[derive(Deserialize)]
pub struct BloxrouteResponse {
    pub signature: String,
}

#[async_trait]
impl TxSender for BloxrouteTxSender {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn send_transaction(
        &self,
        index: u32,
        recent_blockhash: Hash,
        accounts_for_buy: AccountsForBuy,
    ) -> anyhow::Result<TxResult> {
        let tx = self.build_transaction_with_config(index, recent_blockhash, accounts_for_buy);
        let tx_bytes = bincode::serialize(&tx).context("cannot serialize tx to bincode")?;
        let encoded_transaction = base64::encode(tx_bytes);
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", self.auth.parse().unwrap());
        let body = json!({
            "transaction": {
                "content": encoded_transaction
            },
            "skipPreFlight": true,
            "frontRunningProtection": true,
        });
        debug!("sending tx: {}", body.to_string());
        let response = self.client.post(&self.url).headers(headers).json(&body).send().await?;
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(anyhow::anyhow!("failed to send tx: {}", body));
        }
        let parsed_resp = serde_json::from_str::<BloxrouteResponse>(&body).context("cannot deserialize signature")?;
        Ok(TxResult::Signature(
            Signature::from_str(&parsed_resp.signature).expect("failed to parse signature"),
        ))
    }
}
