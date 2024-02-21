# Frame-rs

`frame-rs` is a Rust library designed to facilitate interaction with the [Frame](https://frame.sh/) Ethereum wallet. It provides a straightforward, asynchronous API for switching Ethereum networks via Frame's JSON-RPC interface and performing Ethereum blockchain operations, leveraging the security and user-friendliness of Frame for managing accounts and signing transactions.

## Features

- **Network Switching**: Programmatically switch the connected Ethereum network in Frame.
- **Ethereum Operations**: Simplified interface for common Ethereum operations, such as sending transactions and interacting with smart contracts, using accounts managed by Frame.
- **Async/Await Support**: Built with asynchronous Rust features for non-blocking I/O operations.

## Requirements

- Rust 1.39 or later.
- Access to Frame's JSON-RPC interface (typically via localhost).

## Installation

Add `frame-rs` to your Cargo.toml:

```toml
[dependencies]
frame-rs = "0.1.0"
```

## Quick Start

Here's a quick example to get you started with `frame-rs`:

```rust
use frame_rs::FrameClient;
use ethers::types::U256;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the Frame client
    let client = FrameClient::new(U256::from(1)).await?; // Ethereum Mainnet

    // Switch to Arbitrum One
    let rinkeby_chain_id = U256::from(42161);
    client.switch_network(rinkeby_chain_id).await?;

    println!("Successfully switched to Arbitrum One.");
    Ok(())
}
```

### **Usage**

#### Creating a Client

To start interacting with Frame, create a `FrameClient` instance:

```rust
    use frame_rs::FrameClient;
    use ethers::types::U256;

    #[tokio::main]
    async fn main() {
      let chain_id = U256::from(1); // Example for Ethereum Mainnet
      let client = FrameClient::new(chain_id).await.expect("Failed to create FrameClient");
    }
```

#### Switching Networks

To switch the connected network:

```rust
    use ethers::types::U256;

    #[tokio::main]
    async fn main() {
      let new_chain_id = U256::from(42161); // Example for Arbitrum One
      let client = FrameClient::new(U256::from(1)).await.expect("Failed to create FrameClient");
      client.switch_network(new_chain_id).await.expect("Failed to switch network");
    }
```

#### More Operations

`frame-rs` aims to support additional Ethereum wallet operations. Stay tuned for more features!

### **License**

`frame-rs` is licensed under the MIT License - see the `LICENSE` file for details.
