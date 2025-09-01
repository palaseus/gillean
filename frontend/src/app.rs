use yew::prelude::*;
use crate::components::{BlockchainView, TransactionForm, ContractDeploy, WalletManager, MetricsView};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div class="app">
            <header class="app-header">
                <h1>{"Gillean Blockchain Explorer"}</h1>
                <p>{"Advanced Educational Blockchain Platform"}</p>
            </header>
            
            <nav class="app-nav">
                <a href="#blockchain">{"Blockchain"}</a>
                <a href="#transactions">{"Transactions"}</a>
                <a href="#contracts">{"Smart Contracts"}</a>
                <a href="#wallet">{"Wallet"}</a>
                <a href="#metrics">{"Metrics"}</a>
            </nav>
            
            <main class="app-main">
                <div class="content-section">
                    <h2>{"Blockchain Status"}</h2>
                    <BlockchainView />
                </div>
                
                <div class="content-section">
                    <h2>{"Create Transaction"}</h2>
                    <TransactionForm />
                </div>
                
                <div class="content-section">
                    <h2>{"Deploy Smart Contract"}</h2>
                    <ContractDeploy />
                </div>
                
                <div class="content-section">
                    <h2>{"Wallet Management"}</h2>
                    <WalletManager />
                </div>
                
                <div class="content-section">
                    <h2>{"Network Metrics"}</h2>
                    <MetricsView />
                </div>
            </main>
        </div>
    }
}
