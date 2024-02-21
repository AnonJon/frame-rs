use anyhow::{bail, Result};
use ethers::{
    middleware::Middleware,
    providers::{Http, Provider},
    types::U256,
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
    ///
    /// # Returns
    /// Returns a `Result` wrapping a new `FrameClient` instance if the connection and network switch
    /// were successful. If there is an error in connecting to the Frame wallet's RPC endpoint or in
    /// switching the network, an error wrapped in `Result` is returned instead.
    ///
    /// # Examples
    /// ```no_run
    /// use ethers::types::U256;
    /// use frame_rs::FrameClient; // Adjust this to the actual path where FrameClient is defined
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let chain_id = U256::from(1); // Ethereum Mainnet
    ///     let client = FrameClient::new(chain_id).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    /// This method will return an error if the connection to the Frame wallet cannot be established,
    /// or if the network switch request fails. The error will contain details about the failure.
    /// It will not return an error for an invalid chain ID, as the chain ID is not validated here.
    pub async fn new(chain_id: U256) -> Result<Self> {
        let rpc_url = "http://127.0.0.1:1248".to_string();
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
    /// use frame_rs::FrameClient; // Adjust this to the actual path where FrameClient is defined
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let chain_id = U256::from(1); // Example chain_id, e.g., Ethereum Mainnet
    ///     let client = FrameClient::new(chain_id).await?;
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
    /// use frame_rs::FrameClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let client = FrameClient::new(U256::from(1)).await?; // Assuming Ethereum Mainnet
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
    async fn switch_network(&self, chain_id: U256) -> Result<()> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_switch_network() {
        let client = FrameClient::new(U256::from(1)).await.unwrap();
        assert_eq!(client.get_chain_id().await.unwrap(), U256::from(1));
        let next_chain_id = U256::from(42161);
        client.switch_network(next_chain_id).await.unwrap();
        assert_eq!(client.get_chain_id().await.unwrap(), next_chain_id);
    }
}
