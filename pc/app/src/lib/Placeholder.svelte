<script lang="ts">
  // A screen that is not built yet (2.9 New batch, 2.12 Categories).
  import { texts } from "../texts";

  let { text }: { text: string } = $props();
</script>

<main>
  <p>{text}</p>
</main>

<style>
  main {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 60vh;
    color: #6a6a6a;
  }

  @media (prefers-color-scheme: dark) {
    main {
      color: #b0b0b0;
    }
  }
</style>
