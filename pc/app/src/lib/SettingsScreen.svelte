<script lang="ts">
  // Settings (08 › *6. Settings*): the data folder and the packing-list default.
  import { texts } from "../texts";
  import type { About, Layout, Scope, Settings } from "./api";

  let {
    settings,
    about,
    dataFolder,
    onChangeFolder,
    onOpenFolder,
    onSave,
  }: {
    settings: Settings | null;
    about: About | null;
    dataFolder: string | null;
    onChangeFolder: () => void;
    onOpenFolder: () => void;
    onSave: (settings: Settings) => void;
  } = $props();

  let scope = $state<Scope>("whole");
  let layouts = $state<Layout[]>(["pick"]);

  // Fill the form when the settings arrive or change.
  $effect(() => {
    if (settings) {
      scope = settings.scope;
      layouts = [...settings.layouts];
    }
  });

  const scopes: { value: Scope; label: string }[] = [
    { value: "whole", label: texts.settings.scopeWhole },
    { value: "per-pdf", label: texts.settings.scopePerPdf },
    { value: "none", label: texts.settings.scopeNone },
  ];

  const layoutOptions: { value: Layout; label: string }[] = [
    { value: "full", label: texts.settings.layoutFull },
    { value: "summary", label: texts.settings.layoutSummary },
    { value: "pick", label: texts.settings.layoutPick },
  ];

  function toggleLayout(layout: Layout, checked: boolean) {
    layouts = checked
      ? [...layouts, layout]
      : layouts.filter((item) => item !== layout);
  }
</script>

<main>
  <h2>{texts.settings.title}</h2>

  <section>
    <h3>{texts.settings.dataFolder}</h3>
    <div class="row">
      <span class="path">{dataFolder ?? ""}</span>
      <button onclick={onChangeFolder}>{texts.settings.change}</button>
      <button onclick={onOpenFolder}>{texts.settings.open}</button>
    </div>
  </section>

  <section>
    <h3>{texts.settings.packingList}</h3>
    <div class="group">
      <span class="label">{texts.settings.scope}</span>
      {#each scopes as option (option.value)}
        <label>
          <input
            type="radio"
            name="scope"
            value={option.value}
            checked={scope === option.value}
            onchange={() => (scope = option.value)}
          />
          {option.label}
        </label>
      {/each}
    </div>
    <div class="group">
      <span class="label">{texts.settings.layouts}</span>
      {#each layoutOptions as option (option.value)}
        <label>
          <input
            type="checkbox"
            checked={layouts.includes(option.value)}
            onchange={(event) =>
              toggleLayout(option.value, event.currentTarget.checked)}
          />
          {option.label}
        </label>
      {/each}
    </div>
  </section>

  <section>
    <h3>{texts.settings.about}</h3>
    {#if about}
      <div class="group">
        <span>{texts.settings.version(about.version)}</span>
        <span>{texts.settings.pdfium(about.pdfium)}</span>
      </div>
    {/if}
  </section>

  <div class="actions">
    <button class="primary" onclick={() => onSave({ scope, layouts })}
      >{texts.settings.save}</button
    >
  </div>
</main>

<style>
  main {
    padding: 1rem 1.5rem;
    max-width: 48rem;
  }

  h2 {
    margin: 0 0 1rem;
  }

  section {
    margin-bottom: 1.5rem;
  }

  h3 {
    margin: 0 0 0.5rem;
    font-size: 1rem;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .path {
    flex: 1;
    font-family: Consolas, monospace;
    overflow-wrap: anywhere;
  }

  .group {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.4rem;
  }

  .group label {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .label {
    width: 4rem;
    color: #4a4a4a;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
  }

  @media (prefers-color-scheme: dark) {
    .label {
      color: #b0b0b0;
    }
  }
</style>
