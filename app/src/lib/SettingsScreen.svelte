<script lang="ts">
  // Settings: the theme, the data folder with the way to the Archive page, and *About*, as
  // full-width groups of rows (the setting at the left, its control at the right). Every other
  // choice has one fixed default, shown where it acts.
  import { texts } from "../texts";
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
    onOpenArchive,
  }: {
    about: About | null;
    dataFolder: string | null;
    theme: Theme;
    onTheme: (theme: Theme) => void;
    onChangeFolder: () => void;
    onOpenFolder: () => void;
    onOpenArchive: () => void;
  } = $props();

  const words = texts.settings;

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
      <div class="row">
        <div class="what">
          <span class="name">{texts.archive.settingsRow}</span>
          <span class="hint">{texts.archive.settingsHint}</span>
        </div>
        <div class="controls">
          <button onclick={onOpenArchive}><Icon name="archive" size={16} />{texts.archive.open}</button>
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
    align-items: center;
    gap: 0.5rem;
    flex: none;
  }

  .controls button {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

</style>
