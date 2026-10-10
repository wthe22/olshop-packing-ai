<script lang="ts">
  // The boxes editor (08 › *5. Categories*): a group is *All of these* (`and`) or *Any of these*
  // (`or`) and holds rows (field · operator · value) and other groups; **not** sits on rows and
  // groups. `onEdit` is called after every change so the caller can recount and re-check.
  import { conditionFields, numberOperators, textOperators, texts } from "../texts";
  import {
    fieldType,
    newGroup,
    newRow,
    type UiGroup,
    type UiRow,
    type UiTree,
  } from "./conditions";

  let { tree, onEdit }: { tree: UiTree; onEdit: () => void } = $props();

  const operatorsFor = (field: string) =>
    fieldType(field) === "text" ? textOperators : numberOperators;
  const valueType = (field: string) => (fieldType(field) === "number" ? "number" : "text");

  function addRow(group: UiGroup) {
    group.children.push(newRow());
    onEdit();
  }

  function addGroup(group: UiGroup) {
    group.children.push(newGroup());
    onEdit();
  }

  function removeChild(group: UiGroup, index: number) {
    group.children.splice(index, 1);
    onEdit();
  }

  function setField(row: UiRow, field: string) {
    row.field = field;
    row.operator = fieldType(field) === "text" ? "contains" : "=";
    row.value = "";
    onEdit();
  }
</script>

{#snippet groupBox(group: UiGroup, root: boolean, parent: UiGroup | null, index: number)}
  <div class="group" class:root>
    <div class="group-head">
      <select
        value={group.groupKind}
        onchange={(event) => {
          group.groupKind = event.currentTarget.value === "any" ? "any" : "all";
          onEdit();
        }}
      >
        <option value="all">{texts.categories.allOf}</option>
        <option value="any">{texts.categories.anyOf}</option>
      </select>
      <label class="not">
        <input
          type="checkbox"
          checked={group.not}
          onchange={(event) => {
            group.not = event.currentTarget.checked;
            onEdit();
          }}
        />
        {texts.categories.not}
      </label>
      {#if !root && parent}
        <span class="grow"></span>
        <button class="remove" onclick={() => removeChild(parent, index)}
          >{texts.categories.removeGroup}</button
        >
      {/if}
    </div>

    {#each group.children as child, i (child)}
      {#if child.kind === "row"}
        {@render rowBox(child, group, i)}
      {:else}
        {@render groupBox(child, false, group, i)}
      {/if}
    {/each}

    <div class="group-foot">
      <button onclick={() => addRow(group)}>{texts.categories.addCondition}</button>
      <button onclick={() => addGroup(group)}>{texts.categories.addGroup}</button>
    </div>
  </div>
{/snippet}

{#snippet rowBox(row: UiRow, parent: UiGroup, index: number)}
  <div class="row">
    <select value={row.field} onchange={(event) => setField(row, event.currentTarget.value)}>
      {#each conditionFields as field (field.value)}
        <option value={field.value}>{field.label}</option>
      {/each}
    </select>
    <select
      value={row.operator}
      onchange={(event) => {
        row.operator = event.currentTarget.value;
        onEdit();
      }}
    >
      {#each operatorsFor(row.field) as op (op.value)}
        <option value={op.value}>{op.label}</option>
      {/each}
    </select>
    <input
      class="value"
      type={valueType(row.field)}
      value={row.value}
      placeholder={fieldType(row.field) === "time" ? "2026-10-07 14:00" : ""}
      oninput={(event) => {
        row.value = event.currentTarget.value;
        onEdit();
      }}
    />
    <label class="not">
      <input
        type="checkbox"
        checked={row.not}
        onchange={(event) => {
          row.not = event.currentTarget.checked;
          onEdit();
        }}
      />
      {texts.categories.not}
    </label>
    <button class="remove" title={texts.categories.delete} onclick={() => removeChild(parent, index)}
      >×</button
    >
  </div>
{/snippet}

<div class="boxes">
  {@render groupBox(tree.root, true, null, 0)}
</div>

<style>
  .boxes {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .group {
    border: 1px solid #d5d5d5;
    border-radius: 5px;
    padding: 0.35rem 0.5rem;
    background: #fafafa;
  }

  .group.root {
    background: #ffffff;
  }

  .group-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.3rem;
  }

  .group-foot {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.3rem;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin: 0.2rem 0;
  }

  .row .value {
    flex: 1;
    min-width: 8rem;
  }

  .not {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: #4a4a4a;
  }

  .grow {
    flex: 1;
  }

  button.remove {
    padding: 0.05rem 0.45rem;
  }

  select,
  input {
    font: inherit;
  }

  @media (prefers-color-scheme: dark) {
    .group {
      background: #2b2b2b;
      border-color: #444444;
    }

    .group.root {
      background: #242424;
    }

    .not {
      color: #b0b0b0;
    }
  }
</style>
