use yew::prelude::*;
use crate::api::{BlockchainApi, WalletInfo, CreateWalletRequest};

#[function_component(WalletManager)]
pub fn wallet_manager() -> Html {
    let wallets = use_state(Vec::<WalletInfo>::new);
    let password = use_state(String::new);
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| None::<String>);

    {
        let wallets = wallets.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with_deps(move |_| {
            loading.set(true);
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match BlockchainApi::get_wallets().await {
                    Ok(data) => {
                        wallets.set(data);
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

    let on_password_change = {
        let password = password.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let on_create_wallet = {
        let password = password.clone();
        let wallets = wallets.clone();
        let loading = loading.clone();
        let error = error.clone();
        let success = success.clone();

        Callback::from(move |_| {
            let password_value = (*password).clone();

            if password_value.is_empty() {
                error.set(Some("Password is required".to_string()));
                return;
            }

            loading.set(true);
            error.set(None);
            success.set(None);

            let request = CreateWalletRequest {
                password: password_value,
            };

            let wallets_clone = wallets.clone();
            let success_clone = success.clone();
            let error_clone = error.clone();
            let loading_clone = loading.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                match BlockchainApi::create_wallet(request).await {
                    Ok(wallet_info) => {
                        let mut current_wallets = (*wallets_clone).clone();
                        current_wallets.push(wallet_info);
                        wallets_clone.set(current_wallets);
                        success_clone.set(Some("Wallet created successfully!".to_string()));
                        loading_clone.set(false);
                    }
                    Err(e) => {
                        error_clone.set(Some(e.to_string()));
                        loading_clone.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="wallet-manager">
            <div class="create-wallet">
                <h4>{"Create New Wallet"}</h4>
                <div class="form-group">
                    <label for="wallet-password">{"Password:"}</label>
                    <input
                        id="wallet-password"
                        type="password"
                        value={(*password).clone()}
                        onchange={on_password_change}
                        placeholder="Enter wallet password"
                    />
                </div>
                <button onclick={on_create_wallet} disabled={*loading}>
                    if *loading {
                        {"Creating Wallet..."}
                    } else {
                        {"Create Wallet"}
                    }
                </button>
            </div>

            if *loading {
                <div class="loading">{"Loading wallets..."}</div>
            } else if let Some(ref err) = *error {
                <div class="error">{"Error: "}{err}</div>
            } else {
                <div class="wallets-list">
                    <h4>{"Your Wallets"}</h4>
                    if wallets.is_empty() {
                        <p>{"No wallets found. Create your first wallet above."}</p>
                    } else {
                        <div class="wallets-grid">
                            {wallets.iter().map(|wallet| {
                                html! {
                                    <div class="wallet-item" key={&*wallet.address}>
                                        <div class="wallet-header">
                                            <span class="wallet-address">{&wallet.address[..16]}{"..."}</span>
                                            <span class="wallet-balance">{wallet.balance}</span>
                                        </div>
                                        <div class="wallet-details">
                                            <div class="detail">
                                                <label>{"Public Key:"}</label>
                                                <span class="public-key">{&wallet.public_key[..16]}{"..."}</span>
                                            </div>
                                            <div class="detail">
                                                <label>{"Balance:"}</label>
                                                <span class="balance">{wallet.balance}</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()}
                        </div>
                    }
                </div>
            }

            if let Some(ref msg) = *success {
                <div class="success-message">
                    {msg}
                </div>
            }
        </div>
    }
}
