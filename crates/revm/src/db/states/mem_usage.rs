//! This module defines the method for obtaining the memory size occupied by the State.
use super::{
    cache::CacheState, transition_account::TransitionAccount, BundleAccount, BundleState,
    CacheAccount, State, TransitionState,
};
use revm_interpreter::primitives::{db::Database, AccountInfo};

/// This trait is used to support types in obtaining the dynamically allocated memory
/// size used by them
pub trait DynMemUsage {
    fn dyn_mem_usage(&self) -> usize;
}

impl DynMemUsage for AccountInfo {
    fn dyn_mem_usage(&self) -> usize {
        self.code.as_ref().map(|c| c.len()).unwrap_or(0)
    }
}

impl DynMemUsage for CacheAccount {
    fn dyn_mem_usage(&self) -> usize {
        self.account
            .as_ref()
            .map(|a| a.info.dyn_mem_usage())
            .unwrap_or(0)
    }
}

impl DynMemUsage for CacheState {
    fn dyn_mem_usage(&self) -> usize {
        let accounts_dyn_size = self
            .accounts
            .iter()
            .map(|(_k, v)| v.dyn_mem_usage())
            .sum::<usize>();
        let contracts_dyn_size = self.contracts.iter().map(|(_k, v)| v.len()).sum::<usize>();
        accounts_dyn_size + contracts_dyn_size
    }
}

impl DynMemUsage for TransitionAccount {
    fn dyn_mem_usage(&self) -> usize {
        let info_dyn_size = self.info.as_ref().map(|a| a.dyn_mem_usage()).unwrap_or(0);

        let pre_info_dyn_size = self
            .previous_info
            .as_ref()
            .map(|a| a.dyn_mem_usage())
            .unwrap_or(0);

        info_dyn_size + pre_info_dyn_size
    }
}

impl DynMemUsage for TransitionState {
    fn dyn_mem_usage(&self) -> usize {
        self.transitions
            .iter()
            .map(|(_k, v)| v.dyn_mem_usage())
            .sum::<usize>()
    }
}

impl DynMemUsage for BundleAccount {
    fn dyn_mem_usage(&self) -> usize {
        let info_dyn_size = self.info.as_ref().map(|v| v.dyn_mem_usage()).unwrap_or(0);
        let original_info_dyn_size = self
            .original_info
            .as_ref()
            .map(|v| v.dyn_mem_usage())
            .unwrap_or(0);
        info_dyn_size + original_info_dyn_size
    }
}

impl DynMemUsage for BundleState {
    fn dyn_mem_usage(&self) -> usize {
        let state_dyn_size = self
            .state
            .iter()
            .map(|(_, v)| v.dyn_mem_usage())
            .sum::<usize>();
        let contracts_dyn_size = self.contracts.iter().map(|(_, v)| v.len()).sum::<usize>();
        state_dyn_size + contracts_dyn_size
    }
}

impl<DB: Database> State<DB> {
    fn dyn_mem_size(&self) -> usize {
        // Calculate the memory size of the State on the heap (excluding the HashMap section).
        let cache = self.cache.dyn_mem_usage();
        let transaction_state = self
            .transition_state
            .as_ref()
            .map(|v| v.dyn_mem_usage() + std::mem::size_of::<TransitionState>())
            .unwrap_or(0);
        let bundle_state = self.bundle_state.dyn_mem_usage();
        // block_hashes is a BTreeMap, and here we use the following formula to estimate its
        // memory usage:
        //          memory_size = ( sizeof(key) + sizeof(value) ) * block_hashes.len()
        let block_hashes = self.block_hashes.len() * (64 + 32);

        // The size of the hashmap calculated using a memory allocator.
        let map_size = revm_utils::allocator::stats().diff as usize;

        // Total dynamic memory size.
        let total_dyn_size = cache + transaction_state + bundle_state + block_hashes + map_size;
        println!("cache_heap_size: {:?}", cache);
        println!("transaction_size: {:?}", transaction_state);
        println!("bundle_state: {:?}", bundle_state);
        println!("block_hashes_size: {:?}", block_hashes);
        println!("map_size: {:?}", map_size);
        println!("total_dyn_size: {:?}", total_dyn_size);

        total_dyn_size
    }

    fn static_mem_size(&self) -> usize {
        let state_size = std::mem::size_of::<State<DB>>();
        println!("state_size: {:?}", state_size);
        state_size
    }

    pub fn mem_usage(&self) -> usize {
        let total = self.dyn_mem_size() + self.static_mem_size();
        println!("total_size: {:?}", total);
        total
    }
}
