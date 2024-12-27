#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use core::marker::PhantomData;

use alloy_primitives::{Address, U256, U64};
use stylus_sdk::{
    host::*,
    prelude::*,
    storage::{StorageU256, StorageU64},
};

#[storage]
// #[entrypoint]
pub struct Foo<H: Host> {
    number: StorageU256<H>,
    host: *const H,
}

#[public]
impl<H: Host> Foo<H> {
    pub fn check_sender(&self) -> Address {
        self.get_host().msg_sender()
    }
    pub fn set_number(&mut self, num: U256) {
        self.number.set(num)
    }
    pub fn number(&self) -> U256 {
        self.number.get()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloy_primitives::{address, FixedBytes, B256, U256};
    use ethers::core::types::H256;
    use ethers::middleware::Middleware;
    use ethers::providers::{Http, Provider};
    use ethers::types::{BigEndianHash, NameOrAddress, H160};
    use std::collections::HashMap;
    use std::sync::Arc;
    use stylus_sdk::testing::CheatcodeProvider;
    use tokio::runtime::Runtime;

    #[derive(Default)]
    pub struct MockHost {
        sender: Address,
        contract_address: Address,
        provider: Option<Arc<Provider<Http>>>,
        rpc_url: String,
        storage: HashMap<Address, HashMap<U256, B256>>,
        block_num: Option<u64>,
        block_basefee: U256,
        block_timestamp: u64,
    }

    #[derive(Default)]
    pub struct MockHostBuilder {
        sender: Option<Address>,
        contract_address: Option<Address>,
        rpc_url: Option<String>,
        storage: Option<HashMap<Address, HashMap<U256, B256>>>,
        provider: Option<Arc<Provider<Http>>>,
        block_num: Option<u64>,
        block_basefee: Option<U256>,
        block_timestamp: Option<u64>,
    }

    impl MockHostBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn sender(mut self, sender: Address) -> Self {
            self.sender = Some(sender);
            self
        }

        pub fn contract_address(mut self, address: Address) -> Self {
            self.contract_address = Some(address);
            self
        }

        pub fn rpc_url(mut self, url: String, block_num: Option<u64>) -> Self {
            self.rpc_url = Some(url);
            self.block_num = block_num;
            if let Some(url) = &self.rpc_url {
                if let Ok(provider) = Provider::<Http>::try_from(url.as_str()) {
                    self.provider = Some(Arc::new(provider));
                }
            }
            self
        }

        pub fn build(self) -> Result<MockHost, &'static str> {
            Ok(MockHost {
                sender: self.sender.unwrap_or(Address::ZERO),
                storage: self.storage.unwrap_or_default(),
                block_num: self.block_num,
                block_basefee: self.block_basefee.unwrap_or(U256::ZERO),
                block_timestamp: self.block_timestamp.unwrap_or(0),
                contract_address: self.contract_address.unwrap_or(Address::ZERO),
                rpc_url: self
                    .rpc_url
                    .unwrap_or_else(|| "https://sepolia-rollup.arbitrum.io/rpc".to_string()),
                provider: self.provider,
            })
        }
    }

    impl Host for MockHost {}

    impl CryptographyAccess for MockHost {
        fn native_keccak256(&self, _input: &[u8]) -> FixedBytes<32> {
            FixedBytes::<32>::default()
        }
    }

    impl CalldataAccess for MockHost {
        fn args(&self, _len: usize) -> Vec<u8> {
            Vec::new()
        }
        fn read_return_data(&self, _offset: usize, _size: Option<usize>) -> Vec<u8> {
            Vec::new()
        }
        fn return_data_len(&self) -> usize {
            0
        }
        fn output(&self, _data: &[u8]) {}
    }

    impl DeploymentAccess for MockHost {
        fn create1(&self) {}
        fn create2(&self) {}
    }

    impl StorageAccess for MockHost {
        fn emit_log(&self, _input: &[u8]) {}
        fn load(&self, key: U256) -> B256 {
            if let Some(provider) = self.ensure_provider() {
                // Create a new runtime for the blocking call
                let rt = Runtime::new().expect("Failed to create runtime");

                // Convert U256 to H256 for the storage slot
                let slot_bytes: &[u8; 32] = &key.to_be_bytes();
                let slot = H256::from_slice(&slot_bytes[..]);

                // Execute the async call in a blocking context
                let addr =
                    NameOrAddress::Address(H160::from_slice(&self.contract_address.as_slice()));
                let storage = rt
                    .block_on(async { provider.get_storage_at(addr, slot, None).await })
                    .unwrap_or_default();
                return B256::from_slice(storage.as_bytes());
            } else {
                // Fallback to local storage if no provider
                self.storage
                    .get(&self.contract_address)
                    .and_then(|contract_storage| contract_storage.get(&key))
                    .copied()
                    .unwrap_or_default()
            }
        }
        fn cache(&self, _key: U256, _value: B256) {}
        fn flush_cache(&self, _clear: bool) {}
    }

    impl CallAccess for MockHost {
        fn call_contract(&self) {}
        fn static_call_contract(&self) {}
        fn delegate_call_contract(&self) {}
    }

    impl BlockAccess for MockHost {
        fn block_basefee(&self) -> U256 {
            U256::ZERO
        }
        fn block_coinbase(&self) -> Address {
            Address::ZERO
        }
        fn block_number(&self) -> u64 {
            0
        }
        fn block_timestamp(&self) -> u64 {
            0
        }
        fn block_gas_limit(&self) -> u64 {
            0
        }
    }

    impl ChainAccess for MockHost {
        fn chain_id(&self) -> u64 {
            0
        }
    }

    impl AccountAccess for MockHost {
        fn balance(&self, _account: Address) -> U256 {
            U256::ZERO
        }
        fn contract_address(&self) -> Address {
            Address::ZERO
        }
        fn code(&self, _account: Address) -> Vec<u8> {
            Vec::new()
        }
        fn code_size(&self, _account: Address) -> usize {
            0
        }
        fn codehash(&self, _account: Address) -> FixedBytes<32> {
            FixedBytes::<32>::default()
        }
    }

    impl MemoryAccess for MockHost {
        fn pay_for_memory_grow(&self, _pages: u16) {}
    }

    impl MessageAccess for MockHost {
        fn msg_sender(&self) -> Address {
            self.sender
        }
        fn msg_reentrant(&self) -> bool {
            false
        }
        fn msg_value(&self) -> U256 {
            U256::ZERO
        }
        fn tx_origin(&self) -> Address {
            Address::ZERO
        }
    }

    impl MeteringAccess for MockHost {
        fn evm_gas_left(&self) -> u64 {
            0
        }
        fn evm_ink_left(&self) -> u64 {
            0
        }
        fn tx_gas_price(&self) -> U256 {
            U256::ZERO
        }
        fn tx_ink_price(&self) -> u32 {
            0
        }
    }

    impl MockHost {
        fn ensure_provider(&self) -> Option<Arc<Provider<Http>>> {
            if self.provider.is_none() && !self.rpc_url.is_empty() {
                // Create provider if we have an RPC URL but no provider yet
                if let Ok(provider) = Provider::<Http>::try_from(&self.rpc_url) {
                    return Some(Arc::new(provider));
                }
            }
            self.provider.clone()
        }
    }

    #[test]
    fn test_my_contract() {
        let host = MockHostBuilder::new()
            .contract_address(address!("2460d3db27c4bef88557e8dc9136e6fad189e8c3"))
            .rpc_url("https://sepolia-rollup.arbitrum.io/rpc".to_string(), None)
            .build()
            .unwrap();

        let foo = unsafe { Foo::new(U256::ZERO, 0, &host) };
        assert_eq!(foo.number(), U256::from(5));
    }
}
