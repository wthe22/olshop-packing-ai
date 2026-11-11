//! Accounts (accounts.md): the owner's groups of shops, as `packing-settings.toml` lists them, the
//! rules for their names, and the account an order's label belongs to.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::label::SenderText;
use crate::sorting::default_file_name;

/// An account name is at most this many characters.
pub const ACCOUNT_LIMIT: usize = 40;
/// A shop name is at most this many characters.
pub const SHOP_LIMIT: usize = 80;

/// One account of `packing-settings.toml`: a name and the shop names its labels are sent by.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    #[serde(default)]
    pub shops: Vec<String>,
}

/// The account an order was read as, and the listed shop found on its label: none when the label
/// printed no sender.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderAccount {
    pub account: String,
    pub shop: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameError {
    /// Empty, too long, or (an account) not what the default file-name rule makes of it.
    NotAName,
    /// Another account has it, upper and lower case counting as the same.
    AccountTaken,
    /// An account already lists it, spaces and upper and lower case ignored.
    ShopListed { account: String },
    /// No account has that name (another copy of the app may have renamed it).
    NoAccount,
}

/// Where the shop prompt puts a shop: an account already listed, or a new one by this name.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShopAccount {
    Listed(String),
    New(String),
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameError::NotAName => f.write_str(
                "an account is 1 to 40 letters, digits, spaces, - or _, with no space at either \
                 end; a shop is 1 to 80 characters",
            ),
            NameError::AccountTaken => f.write_str("another account has that name"),
            NameError::ShopListed { account } => write!(f, "{account} already lists that shop"),
            NameError::NoAccount => f.write_str("there is no account by that name"),
        }
    }
}

/// Whether `name` can be an account's name beside `accounts`; `renamed` is the account whose name
/// it replaces, which may differ from it in case only.
pub fn check_account_name(
    name: &str,
    accounts: &[Account],
    renamed: Option<&str>,
) -> Result<(), NameError> {
    let length = name.chars().count();
    if length == 0 || length > ACCOUNT_LIMIT || default_file_name(name) != name {
        return Err(NameError::NotAName);
    }
    let taken = accounts
        .iter()
        .filter(|account| Some(account.name.as_str()) != renamed)
        .any(|account| account.name.to_lowercase() == name.to_lowercase());
    if taken {
        return Err(NameError::AccountTaken);
    }
    Ok(())
}

/// Whether `name` can be a new shop of an account: a shop belongs to one account only.
pub fn check_shop_name(name: &str, accounts: &[Account]) -> Result<(), NameError> {
    let length = name.trim().chars().count();
    if length == 0 || length > SHOP_LIMIT {
        return Err(NameError::NotAName);
    }
    for account in accounts {
        if account.shops.iter().any(|shop| same_shop(shop, name)) {
            return Err(NameError::ShopListed {
                account: account.name.clone(),
            });
        }
    }
    Ok(())
}

/// Add `shop` to the account `to` names, making the account when it is new. Both names are checked
/// first: nothing changes when one is refused.
pub fn add_shop(
    accounts: &mut Vec<Account>,
    shop: &str,
    to: &ShopAccount,
) -> Result<(), NameError> {
    check_shop_name(shop, accounts)?;
    let shop = shop.trim().to_string();
    match to {
        ShopAccount::Listed(name) => accounts
            .iter_mut()
            .find(|account| &account.name == name)
            .ok_or(NameError::NoAccount)?
            .shops
            .push(shop),
        ShopAccount::New(name) => {
            check_account_name(name, accounts, None)?;
            accounts.push(Account {
                name: name.clone(),
                shops: vec![shop],
            });
        }
    }
    Ok(())
}

/// The account whose listed shop a label shows: the longest name found, then the one listed first.
pub fn account_of(sender: &SenderText, accounts: &[Account]) -> Option<OrderAccount> {
    let mut found: Option<(usize, &Account, &String)> = None;
    for account in accounts {
        for shop in &account.shops {
            let length = squash(shop).chars().count();
            if sender.shows(shop) && found.is_none_or(|(longest, _, _)| length > longest) {
                found = Some((length, account, shop));
            }
        }
    }
    found.map(|(_, account, shop)| OrderAccount {
        account: account.name.clone(),
        shop: Some(shop.clone()),
    })
}

/// Two shop names are the same with spaces and upper and lower case taken out.
pub fn same_shop(a: &str, b: &str) -> bool {
    squash(a) == squash(b)
}

fn squash(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests;
