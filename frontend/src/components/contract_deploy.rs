use yew::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::api::BlockchainApi;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContractDeployRequest {
    sender: String,
    contract_code: String,
    gas_limit: u64,
    gas_price: f64,
}

#[function_component(ContractDeploy)]
pub fn contract_deploy() -> Html {
    let sender = use_state(|| String::new());
    let contract_code = use_state(|| String::new());
    let gas_limit = use_state(|| String::from("1000000"));
    let gas_price = use_state(|| String::from("0.000001"));
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    let on_sender_change = {
        let sender = sender.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            sender.set(input.value());
        })
    };

    let on_contract_code_change = {
        let contract_code = contract_code.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            contract_code.set(input.value());
        })
    };

    let on_gas_limit_change = {
        let gas_limit = gas_limit.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            gas_limit.set(input.value());
        })
    };

    let on_gas_price_change = {
        let gas_price = gas_price.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            gas_price.set(input.value());
        })
    };

    let on_submit = {
        let sender = sender.clone();
        let contract_code = contract_code.clone();
        let gas_limit = gas_limit.clone();
        let gas_price = gas_price.clone();
        let loading = loading.clone();
        let error = error.clone();
        let success = success.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let sender_value = (*sender).clone();
            let contract_code_value = (*contract_code).clone();
            let gas_limit_value = (*gas_limit).clone();
            let gas_price_value = (*gas_price).clone();

            if sender_value.is_empty() || contract_code_value.is_empty() {
                error.set(Some("Sender and contract code are required".to_string()));
                return;
            }

            let gas_limit_parsed = match gas_limit_value.parse::<u64>() {
                Ok(val) if val > 0 => val,
                _ => {
                    error.set(Some("Gas limit must be a positive number".to_string()));
                    return;
                }
            };

            let gas_price_parsed = match gas_price_value.parse::<f64>() {
                Ok(val) if val > 0.0 => val,
                _ => {
                    error.set(Some("Gas price must be a positive number".to_string()));
                    return;
                }
            };

            loading.set(true);
            error.set(None);
            success.set(None);

            let request = ContractDeployRequest {
                sender: sender_value,
                contract_code: contract_code_value,
                gas_limit: gas_limit_parsed,
                gas_price: gas_price_parsed,
            };

            wasm_bindgen_futures::spawn_local(async move {
                match BlockchainApi::deploy_contract(request).await {
                    Ok(contract_address) => {
                        success.set(Some(format!("Contract deployed successfully! Address: {}", contract_address)));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                        loading.set(false);
                    }
                }
            });
        })
    };

    let example_contract = r#"# Simple Counter Contract
PUSH 0
STORE counter
PUSH 1
STORE increment_value
RETURN"#;

    let on_load_example = {
        let contract_code = contract_code.clone();
        Callback::from(move |_| {
            contract_code.set(example_contract.to_string());
        })
    };

    html! {
        <div class="contract-deploy">
            <div class="contract-examples">
                <h4>{"Example Contracts"}</h4>
                <button onclick={on_load_example}>{"Load Counter Contract Example"}</button>
            </div>

            <form onsubmit={on_submit}>
                <div class="form-group">
                    <label for="contract-sender">{"Sender Address:"}</label>
                    <input
                        id="contract-sender"
                        type="text"
                        value={(*sender).clone()}
                        onchange={on_sender_change}
                        placeholder="Enter sender address"
                        required=true
                    />
                </div>

                <div class="form-group">
                    <label for="contract-code">{"Contract Code:"}</label>
                    <textarea
                        id="contract-code"
                        value={(*contract_code).clone()}
                        onchange={on_contract_code_change}
                        placeholder="Enter smart contract code"
                        rows="10"
                        required=true
                    />
                    <small>{"Use stack-based instructions: PUSH, STORE, LOAD, ADD, SUB, MUL, DIV, EQ, GT, IF, ENDIF, RETURN"}</small>
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label for="gas-limit">{"Gas Limit:"}</label>
                        <input
                            id="gas-limit"
                            type="number"
                            min="1"
                            value={(*gas_limit).clone()}
                            onchange={on_gas_limit_change}
                            placeholder="1000000"
                        />
                    </div>

                    <div class="form-group">
                        <label for="gas-price">{"Gas Price:"}</label>
                        <input
                            id="gas-price"
                            type="number"
                            step="0.000001"
                            min="0.000001"
                            value={(*gas_price).clone()}
                            onchange={on_gas_price_change}
                            placeholder="0.000001"
                        />
                    </div>
                </div>

                <div class="form-actions">
                    <button type="submit" disabled={*loading}>
                        if *loading {
                            {"Deploying Contract..."}
                        } else {
                            {"Deploy Contract"}
                        }
                    </button>
                </div>

                if let Some(ref err) = *error {
                    <div class="error-message">
                        {"Error: "}{err}
                    </div>
                }

                if let Some(ref msg) = *success {
                    <div class="success-message">
                        {msg}
                    </div>
                }
            </form>
        </div>
    }
}
