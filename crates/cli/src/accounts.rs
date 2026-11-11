//! The shop prompt without a prompt: orders whose shop no account lists stop the command, naming
//! the shops, unless `--account` puts them under one account for this run. Nothing is written to
//! `packing-settings.toml`.

use packing_batch::Draft;
use packing_engine::account_questions::AccountQuestions;
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
        return Err(CliError::Accounts(describe(&questions, draft, quiet_ids)));
    };
    let mut accounts = draft.accounts.clone();
    for shop in &questions.shops {
        let to = if accounts.iter().any(|listed| listed.name == account) {
            ShopAccount::Listed(account.to_string())
        } else {
            ShopAccount::New(account.to_string())
        };
        add_shop(&mut accounts, &shop.name, &to).map_err(|error| {
            CliError::Accounts(format!("cannot put a shop under \"{account}\": {error}"))
        })?;
    }
    let no_sender = questions
        .no_sender
        .iter()
        .map(|file| (file.input, account.to_string()))
        .collect();
    draft.answer_accounts(accounts, no_sender);
    Ok(())
}

fn describe(questions: &AccountQuestions, draft: &Draft, quiet_ids: bool) -> String {
    let mut text =
        String::from("some orders have no account: list their shops in packing-settings.toml");
    text.push_str(" or pass --account <name> to put them all under one account for this run.");
    if quiet_ids {
        text.push_str(&format!(
            "\n  {} shops no account lists, {} files with labels that show no sender",
            questions.shops.len(),
            questions.no_sender.len()
        ));
        return text;
    }
    for shop in &questions.shops {
        text.push_str(&format!(
            "\n  shop \"{}\": {} orders",
            shop.name, shop.orders
        ));
    }
    for file in &questions.no_sender {
        text.push_str(&format!(
            "\n  {} shows no sender on {} of {} orders",
            draft.files[file.input].filename, file.orders, file.of_orders
        ));
    }
    text
}
