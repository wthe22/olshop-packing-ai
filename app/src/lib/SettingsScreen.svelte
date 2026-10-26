<script lang="ts">
  // Settings: the data folder and *About*. Every other choice has one fixed default, shown where
  // it acts.
  import { texts } from "../texts";
  import type { About } from "./api/start";

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
  <section>
    <h3>{texts.settings.dataFolder}</h3>
    <div class="row">
      <span class="path">{dataFolder ?? ""}</span>
      <button onclick={onChangeFolder}>{texts.settings.change}</button>
      <button onclick={onOpenFolder}>{texts.settings.open}</button>
    </div>
  </section>

  <section>
    <h3>{texts.settings.about}</h3>
    {#if about}
      <div class="group">
        <span>{texts.settings.version(about.version)}</span>
        <span>{texts.settings.pdfium(about.pdfium)}</span>
        <span>{texts.settings.schema(about.schema)}</span>
      </div>
    {/if}
  </section>
</div>

<style>
  .settings {
    max-width: 48rem;
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
</style>
