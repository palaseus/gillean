# Gillean Frontend v2.0.0

A modern, real-time web interface for the Gillean blockchain platform built with Yew (Rust + WebAssembly).

## 🌟 Features

### Real-time Updates
- **WebSocket Support**: Live blockchain updates via WebSocket connections
- **Real-time Metrics**: Live monitoring of blockchain statistics
- **Live Transaction Feed**: Real-time transaction processing updates
- **Shard Monitoring**: Live shard status and cross-shard transaction tracking
- **Cross-chain Activity**: Real-time cross-chain transfer monitoring

### Enhanced UI Components
- **Blockchain Explorer**: Comprehensive block and transaction explorer
- **Transaction Creation**: Create and send transactions with real-time feedback
- **Smart Contract Deployment**: Deploy and interact with WASM contracts
- **Wallet Management**: Create and manage encrypted wallets
- **Network Metrics**: Real-time blockchain statistics and health monitoring
- **Shard Dashboard**: View shard status and cross-shard transactions
- **Cross-chain Interface**: Initiate and monitor cross-chain transfers
- **Contract Interaction**: Deploy and interact with WASM smart contracts

### Modern Web Technologies
- **Yew Framework**: Rust-based frontend framework
- **WebAssembly**: High-performance client-side execution
- **Responsive Design**: Mobile-friendly interface
- **Dark/Light Theme**: User preference support
- **Progressive Web App**: Offline capability and app-like experience

## 🚀 Quick Start

### Prerequisites

1. **Rust Toolchain**: Install the latest stable Rust
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Trunk**: Install Trunk for building and serving
   ```bash
   cargo install trunk
   ```

3. **wasm-bindgen-cli**: Install wasm-bindgen CLI
   ```bash
   cargo install wasm-bindgen-cli
   ```

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/gillean.git
   cd gillean/frontend
   ```

2. **Install dependencies**:
   ```bash
   cargo build
   ```

3. **Start development server**:
   ```bash
   trunk serve
   ```

4. **Open in browser**:
   Navigate to `http://localhost:8080`

### Production Build

1. **Build for production**:
   ```bash
   trunk build --release
   ```

2. **Serve static files**:
   ```bash
   # Using Python
   python3 -m http.server 8080
   
   # Using Node.js
   npx serve dist
   ```

## 📱 UI Components

### Blockchain Explorer
- **Block List**: View all blocks with details
- **Transaction List**: Browse all transactions
- **Search Functionality**: Search by block hash, transaction ID, or address
- **Filtering**: Filter by date, type, or status

### Transaction Creation
- **Send Form**: Create new transactions
- **Address Book**: Save and manage addresses
- **Transaction History**: View past transactions
- **Real-time Status**: Live transaction status updates

### Smart Contract Interface
- **Contract Deployment**: Deploy WASM contracts
- **Contract Interaction**: Call contract functions
- **Contract Templates**: Use pre-built contract templates
- **Contract Monitoring**: Monitor contract execution

### Wallet Management
- **Wallet Creation**: Create new encrypted wallets
- **Wallet Import**: Import existing wallets
- **Balance Display**: Real-time balance updates
- **Transaction History**: Wallet-specific transaction history

### Network Monitoring
- **Real-time Metrics**: Live blockchain statistics
- **Health Status**: Network health monitoring
- **Peer Information**: Connected peer details
- **Performance Metrics**: Throughput and latency

### Shard Dashboard
- **Shard Overview**: View all shards and their status
- **Cross-shard Transactions**: Monitor cross-shard activity
- **Shard Performance**: Individual shard metrics
- **Load Balancing**: Shard load distribution

### Cross-chain Interface
- **Bridge Status**: Monitor cross-chain bridge
- **Transfer Initiation**: Start cross-chain transfers
- **Transfer Tracking**: Track transfer status
- **External Chain Status**: Monitor connected chains

## 🔧 Configuration

### WebSocket Settings

Configure WebSocket connection in `src/api.rs`:

```rust
pub const WS_URL: &str = "ws://localhost:3000/api/ws";
pub const API_BASE_URL: &str = "http://localhost:3000/api";
```

### API Endpoints

The frontend communicates with the following API endpoints:

- `GET /api/blockchain/status` - Blockchain status
- `GET /api/blocks` - Block list
- `GET /api/transactions` - Transaction list
- `GET /api/wallets` - Wallet information
- `GET /api/contracts` - Smart contract information
- `GET /api/sharding/stats` - Sharding statistics
- `GET /api/bridge/status` - Cross-chain bridge status
- `POST /api/transactions` - Create transaction
- `POST /api/contracts/deploy` - Deploy contract
- `POST /api/contracts/call` - Call contract
- `WS /api/ws` - WebSocket for real-time updates

### Environment Variables

Set environment variables for configuration:

```bash
export GILLEAN_API_URL=http://localhost:3000/api
export GILLEAN_WS_URL=ws://localhost:3000/api/ws
export GILLEAN_ENVIRONMENT=development
```

## 🎨 Styling

### CSS Framework
The frontend uses a custom CSS framework with:
- **CSS Grid**: Modern layout system
- **Flexbox**: Flexible component layouts
- **CSS Variables**: Theme customization
- **Responsive Design**: Mobile-first approach

### Themes
- **Light Theme**: Default light appearance
- **Dark Theme**: Dark mode support
- **Custom Themes**: User-defined color schemes

### Components
- **Buttons**: Primary, secondary, and danger variants
- **Cards**: Information display containers
- **Forms**: Input fields and validation
- **Tables**: Data display with sorting
- **Modals**: Overlay dialogs
- **Notifications**: Toast messages and alerts

## 🔌 WebSocket Integration

### Real-time Features

The frontend uses WebSocket connections for real-time updates:

```rust
// WebSocket message types
#[derive(Serialize, Deserialize)]
pub enum WebSocketMessage {
    BlockCreated(Block),
    TransactionProcessed(Transaction),
    ShardUpdate(ShardStats),
    CrossChainUpdate(BridgeTransaction),
    ContractDeployed(ContractInfo),
    Error(String),
}
```

### Connection Management

- **Automatic Reconnection**: Reconnect on connection loss
- **Heartbeat**: Keep connection alive
- **Message Queuing**: Queue messages during disconnection
- **Error Handling**: Graceful error recovery

## 🧪 Testing

### Unit Tests

Run unit tests:

```bash
cargo test
```

### Integration Tests

Run integration tests:

```bash
cargo test --test integration
```

### Browser Tests

Run browser-based tests:

```bash
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome
```

## 📦 Building

### Development Build

```bash
trunk build
```

### Production Build

```bash
trunk build --release
```

### Optimized Build

```bash
# Enable optimizations
RUSTFLAGS="-C target-cpu=native" trunk build --release

# Enable link-time optimization
RUSTFLAGS="-C lto=fat" trunk build --release
```

## 🚀 Deployment

### Static Hosting

Deploy to static hosting services:

```bash
# Build for production
trunk build --release

# Deploy to Netlify
netlify deploy --prod --dir dist

# Deploy to Vercel
vercel --prod
```

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo install trunk
RUN trunk build --release

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

## 🔧 Development

### Project Structure

```
frontend/
├── src/
│   ├── main.rs              # Application entry point
│   ├── app.rs               # Main application component
│   ├── api.rs               # API client and WebSocket
│   ├── components/          # UI components
│   │   ├── blockchain_view.rs
│   │   ├── transaction_form.rs
│   │   ├── wallet_manager.rs
│   │   ├── contract_deploy.rs
│   │   ├── metrics_view.rs
│   │   ├── shard_dashboard.rs
│   │   └── cross_chain.rs
│   └── components.rs        # Component exports
├── index.html               # HTML template
├── styles.css               # Global styles
├── Cargo.toml               # Dependencies
└── README.md                # This file
```

### Adding New Components

1. **Create component file**:
   ```rust
   // src/components/my_component.rs
   use yew::prelude::*;

   #[function_component(MyComponent)]
   pub fn my_component() -> Html {
       html! {
           <div class="my-component">
               {"Hello, World!"}
           </div>
       }
   }
   ```

2. **Export component**:
   ```rust
   // src/components.rs
   pub mod my_component;
   pub use my_component::MyComponent;
   ```

3. **Use in app**:
   ```rust
   use crate::components::MyComponent;

   html! {
       <MyComponent />
   }
   ```

### Styling Components

Add component-specific styles:

```css
/* styles.css */
.my-component {
    padding: 1rem;
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    background: var(--background-color);
}

.my-component:hover {
    border-color: var(--primary-color);
}
```

## 🐛 Troubleshooting

### Common Issues

1. **WebSocket Connection Failed**:
   - Check if the backend is running
   - Verify WebSocket URL configuration
   - Check browser console for errors

2. **Build Errors**:
   - Update Rust toolchain: `rustup update`
   - Clean and rebuild: `cargo clean && cargo build`
   - Check dependency versions

3. **Performance Issues**:
   - Enable release mode: `trunk build --release`
   - Check for memory leaks in components
   - Optimize WebSocket message handling

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug trunk serve
```

### Browser Developer Tools

Use browser developer tools for debugging:
- **Console**: View logs and errors
- **Network**: Monitor API requests
- **WebSocket**: Inspect WebSocket messages
- **Performance**: Profile application performance

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Development Guidelines

- Follow Rust coding standards
- Add documentation for new components
- Include tests for new functionality
- Ensure responsive design
- Test across different browsers

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🙏 Acknowledgments

- Yew framework for Rust-based frontend development
- WebAssembly for high-performance client-side execution
- Modern web standards for responsive design
- WebSocket API for real-time communication
