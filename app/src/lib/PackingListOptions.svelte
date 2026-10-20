<script lang="ts">
  // The packing-list options (08 › *3. Plan*): what a list covers (*Scope*) and which layouts are
  // written, both sent with *Save batch* (07 › *Packing list*).
  import type { Layout, Scope } from "./api/start";
  import { texts } from "../texts";

  let {
    scope,
    layouts,
    onScope,
    onLayout,
  }: {
    scope: Scope;
    layouts: Layout[];
    onScope: (scope: Scope) => void;
    onLayout: (layout: Layout, checked: boolean) => void;
  } = $props();

  const scopes: { value: Scope; label: string }[] = [
    { value: "whole", label: texts.plan.scopeWhole },
    { value: "per-pdf", label: texts.plan.scopePerPdf },
    { value: "none", label: texts.plan.scopeNone },
  ];
  const layoutOptions: { value: Layout; label: string }[] = [
    { value: "full", label: texts.plan.layoutFull },
    { value: "summary", label: texts.plan.layoutSummary },
    { value: "pick", label: texts.plan.layoutCategory },
  ];
</script>

<div class="packing">
  <span class="label">{texts.plan.packingList}</span>
  {#each scopes as option (option.value)}
    <label>
      <input
        type="radio"
        name="scope"
        value={option.value}
        checked={scope === option.value}
        onchange={() => onScope(option.value)}
      />
      {option.label}
    </label>
  {/each}
  {#each layoutOptions as option (option.value)}
    <label>
      <input
        type="checkbox"
        checked={layouts.includes(option.value)}
        onchange={(e) => onLayout(option.value, e.currentTarget.checked)}
      />
      {option.label}
    </label>
  {/each}
</div>

<style>
  .packing {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    flex-wrap: wrap;
  }

  .packing label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .packing .label {
    font-weight: 600;
  }
</style>
