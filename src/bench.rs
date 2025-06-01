use crate::config::PingThingsArgs;
use crate::meteora::AccountsForBuy;
use crate::tx_senders::transaction::TransactionConfig;
use crate::tx_senders::{TxSender, create_tx_sender};
use reqwest::Client;
use solana_sdk::hash::Hash;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Clone)]
pub struct Bench {
    rpcs: Vec<Arc<dyn TxSender>>,
}

impl Bench {
    pub fn new(config: PingThingsArgs) -> Self {
        let tx_config: TransactionConfig = config.clone().into();
        let client = Client::new();

        let rpcs = config
            .rpc
            .clone()
            .into_iter()
            .map(|(name, rpc)| create_tx_sender(name, rpc, tx_config.clone(), client.clone()))
            .collect::<Vec<Arc<dyn TxSender>>>();

        Bench { rpcs }
    }

    pub async fn send_and_confirm_transaction(
        tx_index: u32,
        rpc_sender: Arc<dyn TxSender>,
        recent_blockhash: Hash,
        accounts_for_buy: AccountsForBuy,
    ) -> anyhow::Result<()> {
        let start = tokio::time::Instant::now();

        let _tx_result = rpc_sender.send_transaction(tx_index, recent_blockhash, accounts_for_buy).await?;

        info!(
            "complete rpc: {:?} {:?} ms",
            rpc_sender.name(),
            start.elapsed().as_millis() as u64
        );
        Ok(())
    }

    pub async fn send_buy_tx(self, recent_blockhash: Hash, accounts_for_buy: AccountsForBuy) {
        tokio::select! {
            _ = self.send_buy_tx_inner(
                recent_blockhash,
                accounts_for_buy,
            ) => {}
        }
    }

    async fn send_buy_tx_inner(self, recent_blockhash: Hash, accounts_for_buy: AccountsForBuy) {
        let start = tokio::time::Instant::now();
        info!("starting create buy tx");
        let mut tx_handles = Vec::new();

        for rpc in &self.rpcs {
            // let rpc_name = rpc.name();
            let rpc_sender = rpc.clone();
            // let client = self.client.clone();
            let hdl = tokio::spawn(async move {
                let index = 0;
                if let Err(e) =
                    Self::send_and_confirm_transaction(index, rpc_sender, recent_blockhash, accounts_for_buy).await
                {
                    error!("error end_and_confirm_transaction {:?}", e);
                }
            });
            tx_handles.push(hdl);
        }
        info!("waiting for transactions to complete...");

        // wait for all transactions to complete
        for hdl in tx_handles {
            hdl.await.unwrap_or_default();
        }

        info!("bench complete! {:?} ms", start.elapsed().as_millis() as u64);
    }
}
