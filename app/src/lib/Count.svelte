<script lang="ts">
  // One count of a card's row in a column of its own: the number, right-aligned in a fixed width,
  // then its word, so the numbers of a card stand under each other ("34 orders", " 1 content").
  let { n, word }: { n: number; word: string } = $props();
</script>

<span class="count"><span class="n">{n}</span> <span class="word">{word}</span></span>

<style>
  .count {
    display: inline-block;
    width: 13ch;
    white-space: nowrap;
  }

  .n {
    display: inline-block;
    width: 4ch;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .word {
    color: var(--text-muted);
  }
</style>
