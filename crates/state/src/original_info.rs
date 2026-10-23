//! The account's original info, read by reference.

use crate::{Account, AccountInfo};
use primitives::OnceLock;

impl Account {
    /// Returns a reference to the original account info: the info
    /// [`original_info`](Self::original_info) returns, without cloning it.
    ///
    /// An account whose original info was never set reads the default info, as it does through
    /// `original_info`.
    #[inline]
    pub fn original_info_ref(&self) -> &AccountInfo {
        match self.original_info.as_deref() {
            Some(info) => info,
            None => default_info(),
        }
    }
}

/// The default account info, built once and shared.
fn default_info() -> &'static AccountInfo {
    static DEFAULT: OnceLock<AccountInfo> = OnceLock::new();
    DEFAULT.get_or_init(AccountInfo::default)
}

#[cfg(test)]
mod tests {
    use crate::{Account, AccountInfo, TransactionId};
    use bytecode::Bytecode;
    use primitives::{Bytes, U256};

    fn info(balance: u64, nonce: u64) -> AccountInfo {
        AccountInfo::from_balance(U256::from(balance))
            .with_code(Bytecode::new_legacy(Bytes::from_static(&[0x60, 0x00])))
            .with_nonce(nonce)
    }

    /// An account whose original info was never set reads the default info, code included,
    /// whatever its info is now.
    #[test]
    fn test_an_unset_original_reads_the_default_info() {
        let mut account = Account::new_not_existing(TransactionId::ZERO);
        account.info = info(5, 1);
        let original = account.original_info_ref();
        assert_eq!(*original, account.original_info());
        assert!(original.is_default());
    }

    /// A set original is read where the account holds it, not from a copy, and follows every
    /// change made through `original_info_mut`.
    #[test]
    fn test_a_set_original_is_read_in_place() {
        let mut account = Account::from(info(5, 1));
        account.info = info(7, 2);
        assert_eq!(*account.original_info_ref(), info(5, 1));
        assert!(core::ptr::eq(
            account.original_info_ref(),
            account.original_info.as_deref().unwrap()
        ));

        account.original_info_mut().nonce = 9;
        assert_eq!(account.original_info_ref().nonce, 9);
        assert_eq!(*account.original_info_ref(), account.original_info());
    }
}
