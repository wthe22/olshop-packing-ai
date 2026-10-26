<script lang="ts">
  // Settings: the data folder and *About*. The theme is the title bar's menu; every other choice
  // has one fixed default, shown where it acts.
  import { texts } from "../texts";
  import type { About } from "./api/start";
  import Icon from "./Icon.svelte";

  let {
    about,
    dataFolder,
    onChangeFolder,
    onOpenFolder,
  }: {
    about: About | null;
    dataFolder: string | null;
    onChangeFolder: () => void;
    onOpenFolder: () => void;
  } = $props();
</script>

<div class="settings">
  <section class="card">
    <h3>{texts.settings.dataFolder}</h3>
    <p class="hint">{texts.settings.dataFolderHint}</p>
    <div class="folder">
      <Icon name="folder" />
      <span class="path">{dataFolder ?? ""}</span>
      <button onclick={onOpenFolder}>{texts.settings.open}</button>
      <button onclick={onChangeFolder}>{texts.settings.change}</button>
    </div>
  </section>

  <section class="card about">
    <h3>{texts.settings.about}</h3>
    <div class="brand">
      <span class="mark"><Icon name="box" size={28} /></span>
      <div>
        <div class="name">
          {texts.title}
          {#if about}<span class="version">{about.version}</span>{/if}
        </div>
        <p class="tagline">{texts.settings.tagline}</p>
      </div>
    </div>
    {#if about}
      <dl>
        <dt>{texts.settings.version}</dt>
        <dd>{about.version}</dd>
        <dt>{texts.settings.pdfium}</dt>
        <dd>{about.pdfium}</dd>
        <dt>{texts.settings.schema}</dt>
        <dd>{about.schema}</dd>
      </dl>
    {/if}
  </section>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    max-width: 46rem;
  }

  .card {
    padding: 1rem 1.25rem 1.25rem;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface);
  }

  h3 {
    margin: 0 0 0.25rem;
    font-size: 1rem;
  }

  .hint {
    margin: 0 0 0.75rem;
    color: var(--text-muted);
  }

  .folder {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5rem 0.5rem 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
    color: var(--text-muted);
  }

  .path {
    flex: 1;
    color: var(--text);
    font-family: Consolas, monospace;
    font-size: 0.9rem;
    overflow-wrap: anywhere;
  }

  .about .brand {
    display: flex;
    align-items: center;
    gap: 0.9rem;
    margin: 0.75rem 0 1rem;
  }

  .mark {
    display: inline-flex;
    padding: 0.55rem;
    border-radius: 12px;
    background: var(--accent-soft);
    color: var(--accent);
  }

  .name {
    font-size: 1.2rem;
    font-weight: 650;
  }

  .version {
    margin-left: 0.3rem;
    padding: 0.05rem 0.5rem;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--text-muted);
    font-size: 0.8rem;
    font-weight: 500;
    vertical-align: middle;
  }

  .tagline {
    margin: 0.15rem 0 0;
    color: var(--text-muted);
  }

  dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.4rem 1.5rem;
    margin: 0;
    padding-top: 0.9rem;
    border-top: 1px solid var(--border-soft);
  }

  dt {
    color: var(--text-muted);
  }

  dd {
    margin: 0;
    font-variant-numeric: tabular-nums;
  }
</style>
