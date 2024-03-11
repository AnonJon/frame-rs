use anyhow::{bail, Result};
use ethers::{
    middleware::Middleware,
    providers::{Http, Provider},
    types::{Address, TransactionRequest, H256, U256},
};
use reqwest::Client;
use serde_json::json;
use std::{convert::TryFrom, sync::Arc};

#[derive(Clone)]
pub struct FrameClient {
    pub provider: Arc<Provider<Http>>,
    rpc_url: String,
}

impl FrameClient {
    /// Creates a new instance of `FrameClient` configured to interact with the Ethereum network
    /// specified by the given `chain_id`. This method initializes the connection to the Frame
    /// wallet via its local RPC endpoint and attempts to switch the network to the specified `chain_id`.
    /// The chain id must be already added to the Frame wallet's network list.
    ///
    /// # Parameters
    /// - `chain_id`: The chain ID of the Ethereum network you want to connect to. This must be provided
    /// as a `U256` value that corresponds to the desired network. For example, to connect to the Ethereum
    /// Mainnet, you would pass `U256::from(1)`.
    /// - `host`: The host address of the Frame wallet's RPC endpoint. This is optional and defaults to
    /// 127.0.0.1. If the Frame wallet is running on a different host, you can specify it here.
    ///
    /// # Returns
    /// Returns a `Result` wrapping a new `FrameClient` instance if the connection and network switch
    /// were successful. If there is an error in connecting to the Frame wallet's RPC endpoint or in
    /// switching the network, an error wrapped in `Result` is returned instead.
    ///
    /// # Examples
    /// ```no_run
    /// use ethers::types::U256;
    /// use frame_rs::client::FrameClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let chain_id = U256::from(1); // Ethereum Mainnet
    ///     let client = FrameClient::new(chain_id, Some("0.0.0.0")).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    /// This method will return an error if the connection to the Frame wallet cannot be established,
    /// or if the network switch request fails. The error will contain details about the failure.
    /// It will not return an error for an invalid chain ID, as the chain ID is not validated here.
    pub async fn new(chain_id: U256, host: Option<&str>) -> Result<Self> {
        let host = host.unwrap_or("127.0.0.1");
        let rpc_url = format!("http://{}:1248", host);
        let provider = Arc::new(Provider::<Http>::try_from(rpc_url.clone())?);
        let client = Self { provider, rpc_url };

        client.switch_network(chain_id).await?;

        Ok(client)
    }

    /// Retrieves the chain ID of the currently connected Ethereum network.
    ///
    /// This method queries the connected Ethereum node (through Frame's RPC endpoint)
    /// to determine the chain ID of the network it's currently interacting with. This
    /// can be useful for confirming that the `FrameClient` is connected to the expected
    /// network, especially after a network switch operation.
    ///
    /// # Returns
    /// A `Result` that, on success, wraps the `U256` chain ID of the currently connected network.
    /// If the query fails for any reason (such as a connection issue), an error is returned
    /// encapsulating the failure's details.
    ///
    /// # Examples
    /// ```no_run
    /// use ethers::types::U256;
    /// use frame_rs::client::FrameClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let chain_id = U256::from(1); // Example chain_id, e.g., Ethereum Mainnet
    ///     let client = FrameClient::new(chain_id, None).await?;
    ///     
    ///     let current_chain_id = client.get_chain_id().await?;
    ///     println!("Current Chain ID: {}", current_chain_id);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    /// If there is an issue in fetching the chain ID from the Ethereum node, an error
    /// is returned detailing the problem. This could be due to network connectivity issues,
    /// or the Ethereum node not responding properly.
    pub async fn get_chain_id(&self) -> Result<U256> {
        let chain_id = self.provider.get_chainid().await?;
        Ok(chain_id)
    }

    /// Attempts to switch the connected Ethereum network in the Frame wallet to the specified `chain_id`.
    ///
    ///
    /// # Parameters
    /// - `chain_id`: The chain ID of the Ethereum network you wish to switch to, provided as a `U256`.
    /// The `chain_id` should be in hexadecimal format, but this method will handle the conversion
    /// for you. For example, to switch to the Ethereum Mainnet, you would pass `U256::from(1)`.
    ///
    /// # Returns
    /// Returns `Ok(())` if the network switch request was successfully sent and acknowledged by the
    /// Frame wallet without errors. If the request fails, for example due to connection issues or
    /// because the requested network is not supported/configured in Frame, an error is returned
    /// detailing the issue.
    ///
    /// # Examples
    /// ```no_run
    /// use ethers::types::U256;
    /// use frame_rs::client::FrameClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let client = FrameClient::new(U256::from(1), None).await?; // Assuming Ethereum Mainnet
    ///     let rinkeby_chain_id = U256::from(42161); // Arbitrum One
    ///
    ///     client.switch_network(rinkeby_chain_id).await?;
    ///     println!("Switched to Arbitrum mainnet.");
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    /// If the network switch cannot be completed, an error is returned with details about the failure.
    /// This might occur if the Frame wallet is not accessible. It will not error for an invalid chain ID.
    pub async fn switch_network(&self, chain_id: U256) -> Result<()> {
        let client = Client::new();
        let chain_id_hex = format!("{:#x}", chain_id);

        let params = json!([{
            "chainId": chain_id_hex,
        }]);

        let response = client
            .post(self.rpc_url.clone())
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "wallet_switchEthereumChain",
                "params": params,
                "id": "1"
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            bail!("Failed to switch network: {}", error_text);
        }

        Ok(())
    }

    /// Sends a specified amount of the native gas token (e.g., ETH on Ethereum) from one address to another.
    ///
    /// This asynchronous method constructs and sends a transaction that transfers the native
    /// blockchain currency (like ETH) from the specified sender address to the recipient address.
    ///
    /// # Parameters
    /// - `from`: The `Address` from which the gas token will be sent.
    /// - `to`: The `Address` to which the gas token will be sent.
    /// - `amount`: The amount of the gas token to send, specified in Wei as a `U256`.
    ///
    /// # Returns
    /// Returns a `Result` that, on success, wraps the `H256` transaction hash of the sent transaction.
    /// If the transaction fails to send, an error is returned detailing the failure.
    ///
    /// # Examples
    /// ```no_run
    /// use ethers::types::{Address, U256};
    /// use frame_rs::client::FrameClient;
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let client = FrameClient::new(U256::from(1), None).await?;
    ///     let from: Address = "0x...".parse()?;
    ///     let to: Address = "0x...".parse()?;
    ///     let amount = U256::from(1000000000000000000u64); // 1 ETH in Wei
    ///
    ///     let tx_hash = client.send_gas_token(from, to, amount).await?;
    ///     println!("Transaction hash: {:?}", tx_hash);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns an error if the transaction fails to be sent or if there is an issue with
    /// the transaction's execution.
    pub async fn send_gas_token(&self, from: Address, to: Address, amount: U256) -> Result<H256> {
        let tx = TransactionRequest::new().from(from).to(to).value(amount);
        let pending_tx = self.provider.send_transaction(tx, None).await?;
        let tx_receipt = pending_tx.await?;
        if let Some(tx_hash) = tx_receipt {
            return Ok(tx_hash.transaction_hash);
        }

        bail!("Tx failed to send");
    }

    /// Retrieves a list of addresses owned by the connected wallet.
    ///
    /// This asynchronous method queries the connected Ethereum provider (e.g., Frame) for
    /// the list of accounts it manages.
    ///
    /// # Returns
    /// Returns a `Result` that, on success, wraps a vector of `Address`es representing the
    /// accounts managed by the connected provider. If the query fails, an error is returned
    /// with details about the failure.
    ///
    /// # Examples
    /// ```no_run
    /// use frame_rs::client::FrameClient;
    /// use anyhow::Result;
    /// use ethers::types::U256;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let client = FrameClient::new(U256::from(1), None).await?;
    ///     let accounts = client.get_accounts().await?;
    ///
    ///     for account in accounts {
    ///         println!("Account address: {:?}", account);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns an error if there is an issue fetching the accounts from the connected provider.
    pub async fn get_accounts(&self) -> Result<Vec<Address>> {
        let accounts = self.provider.get_accounts().await?;
        Ok(accounts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_switch_network() {
        let client = FrameClient::new(U256::from(1), None).await.unwrap();
        assert_eq!(client.get_chain_id().await.unwrap(), U256::from(1));
        let next_chain_id = U256::from(42161);
        client.switch_network(next_chain_id).await.unwrap();
        assert_eq!(client.get_chain_id().await.unwrap(), next_chain_id);
    }
}
