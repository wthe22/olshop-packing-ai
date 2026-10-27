<script lang="ts">
  // A file being written that the owner waits for (the day summary, an output written again): its
  // line and a moving bar, over a backdrop that takes the clicks until it is done.
  import Dialog from "./Dialog.svelte";

  let { text }: { text: string | null } = $props();
</script>

{#if text}
  <Dialog title={text}>
    <progress></progress>
  </Dialog>
{/if}

<style>
  progress {
    width: 100%;
    height: 1.1rem;
  }
</style>
