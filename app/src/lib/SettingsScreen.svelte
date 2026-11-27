<script lang="ts">
  // Settings: the theme, the data folder, its storage and *About*, as full-width groups of
  // rows (the setting at the left, its control at the right). Every other choice has one fixed
  // default, shown where it acts. Read-only, Storage shows the sizes only.
  import { onMount } from "svelte";
  import { texts } from "../texts";
  import { formatSize } from "../size";
  import type { Archiving } from "./archiving.svelte";
  import { Storage } from "./storage.svelte";
  import { access } from "./readOnly.svelte";
  import type { DateRange } from "./searchRange";
  import { unknownKeys, type About } from "./api/start";
  import { themes, type Theme } from "./theme";
  import Icon from "./Icon.svelte";
  import Logo from "./Logo.svelte";

  let {
    about,
    dataFolder,
    theme,
    onTheme,
    onChangeFolder,
    onOpenFolder,
    archiving,
    range,
  }: {
    about: About | null;
    dataFolder: string | null;
    theme: Theme;
    onTheme: (theme: Theme) => void;
    onChangeFolder: () => void;
    onOpenFolder: () => void;
    archiving: Archiving;
    // The range Storage's **Archive** starts with.
    range: DateRange;
  } = $props();

  const words = texts.settings;
  const store = texts.archive;
  // Made as the screen opens, from the range it opens with.
  let storage = $state<Storage | null>(null);
  onMount(() => {
    storage = new Storage(archiving, { ...range });
    storage.load();
  });

  // Read each time Settings opens, so a rules file changed meanwhile is read again.
  let unknown: string[] = $state([]);
  $effect(() => {
    unknownKeys()
      .then((found) => (unknown = found))
      .catch(() => (unknown = []));
  });
</script>

<div class="settings">
  <section>
    <h3>{words.appearance}</h3>
    <div class="group">
      <div class="row">
        <div class="what">
          <span class="name">{words.theme}</span>
          <span class="hint">{words.themeHint}</span>
        </div>
        <div class="segmented" role="radiogroup" aria-label={words.theme}>
          {#each themes as choice (choice)}
            <button
              role="radio"
              aria-checked={choice === theme}
              class:chosen={choice === theme}
              onclick={() => onTheme(choice)}
              ><Icon name={choice} size={16} />{words.themes[choice]}</button
            >
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section>
    <h3>{words.data}</h3>
    <div class="group">
      <div class="row">
        <div class="what">
          <span class="name">{words.dataFolder}</span>
          <span class="path">{dataFolder ?? ""}</span>
        </div>
        <div class="controls">
          <button onclick={onOpenFolder}><Icon name="folder" size={16} />{words.open}</button>
          <button onclick={onChangeFolder}>{words.change}</button>
        </div>
      </div>
      {#if unknown.length > 0}
        <div class="row">
          <div class="what">
            <span class="name">{words.unknownKeys}</span>
            <span class="hint">{words.unknownKeysHint}</span>
            {#each unknown as key (key)}
              <span class="path">{key}</span>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </section>

  <section>
    <h3>{store.storage}</h3>
    {#if storage}
      {@const rows = storage}
      <div class="group">
        <div class="row">
          <div class="what">
            <span class="name">{store.used}</span>
            <span class="hint">{store.usedHint}</span>
          </div>
          {#if rows.used}
            <span class="value"
              >{store.usedValue(formatSize(rows.used.bytes), rows.used.dates, rows.used.archived)}</span
            >
          {/if}
        </div>
        {#if !access.readOnly}
          <div class="row">
            <div class="what">
              <span class="name">{store.archiveDates}</span>
              <span class="hint">{store.archiveDatesHint}</span>
            </div>
            <div class="controls">
              <label class="field"
                >{store.from}
                <input
                  type="date"
                  value={rows.range.from}
                  onchange={(event) => rows.setRange({ ...rows.range, from: event.currentTarget.value })}
                /></label
              >
              <label class="field"
                >{store.to}
                <input
                  type="date"
                  value={rows.range.to}
                  onchange={(event) => rows.setRange({ ...rows.range, to: event.currentTarget.value })}
                /></label
              >
              <span class="value range-value"
                >{rows.preview
                  ? store.rangeValue(rows.preview.dates.length, formatSize(rows.preview.bytes))
                  : ""}</span
              >
              <button
                disabled={!rows.preview || rows.preview.dates.length === 0}
                onclick={rows.archiveRange}><Icon name="archive" size={16} />{store.archive}</button
              >
            </div>
          </div>
          <div class="row">
            <div class="what">
              <span class="name">{store.auto}</span>
              <span class="hint">{store.autoHint}</span>
            </div>
            <div class="controls">
              <label class="field"
                >{store.olderThan}
                <input
                  class="days"
                  type="number"
                  min="1"
                  value={rows.days}
                  onblur={(event) => rows.changeDays(event.currentTarget.valueAsNumber)}
                />
                {store.days}</label
              >
              <div class="segmented" role="radiogroup" aria-label={store.auto}>
                {#each [false, true] as on (on)}
                  <button
                    role="radio"
                    aria-checked={rows.auto === on}
                    class:chosen={rows.auto === on}
                    onclick={() => rows.turn(on)}>{on ? store.on : store.off}</button
                  >
                {/each}
              </div>
            </div>
          </div>
        {/if}
        {#if rows.error}
          <div class="row">
            <span class="problem">{rows.error}</span>
          </div>
        {/if}
      </div>
    {/if}
  </section>

  <section>
    <h3>{words.about}</h3>
    <div class="group">
      <div class="row">
        <Logo size={40} />
        <div class="what">
          <span class="name">{texts.title} {about?.version ?? ""}</span>
          <span class="hint">{words.tagline}</span>
        </div>
      </div>
      {#if about}
        {#each [[words.version, about.version], [words.built, about.built], [words.pdfium, about.pdfium], [words.schema, about.schema]] as [label, value] (label)}
          <div class="row fact">
            <span class="what">{label}</span>
            <span class="value">{value}</span>
          </div>
        {/each}
      {/if}
    </div>
  </section>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 1.6rem;
    padding-bottom: 0.5rem;
  }

  h3 {
    margin: 0 0 0.5rem;
    color: var(--text-muted);
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  /* A group of rows: one box, a line between rows.  */
  .group {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 1rem;
    min-height: 3.4rem;
    padding: 0.6rem 1rem;
  }

  .row + .row {
    border-top: 1px solid var(--border-soft);
  }

  .row.fact {
    min-height: 0;
    padding-top: 0.45rem;
    padding-bottom: 0.45rem;
  }

  .what {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .name {
    font-weight: 600;
  }

  .hint {
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .path {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 0.88rem;
    overflow-wrap: anywhere;
  }

  .value {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .controls {
    display: flex;
    gap: 0.5rem;
    flex: none;
  }

  .controls {
    align-items: center;
  }

  .controls button {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .field {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    color: var(--text-muted);
    white-space: nowrap;
  }

  input.days {
    width: 4.5rem;
    font-variant-numeric: tabular-nums;
  }

  /* The range's count, wide enough that the button beside it does not move as it changes. */
  .range-value {
    min-width: 9rem;
    text-align: right;
  }

  .problem {
    color: var(--danger);
  }

  /* Three choices side by side; the chosen one is raised. */
  .segmented {
    display: inline-flex;
    flex: none;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
  }

  .segmented button {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.85rem;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-muted);
  }

  .segmented button:hover:not(.chosen) {
    border-color: transparent;
    background: var(--surface-hover);
    color: var(--text);
  }

  .segmented button.chosen,
  .segmented button.chosen:hover {
    border-color: var(--border);
    background: var(--surface);
    color: var(--text);
    font-weight: 600;
    box-shadow: var(--shadow);
  }
</style>
