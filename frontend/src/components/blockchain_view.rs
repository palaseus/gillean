use yew::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::api::BlockchainApi;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: i64,
    hash: String,
    previous_hash: String,
    transactions: Vec<Transaction>,
    nonce: u64,
    consensus_type: String,
    validator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    id: String,
    transaction_type: String,
    sender: String,
    receiver: String,
    amount: f64,
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockchainStatus {
    blocks: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    consensus_type: String,
    difficulty: u32,
    mining_reward: f64,
}

#[function_component(BlockchainView)]
pub fn blockchain_view() -> Html {
    let status = use_state(|| None::<BlockchainStatus>);
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);

    {
        let status = status.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with_deps(move |_| {
            loading.set(true);
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match BlockchainApi::get_status().await {
                    Ok(data) => {
                        status.set(Some(data));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                        loading.set(false);
                    }
                }
            });
        }, ());
    }

    html! {
        <div class="blockchain-view">
            if *loading {
                <div class="loading">{"Loading blockchain status..."}</div>
            } else if let Some(ref err) = *error {
                <div class="error">{"Error: "}{err}</div>
            } else if let Some(ref blockchain_status) = *status {
                <div class="status-overview">
                    <h3>{"Blockchain Overview"}</h3>
                    <div class="status-grid">
                        <div class="status-item">
                            <label>{"Consensus Type:"}</label>
                            <span>{&blockchain_status.consensus_type}</span>
                        </div>
                        <div class="status-item">
                            <label>{"Total Blocks:"}</label>
                            <span>{blockchain_status.blocks.len()}</span>
                        </div>
                        <div class="status-item">
                            <label>{"Pending Transactions:"}</label>
                            <span>{blockchain_status.pending_transactions.len()}</span>
                        </div>
                        <div class="status-item">
                            <label>{"Difficulty:"}</label>
                            <span>{blockchain_status.difficulty}</span>
                        </div>
                        <div class="status-item">
                            <label>{"Mining Reward:"}</label>
                            <span>{blockchain_status.mining_reward}</span>
                        </div>
                    </div>
                </div>

                <div class="blocks-list">
                    <h3>{"Recent Blocks"}</h3>
                    <div class="blocks-grid">
                        {blockchain_status.blocks.iter().rev().take(10).map(|block| {
                            html! {
                                <div class="block-item" key={block.index}>
                                    <div class="block-header">
                                        <span class="block-index">{"Block "}{block.index}</span>
                                        <span class="block-consensus">{&block.consensus_type}</span>
                                    </div>
                                    <div class="block-details">
                                        <div class="detail">
                                            <label>{"Hash:"}</label>
                                            <span class="hash">{&block.hash[..16]}{"..."}</span>
                                        </div>
                                        <div class="detail">
                                            <label>{"Transactions:"}</label>
                                            <span>{block.transactions.len()}</span>
                                        </div>
                                        <div class="detail">
                                            <label>{"Nonce:"}</label>
                                            <span>{block.nonce}</span>
                                        </div>
                                        if let Some(ref validator) = block.validator {
                                            <div class="detail">
                                                <label>{"Validator:"}</label>
                                                <span>{validator}</span>
                                            </div>
                                        }
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()}
                    </div>
                </div>

                if !blockchain_status.pending_transactions.is_empty() {
                    <div class="pending-transactions">
                        <h3>{"Pending Transactions"}</h3>
                        <div class="transactions-list">
                            {blockchain_status.pending_transactions.iter().map(|tx| {
                                html! {
                                    <div class="transaction-item" key={&tx.id}>
                                        <div class="tx-header">
                                            <span class="tx-type">{&tx.transaction_type}</span>
                                            <span class="tx-amount">{tx.amount}</span>
                                        </div>
                                        <div class="tx-details">
                                            <div class="detail">
                                                <label>{"From:"}</label>
                                                <span>{&tx.sender}</span>
                                            </div>
                                            <div class="detail">
                                                <label>{"To:"}</label>
                                                <span>{&tx.receiver}</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()}
                        </div>
                    </div>
                }
            }
        </div>
    }
}
