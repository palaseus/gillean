use yew::prelude::*;
use crate::api::{BlockchainApi, BlockchainMetrics};

#[function_component(MetricsView)]
pub fn metrics_view() -> Html {
    let metrics = use_state(|| None::<BlockchainMetrics>);
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);

    {
        let metrics = metrics.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with_deps(move |_| {
            loading.set(true);
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match BlockchainApi::get_metrics().await {
                    Ok(data) => {
                        metrics.set(Some(data));
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
        <div class="metrics-view">
            if *loading {
                <div class="loading">{"Loading metrics..."}</div>
            } else if let Some(ref err) = *error {
                <div class="error">{"Error: "}{err}</div>
            } else if let Some(ref blockchain_metrics) = *metrics {
                <div class="metrics-grid">
                    <div class="metric-card">
                        <h4>{"Blockchain Overview"}</h4>
                        <div class="metric-item">
                            <label>{"Total Blocks:"}</label>
                            <span class="metric-value">{blockchain_metrics.total_blocks}</span>
                        </div>
                        <div class="metric-item">
                            <label>{"Total Transactions:"}</label>
                            <span class="metric-value">{blockchain_metrics.total_transactions}</span>
                        </div>
                        <div class="metric-item">
                            <label>{"Pending Transactions:"}</label>
                            <span class="metric-value">{blockchain_metrics.pending_transactions}</span>
                        </div>
                        <div class="metric-item">
                            <label>{"Consensus Type:"}</label>
                            <span class="metric-value">{&blockchain_metrics.consensus_type}</span>
                        </div>
                    </div>

                    <div class="metric-card">
                        <h4>{"Smart Contracts"}</h4>
                        <div class="metric-item">
                            <label>{"Total Contracts:"}</label>
                            <span class="metric-value">{blockchain_metrics.total_contracts}</span>
                        </div>
                        <div class="metric-item">
                            <label>{"Deployments:"}</label>
                            <span class="metric-value">{blockchain_metrics.contract_deployments}</span>
                        </div>
                        <div class="metric-item">
                            <label>{"Contract Calls:"}</label>
                            <span class="metric-value">{blockchain_metrics.contract_calls}</span>
                        </div>
                        <div class="metric-item">
                            <label>{"Total Gas Used:"}</label>
                            <span class="metric-value">{blockchain_metrics.total_gas_used}</span>
                        </div>
                    </div>

                    if blockchain_metrics.consensus_type == "pos" {
                        <div class="metric-card">
                            <h4>{"Proof of Stake"}</h4>
                            if let Some(validators) = blockchain_metrics.validators {
                                <div class="metric-item">
                                    <label>{"Active Validators:"}</label>
                                    <span class="metric-value">{validators}</span>
                                </div>
                            }
                            if let Some(total_stake) = blockchain_metrics.total_stake {
                                <div class="metric-item">
                                    <label>{"Total Stake:"}</label>
                                    <span class="metric-value">{total_stake}</span>
                                </div>
                            }
                            if let Some(avg_performance) = blockchain_metrics.average_performance {
                                <div class="metric-item">
                                    <label>{"Avg Performance:"}</label>
                                    <span class="metric-value">{(avg_performance * 100.0) as u64}{"%"}</span>
                                </div>
                            }
                        </div>
                    }
                </div>

                <div class="metrics-charts">
                    <h4>{"Performance Charts"}</h4>
                    <div class="chart-placeholder">
                        <p>{"Charts and graphs would be displayed here in a production environment."}</p>
                        <p>{"This could include transaction volume over time, gas usage trends, validator performance, etc."}</p>
                    </div>
                </div>
            }
        </div>
    }
}
