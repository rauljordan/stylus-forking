#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use stylus_sdk::{alloy_primitives::U256, host::*, prelude::*, storage::StorageU256};

#[storage]
// #[entrypoint]
pub struct Foo<'b, H: Host> {
    number: StorageU256<'b, H>,
    host: &'b H,
}

#[public]
impl<'b, H: Host> Foo<'b, H> {
    pub fn check_balance(&self) -> u64 {
        let _ = self.host.msg_sender();
        self.host.block_gas_limit()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_my_contract() {
        use super::*;
        let host = stylus_sdk::testing::MockHost::default();
        let foo = unsafe { Foo::new(U256::ZERO, 0, &host) };
        assert_eq!(foo.check_balance(), 0u64);
    }
}
