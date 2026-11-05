<script lang="ts">
  // The shop prompt: after the logo prompt, before the Plan screen, the files whose labels none
  // of the workspace's shops sent, each with how many of its orders and who sent them. **Back**
  // (also Esc, the caller's) returns to the file list; **Read anyway** goes on to the plan.
  import type { OtherShopView, SenderView } from "./api/batch";
  import { texts } from "../texts";
  import Dialog from "./Dialog.svelte";
  import Icon from "./Icon.svelte";

  let {
    shops,
    files,
    onBack,
    onReadAnyway,
  }: {
    shops: string[];
    files: OtherShopView[];
    onBack: () => void;
    onReadAnyway: () => void;
  } = $props();

  const words = texts.shops;

  function sender(view: SenderView): string {
    switch (view.kind) {
      case "of_workspace":
        return words.ofWorkspace(view.workspace);
      case "unlisted":
        return words.unlisted;
      case "not_found":
        return words.notFound;
    }
  }
</script>

<Dialog title={words.title} width="46rem">
  <p class="listed">{words.listed(shops)}</p>
  <ul>
    {#each files as file (file.file)}
      <li>
        <span class="file" title={file.file}><Icon name="file" size={14} />{file.file}</span>
        <span class="orders">{words.orders(file.orders, file.of_orders)}</span>
        <span class="sender">{sender(file.sender)}</span>
      </li>
    {/each}
  </ul>
  <div class="answers">
    <button onclick={onReadAnyway}>{words.readAnyway}</button>
    <button class="primary" onclick={onBack} {@attach (button) => button.focus()}>
      {words.back}
    </button>
  </div>
</Dialog>

<style>
  .listed {
    margin: 0 0 0.75rem;
    color: var(--text-muted);
  }

  ul {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    gap: 0.4rem 1rem;
    margin: 0 0 1rem;
    padding: 0;
    list-style: none;
  }

  li {
    display: contents;
  }

  .file {
    justify-self: start;
    max-width: 100%;
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    min-width: 0;
    padding: 0.05rem 0.45rem;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-2);
    font-family: var(--font-mono);
    font-size: 0.8rem;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .file :global(svg) {
    flex: none;
  }

  .orders {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .sender {
    color: var(--text-muted);
  }

  .answers {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>
