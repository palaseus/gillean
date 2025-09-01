use yew::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use crate::api::BlockchainApi;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransactionRequest {
    sender: String,
    receiver: String,
    amount: f64,
    message: Option<String>,
}

#[function_component(TransactionForm)]
pub fn transaction_form() -> Html {
    let sender = use_state(|| String::new());
    let receiver = use_state(|| String::new());
    let amount = use_state(|| String::new());
    let message = use_state(|| String::new());
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

    let on_receiver_change = {
        let receiver = receiver.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            receiver.set(input.value());
        })
    };

    let on_amount_change = {
        let amount = amount.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            amount.set(input.value());
        })
    };

    let on_message_change = {
        let message = message.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            message.set(input.value());
        })
    };

    let on_submit = {
        let sender = sender.clone();
        let receiver = receiver.clone();
        let amount = amount.clone();
        let message = message.clone();
        let loading = loading.clone();
        let error = error.clone();
        let success = success.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let sender_value = (*sender).clone();
            let receiver_value = (*receiver).clone();
            let amount_value = (*amount).clone();
            let message_value = (*message).clone();

            if sender_value.is_empty() || receiver_value.is_empty() || amount_value.is_empty() {
                error.set(Some("All fields are required".to_string()));
                return;
            }

            let amount_parsed = match amount_value.parse::<f64>() {
                Ok(val) if val > 0.0 => val,
                _ => {
                    error.set(Some("Amount must be a positive number".to_string()));
                    return;
                }
            };

            loading.set(true);
            error.set(None);
            success.set(None);

            let request = TransactionRequest {
                sender: sender_value,
                receiver: receiver_value,
                amount: amount_parsed,
                message: if message_value.is_empty() { None } else { Some(message_value) },
            };

            wasm_bindgen_futures::spawn_local(async move {
                match BlockchainApi::create_transaction(request).await {
                    Ok(_) => {
                        success.set(Some("Transaction created successfully!".to_string()));
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

    html! {
        <div class="transaction-form">
            <form onsubmit={on_submit}>
                <div class="form-group">
                    <label for="sender">{"Sender Address:"}</label>
                    <input
                        id="sender"
                        type="text"
                        value={(*sender).clone()}
                        onchange={on_sender_change}
                        placeholder="Enter sender address"
                        required=true
                    />
                </div>

                <div class="form-group">
                    <label for="receiver">{"Receiver Address:"}</label>
                    <input
                        id="receiver"
                        type="text"
                        value={(*receiver).clone()}
                        onchange={on_receiver_change}
                        placeholder="Enter receiver address"
                        required=true
                    />
                </div>

                <div class="form-group">
                    <label for="amount">{"Amount:"}</label>
                    <input
                        id="amount"
                        type="number"
                        step="0.01"
                        min="0.01"
                        value={(*amount).clone()}
                        onchange={on_amount_change}
                        placeholder="Enter amount"
                        required=true
                    />
                </div>

                <div class="form-group">
                    <label for="message">{"Message (Optional):"}</label>
                    <textarea
                        id="message"
                        value={(*message).clone()}
                        onchange={on_message_change}
                        placeholder="Enter optional message"
                        rows="3"
                    />
                </div>

                <div class="form-actions">
                    <button type="submit" disabled={*loading}>
                        if *loading {
                            {"Creating Transaction..."}
                        } else {
                            {"Create Transaction"}
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
