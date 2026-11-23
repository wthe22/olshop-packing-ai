<script lang="ts">
  // The window's question (`confirm.svelte.ts`): its title, one short line, the button that does
  // it, the second way to do it when there is one, and *Cancel*. The first button has the focus, so
  // Enter answers it; Escape cancels and closes nothing behind the dialog.
  import { confirmBox } from "./confirm.svelte";
  import { texts } from "../texts";
  import Dialog from "./Dialog.svelte";

  let okButton = $state<HTMLButtonElement>();

  $effect(() => {
    if (confirmBox.question) okButton?.focus();
  });

  function onKey(event: KeyboardEvent) {
    if (!confirmBox.question || event.key !== "Escape") return;
    event.preventDefault();
    event.stopPropagation();
    confirmBox.answer("cancel");
  }
</script>

<svelte:window onkeydowncapture={onKey} />

{#if confirmBox.question}
  <Dialog title={confirmBox.question.title}>
    {#if confirmBox.question.body}
      <p>{confirmBox.question.body}</p>
    {/if}
    <div class="buttons">
      <button class="primary" bind:this={okButton} onclick={() => confirmBox.answer("ok")}
        >{confirmBox.question.ok}</button
      >
      {#if "other" in confirmBox.question}
        <button onclick={() => confirmBox.answer("other")}>{confirmBox.question.other}</button>
      {/if}
      <button onclick={() => confirmBox.answer("cancel")}>{texts.confirmCancel}</button>
    </div>
  </Dialog>
{/if}

<style>
  p {
    margin: 0;
    color: var(--text-muted);
  }

  .buttons {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }
</style>
