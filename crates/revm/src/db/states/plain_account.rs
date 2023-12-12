#[cfg(feature = "enable_cache_record")]
use hashbrown::hash_map::DefaultHashBuilder;
use revm_interpreter::primitives::{AccountInfo, HashMap, StorageSlot, U256};
#[cfg(feature = "enable_cache_record")]
use revm_utils::TrackingAllocator;

// TODO rename this to BundleAccount. As for the block level we have original state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlainAccount {
    pub info: AccountInfo,
    pub storage: PlainStorage,
}

impl PlainAccount {
    pub fn new_empty_with_storage(storage: PlainStorage) -> Self {
        Self {
            info: AccountInfo::default(),
            storage,
        }
    }

    pub fn into_components(self) -> (AccountInfo, PlainStorage) {
        (self.info, self.storage)
    }
}

/// This storage represent values that are before block changed.
///
/// Note: Storage that we get EVM contains original values before t
#[cfg(not(feature = "enable_cache_record"))]
pub type StorageWithOriginalValues = HashMap<U256, StorageSlot>;
#[cfg(feature = "enable_cache_record")]
pub type StorageWithOriginalValues =
    HashMap<U256, StorageSlot, DefaultHashBuilder, TrackingAllocator>;

/// Simple plain storage that does not have previous value.
/// This is used for loading from database, cache and for bundle state.
#[cfg(not(feature = "enable_cache_record"))]
pub type PlainStorage = HashMap<U256, U256>;
#[cfg(feature = "enable_cache_record")]
pub type PlainStorage = HashMap<U256, U256, DefaultHashBuilder, TrackingAllocator>;

impl From<AccountInfo> for PlainAccount {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            #[cfg(not(feature = "enable_cache_record"))]
            storage: HashMap::new(),
            #[cfg(feature = "enable_cache_record")]
            storage: HashMap::new_in(TrackingAllocator),
        }
    }
}
