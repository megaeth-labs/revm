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
    use crate::{Account, AccountId, AccountInfo, TransactionId};
    use bytecode::Bytecode;
    use primitives::{Bytes, U256};

    fn info(balance: u64, nonce: u64) -> AccountInfo {
        AccountInfo::from_balance(U256::from(balance))
            .with_code(Bytecode::new_legacy(Bytes::from_static(&[0x60, 0x00])))
            .with_nonce(nonce)
    }

    /// [`info`] with an account id, which `AccountInfo`'s `PartialEq` ignores.
    fn info_with_id(balance: u64, nonce: u64, id: usize) -> AccountInfo {
        let mut info = info(balance, nonce);
        info.account_id = AccountId::new(id);
        info
    }

    /// Asserts the reference is the info `original_info` returns, field by field: `AccountInfo`'s
    /// `PartialEq` ignores the account id and the cached code.
    #[track_caller]
    fn assert_parity(account: &Account) {
        let by_ref = account.original_info_ref();
        let cloned = account.original_info();
        assert_eq!(by_ref.balance, cloned.balance);
        assert_eq!(by_ref.nonce, cloned.nonce);
        assert_eq!(by_ref.code_hash, cloned.code_hash);
        assert_eq!(by_ref.code, cloned.code);
        assert_eq!(by_ref.account_id, cloned.account_id);
    }

    /// An account whose original info was never set reads the default info, code and account id
    /// included, whatever its info and status are now.
    #[test]
    fn test_an_unset_original_reads_the_default_info() {
        let mut account = Account::new_not_existing(TransactionId::ZERO);
        account.info = info_with_id(5, 1, 3);
        assert_parity(&account);
        let original = account.original_info_ref();
        assert!(original.is_default());
        assert_eq!(*original, AccountInfo::default());
        assert_eq!(original.code, Some(Bytecode::default()));
        assert_eq!(original.account_id, None);

        account.mark_created();
        assert_parity(&account);
        assert!(account.original_info_ref().is_default());
        assert_eq!(account.original_info_ref().account_id, None);

        account.mark_selfdestruct();
        assert_parity(&account);
        assert!(account.original_info_ref().is_default());

        account.unmark_created();
        assert_parity(&account);
        assert!(account.original_info_ref().is_default());
        assert_eq!(account.original_info_ref().account_id, None);
    }

    /// A set original is read where the account holds it, not from a copy, whatever the account's
    /// status, and follows every change made through `original_info_mut` and
    /// `set_current_info_as_original`.
    #[test]
    fn test_a_set_original_is_read_in_place() {
        let mut account = Account::from(info_with_id(5, 1, 7));
        account.info = info_with_id(7, 2, 8);
        assert_parity(&account);
        assert_eq!(account.original_info_ref().balance, U256::from(5));
        assert_eq!(account.original_info_ref().nonce, 1);
        assert_eq!(account.original_info_ref().account_id, AccountId::new(7));
        assert!(core::ptr::eq(
            account.original_info_ref(),
            account.original_info.as_deref().unwrap()
        ));

        account.mark_created();
        assert_parity(&account);
        assert_eq!(account.original_info_ref().nonce, 1);
        assert_eq!(account.original_info_ref().account_id, AccountId::new(7));

        account.mark_selfdestruct();
        assert_parity(&account);
        assert_eq!(account.original_info_ref().nonce, 1);

        account.unmark_created();
        assert_parity(&account);
        assert_eq!(account.original_info_ref().balance, U256::from(5));
        assert_eq!(account.original_info_ref().account_id, AccountId::new(7));

        account.original_info_mut().nonce = 9;
        account.original_info_mut().account_id = AccountId::new(11);
        assert_parity(&account);
        assert_eq!(account.original_info_ref().nonce, 9);
        assert_eq!(account.original_info_ref().account_id, AccountId::new(11));

        account.set_current_info_as_original();
        assert_parity(&account);
        assert_eq!(account.original_info_ref().balance, U256::from(7));
        assert_eq!(account.original_info_ref().nonce, 2);
        assert_eq!(account.original_info_ref().account_id, AccountId::new(8));
        assert!(core::ptr::eq(
            account.original_info_ref(),
            account.original_info.as_deref().unwrap()
        ));
    }
}
