//! The shop prompt without a prompt: orders whose shop no account lists stop the command, naming
//! the shops, unless `--account` puts them under one account for this run. Nothing is written to
//! `config.toml`, so those orders are under *No account* afterwards.

use packing_batch::Draft;
use packing_engine::account_questions::ShopQuestion;
use packing_engine::accounts::{ShopAccount, add_shop};

use crate::error::CliError;

pub fn answer_unlisted(
    draft: &mut Draft,
    account: Option<&str>,
    quiet_ids: bool,
) -> Result<(), CliError> {
    let questions = draft.account_questions();
    if questions.is_empty() {
        return Ok(());
    }
    let Some(account) = account else {
        return Err(CliError::Accounts(describe(&questions, quiet_ids)));
    };
    let mut accounts = draft.accounts.clone();
    for shop in &questions {
        let to = if accounts.iter().any(|listed| listed.name == account) {
            ShopAccount::Listed(account.to_string())
        } else {
            ShopAccount::New(account.to_string())
        };
        add_shop(&mut accounts, &shop.name, &to).map_err(|error| {
            CliError::Accounts(format!("cannot put a shop under \"{account}\": {error}"))
        })?;
    }
    draft.answer_accounts(accounts);
    Ok(())
}

fn describe(questions: &[ShopQuestion], quiet_ids: bool) -> String {
    let mut text = String::from("some orders have no account: list their shops in config.toml");
    text.push_str(" or pass --account <name> to put them all under one account for this run.");
    if quiet_ids {
        text.push_str(&format!("\n  {} shops no account lists", questions.len()));
        return text;
    }
    for shop in questions {
        text.push_str(&format!(
            "\n  shop \"{}\": {} orders",
            shop.name, shop.orders
        ));
    }
    text
}
