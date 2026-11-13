//! The Sort rules screen (sort-rules.md): the sort rules of `config.toml`, one rule's
//! groups, the live check with the counts each group takes in the open draft, and the writes — a
//! rule's groups, a new rule, a rename, a rule switched on or off, a delete, the whole file as
//! text. After a write the open draft's plan follows the file, keeping its per-batch changes where
//! they still apply.

use packing_engine::ids::BatchId;
use packing_engine::rules::{self, Tree, parse_filter};
use packing_engine::sorting::{plan_groups, sort_batch};
use packing_engine::workflows::{
    GroupEdit, NewRule, SortRule, Workflows, add_rule, delete_rule, load_workflows, rename_rule,
    set_rule_enabled, write_groups,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{AppState, OpenDraft, PlanEntry, settings};

#[derive(Serialize)]
pub struct RulesView {
    rules: Vec<RuleView>,
    /// The whole file, for *Edit file as text*.
    text: String,
    /// The couriers by code and name, for the Courier box.
    couriers: Vec<CourierView>,
    /// The open draft's rule, so the screen opens on it; none without a draft.
    draft_rule: Option<String>,
}

/// A sort rule of the file and whether New batch offers it.
#[derive(Serialize)]
pub struct RuleView {
    name: String,
    enabled: bool,
}

#[derive(Serialize)]
pub struct CourierView {
    code: String,
    name: String,
}

/// One label group as the file holds it; `origin` is its place in the rule.
#[derive(Serialize)]
pub struct GroupView {
    code: String,
    name: String,
    filter: Option<String>,
    origin: usize,
}

/// One label group as the screen sends it back.
#[derive(Deserialize)]
pub struct GroupInput {
    code: String,
    name: String,
    filter: Option<String>,
    /// Its place in the rule as loaded; none for a group added on the screen.
    origin: Option<usize>,
}

/// The live check: each group's filter error, what the whole file would fail, and the orders
/// each group takes in the open draft (none without a draft, or while something fails).
#[derive(Serialize)]
pub struct CheckView {
    errors: Vec<Option<String>>,
    general: Option<String>,
    counts: Option<Vec<usize>>,
}

fn edits(groups: &[GroupInput]) -> Vec<GroupEdit> {
    groups
        .iter()
        .map(|group| GroupEdit {
            code: group.code.clone(),
            name: group.name.clone(),
            filter: group.filter.clone(),
            origin: group.origin,
        })
        .collect()
}

pub(crate) fn rules_view(state: &AppState) -> Result<RulesView, String> {
    let (text, workflows) = settings::workflows(state.program())?;
    let draft_rule = state
        .with_draft(|open| Ok(open.draft.rule_name.clone()))
        .ok();
    Ok(RulesView {
        rules: workflows
            .sort_rules
            .iter()
            .map(|rule| RuleView {
                name: rule.name.clone(),
                enabled: rule.enabled,
            })
            .collect(),
        text,
        couriers: workflows
            .couriers
            .iter()
            .map(|courier| CourierView {
                code: courier.code.as_str().to_string(),
                name: courier.name.clone(),
            })
            .collect(),
        draft_rule,
    })
}

pub(crate) fn rule_groups(state: &AppState, rule: &str) -> Result<Vec<GroupView>, String> {
    let (_, workflows) = settings::workflows(state.program())?;
    let rule = workflows
        .sort_rule(rule)
        .ok_or_else(|| format!("There is no sort rule \"{rule}\"."))?;
    Ok(rule
        .groups
        .iter()
        .enumerate()
        .map(|(origin, group)| GroupView {
            code: group.code.clone(),
            name: group.name.clone(),
            filter: group.filter.as_ref().map(|filter| filter.text.clone()),
            origin,
        })
        .collect())
}

pub(crate) fn check(state: &AppState, rule: &str, groups: &[GroupInput]) -> CheckView {
    let errors = groups
        .iter()
        .map(|group| {
            let filter = group.filter.as_deref()?;
            parse_filter(filter).err().map(|error| error.to_string())
        })
        .collect();
    let written = settings::parser_text(state.program()).and_then(|parser| {
        let (text, _) = settings::workflows(state.program())?;
        write_groups(&parser, &text, rule, &edits(groups)).map_err(|e| e.to_string())
    });
    let (general, counts) = match written {
        Ok((_, workflows)) => (
            None,
            workflows.sort_rule(rule).and_then(|r| counts(state, r)),
        ),
        Err(error) => (Some(error), None),
    };
    CheckView {
        errors,
        general,
        counts,
    }
}

/// The orders each group of `rule` takes in the open draft, in the rule's order.
fn counts(state: &AppState, rule: &SortRule) -> Option<Vec<usize>> {
    state
        .with_draft(|open| {
            let draft = &open.draft;
            // Only the counts are read, so the batch number the files would carry does not matter.
            let sorted = sort_batch(
                &draft.copies,
                &draft.orders.kept,
                &plan_groups(rule),
                BatchId::new(1),
                &draft.couriers,
            );
            Ok(rule
                .groups
                .iter()
                .map(|group| {
                    sorted
                        .groups
                        .iter()
                        .find(|taken| taken.code == group.code)
                        .map_or(0, |taken| taken.order_count())
                })
                .collect())
        })
        .ok()
}

/// What an edit makes of the two files' texts: the new `config.toml` and the rules it holds.
type Edited = Result<(String, Workflows), String>;

/// Make `edit` of `config.toml` as it is now, under the data folder's lock so another copy's change
/// to the accounts or the rules is kept. Returns the rules as they were and as they are.
fn edit_config(
    state: &AppState,
    edit: impl FnOnce(&str, &str) -> Edited,
) -> Result<(Workflows, Workflows), String> {
    state.write_data(|_, _| settings::change_config(state.program(), edit))
}

/// Edit the file and let the open draft's plan follow it. `renamed` is a rule's old and new name.
fn write(
    state: &AppState,
    renamed: Option<(&str, &str)>,
    edit: impl FnOnce(&str, &str) -> Edited,
) -> Result<(), String> {
    let (before, after) = edit_config(state, edit)?;
    let _ = state.with_draft(|open| {
        let was = before.sort_rule(&open.draft.rule_name).cloned();
        if let Some((old, new)) = renamed
            && open.draft.rule_name == old
        {
            open.draft.rule_name = new.to_string();
        }
        follow(open, was.as_ref(), &after);
        Ok(())
    });
    Ok(())
}

/// The draft's plan from its rule as the file now holds it: a group keeps its skip and its file
/// name, a one-off group of this batch stays above the rest. A rule that is gone leaves the plan.
fn follow(open: &mut OpenDraft, was: Option<&SortRule>, after: &Workflows) {
    let Some(rule) = after.sort_rule(&open.draft.rule_name) else {
        return;
    };
    let ruled: Vec<&str> = was
        .iter()
        .flat_map(|rule| rule.groups.iter().map(|group| group.code.as_str()))
        .collect();
    let mut entries: Vec<PlanEntry> = plan_groups(rule)
        .into_iter()
        .map(|mut group| {
            let old = open
                .entries
                .iter()
                .find(|entry| entry.group.code == group.code);
            group.file_name = old.and_then(|entry| entry.group.file_name.clone());
            PlanEntry {
                skipped: old.is_some_and(|entry| entry.skipped),
                group,
            }
        })
        .collect();
    let rest = entries.len().saturating_sub(1);
    let one_off: Vec<PlanEntry> = open
        .entries
        .iter()
        .filter(|entry| !ruled.contains(&entry.group.code.as_str()))
        .filter(|entry| {
            !entries
                .iter()
                .any(|kept| kept.group.code == entry.group.code)
        })
        .cloned()
        .collect();
    entries.splice(rest..rest, one_off);
    open.entries = entries;
    open.apply_plan();
}

#[tauri::command]
pub fn rules_list(state: State<'_, AppState>) -> Result<RulesView, String> {
    rules_view(&state)
}

#[tauri::command]
pub fn rules_load(state: State<'_, AppState>, rule: String) -> Result<Vec<GroupView>, String> {
    rule_groups(&state, &rule)
}

#[tauri::command]
pub fn rules_check(state: State<'_, AppState>, rule: String, groups: Vec<GroupInput>) -> CheckView {
    check(&state, &rule, &groups)
}

#[tauri::command]
pub fn rules_save(
    state: State<'_, AppState>,
    rule: String,
    groups: Vec<GroupInput>,
) -> Result<(), String> {
    save_groups(&state, &rule, &groups)
}

pub(crate) fn save_groups(
    state: &AppState,
    rule: &str,
    groups: &[GroupInput],
) -> Result<(), String> {
    write(state, None, |parser, text| {
        write_groups(parser, text, rule, &edits(groups)).map_err(|e| e.to_string())
    })
}

/// **+ New rule**: one group `Z Lainnya`, or a copy of `copy_from`.
#[tauri::command]
pub fn rule_new(
    state: State<'_, AppState>,
    name: String,
    copy_from: Option<String>,
) -> Result<(), String> {
    new_rule(&state, &name, copy_from.as_deref())
}

pub(crate) fn new_rule(
    state: &AppState,
    name: &str,
    copy_from: Option<&str>,
) -> Result<(), String> {
    let from = copy_from.map_or(NewRule::Empty, NewRule::CopyOf);
    write(state, None, |parser, text| {
        add_rule(parser, text, name, from).map_err(|e| e.to_string())
    })
}

/// **Rename**: saved batches keep the name they recorded.
#[tauri::command]
pub fn rule_rename(state: State<'_, AppState>, rule: String, name: String) -> Result<(), String> {
    renamed(&state, &rule, &name)
}

pub(crate) fn renamed(state: &AppState, rule: &str, name: &str) -> Result<(), String> {
    write(state, Some((rule, name)), |parser, text| {
        rename_rule(parser, text, rule, name).map_err(|e| e.to_string())
    })
}

/// The rule's switch: offered on New batch or not; the only rule that is on stays on. Nothing a
/// batch is sorted by changes, so the open draft's plan is left as the owner arranged it.
#[tauri::command]
pub fn rule_enable(state: State<'_, AppState>, rule: String, enabled: bool) -> Result<(), String> {
    switched(&state, &rule, enabled)
}

pub(crate) fn switched(state: &AppState, rule: &str, enabled: bool) -> Result<(), String> {
    edit_config(state, |parser, text| {
        set_rule_enabled(parser, text, rule, enabled).map_err(|e| e.to_string())
    })
    .map(|_| ())
}

/// **Delete rule**; the last rule, and the only one that is on, cannot be deleted.
#[tauri::command]
pub fn rule_delete(state: State<'_, AppState>, rule: String) -> Result<(), String> {
    write(&state, None, |parser, text| {
        delete_rule(parser, text, &rule).map_err(|e| e.to_string())
    })
}

/// *Edit file as text*: the whole file, checked as at start.
#[tauri::command]
pub fn rules_check_text(state: State<'_, AppState>, text: String) -> Result<(), String> {
    let parser = settings::parser_text(state.program())?;
    load_workflows(&parser, &text)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rules_save_text(state: State<'_, AppState>, text: String) -> Result<(), String> {
    write(&state, None, |parser, _| {
        let workflows = load_workflows(parser, &text).map_err(|error| error.to_string())?;
        Ok((text.clone(), workflows))
    })
}

/// The boxes of a filter, from the engine's parser, so the window has no second parser.
#[tauri::command]
pub fn filter_to_tree(text: String) -> Result<Tree, String> {
    rules::filter_to_tree(&text).map_err(|error| error.to_string())
}

/// Boxes as filter text, in the engine printer's one-line form.
#[tauri::command]
pub fn tree_to_filter(tree: Tree) -> Result<String, String> {
    rules::tree_to_filter(&tree).map_err(|error| error.to_string())
}
