//! The Sort rules screen (sort-rules.md): the sort rules of `packing-workflows.toml`, one rule's
//! groups, the live check with the counts each group takes in the open draft, and the writes — a
//! rule's groups, a new rule, a rename, a delete, the whole file as text. After a write the open
//! draft's plan follows the file, keeping its per-batch changes where they still apply.

use packing_engine::ids::BatchId;
use packing_engine::rules::parse_filter;
use packing_engine::rules_edit::{self, Tree};
use packing_engine::sorting::{plan_groups, sort_batch};
use packing_engine::workflows::{
    GroupEdit, NewRule, SortRule, Workflows, add_rule, delete_rule, load_workflows, rename_rule,
    write_groups,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{AppState, OpenDraft, PlanEntry, settings};

#[derive(Serialize)]
pub struct RulesView {
    rules: Vec<String>,
    /// The whole file, for *Edit file as text*.
    text: String,
    /// The couriers by code and name, for the Courier box.
    couriers: Vec<CourierView>,
    /// The open draft's rule, so the screen opens on it; none without a draft.
    draft_rule: Option<String>,
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
            .map(|r| r.name.clone())
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
    let written = settings::workflows(state.program())
        .and_then(|(text, _)| write_groups(&text, rule, &edits(groups)).map_err(|e| e.to_string()));
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

/// Write the file and let the open draft's plan follow it. `renamed` is a rule's old and new name.
fn write(
    state: &AppState,
    written: &str,
    workflows: &Workflows,
    renamed: Option<(&str, &str)>,
) -> Result<(), String> {
    let (_, before) = settings::workflows(state.program())?;
    settings::write_workflows(state.program(), written)?;
    let _ = state.with_draft(|open| {
        let was = before.sort_rule(&open.draft.rule_name).cloned();
        if let Some((old, new)) = renamed
            && open.draft.rule_name == old
        {
            open.draft.rule_name = new.to_string();
        }
        follow(open, was.as_ref(), workflows);
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
    let (text, _) = settings::workflows(state.program())?;
    let (written, workflows) =
        write_groups(&text, rule, &edits(groups)).map_err(|e| e.to_string())?;
    write(state, &written, &workflows, None)
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
    let (text, _) = settings::workflows(state.program())?;
    let from = copy_from.map_or(NewRule::Empty, NewRule::CopyOf);
    let (written, workflows) = add_rule(&text, name, from).map_err(|e| e.to_string())?;
    write(state, &written, &workflows, None)
}

/// **Rename…**: saved batches keep the name they recorded.
#[tauri::command]
pub fn rule_rename(state: State<'_, AppState>, rule: String, name: String) -> Result<(), String> {
    renamed(&state, &rule, &name)
}

pub(crate) fn renamed(state: &AppState, rule: &str, name: &str) -> Result<(), String> {
    let (text, _) = settings::workflows(state.program())?;
    let (written, workflows) = rename_rule(&text, rule, name).map_err(|e| e.to_string())?;
    write(state, &written, &workflows, Some((rule, name)))
}

/// **Delete rule**; the last rule cannot be deleted.
#[tauri::command]
pub fn rule_delete(state: State<'_, AppState>, rule: String) -> Result<(), String> {
    let (text, _) = settings::workflows(state.program())?;
    let (written, workflows) = delete_rule(&text, &rule).map_err(|e| e.to_string())?;
    write(&state, &written, &workflows, None)
}

/// *Edit file as text*: the whole file, checked as at start.
#[tauri::command]
pub fn rules_check_text(text: String) -> Result<(), String> {
    load_workflows(&text)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rules_save_text(state: State<'_, AppState>, text: String) -> Result<(), String> {
    let workflows = load_workflows(&text).map_err(|error| error.to_string())?;
    write(&state, &text, &workflows, None)
}

/// The boxes of a filter, from the engine's parser, so the window has no second parser.
#[tauri::command]
pub fn filter_to_tree(text: String) -> Result<Tree, String> {
    rules_edit::filter_to_tree(&text).map_err(|error| error.to_string())
}

/// Boxes as filter text, in the engine printer's one-line form.
#[tauri::command]
pub fn tree_to_filter(tree: Tree) -> Result<String, String> {
    rules_edit::tree_to_filter(&tree).map_err(|error| error.to_string())
}
