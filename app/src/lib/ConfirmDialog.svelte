<script lang="ts">
  // The window's yes-or-no question (`confirm.svelte.ts`): its title, one short line, the button
  // that does it and *Cancel*. The button has the focus, so Enter answers yes; Escape answers no
  // and closes nothing behind the dialog.
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
    confirmBox.answer(false);
  }
</script>

<svelte:window onkeydowncapture={onKey} />

{#if confirmBox.question}
  <Dialog title={confirmBox.question.title}>
    {#if confirmBox.question.body}
      <p>{confirmBox.question.body}</p>
    {/if}
    <div class="buttons">
      <button class="primary" bind:this={okButton} onclick={() => confirmBox.answer(true)}
        >{confirmBox.question.ok}</button
      >
      <button onclick={() => confirmBox.answer(false)}>{texts.confirmCancel}</button>
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
