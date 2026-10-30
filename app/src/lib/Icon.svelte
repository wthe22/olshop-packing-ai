<script lang="ts" module>
  // The line icons of the title bar, the panel, the dialogs, Settings and Sort rules (24 × 24, drawn with the text colour).
  const paths = {
    panel: "M3 5h18v14H3zM9 5v14",
    search: "M10.5 4a6.5 6.5 0 1 1 0 13 6.5 6.5 0 0 1 0-13zM15.5 15.5 21 21",
    rules: "M4 6h16M4 12h10M4 18h6",
    settings:
      "M12 9a3 3 0 1 1 0 6 3 3 0 0 1 0-6zM12 2v3M12 19v3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M2 12h3M19 12h3M4.9 19.1 7 17M17 7l2.1-2.1",
    light:
      "M12 8a4 4 0 1 1 0 8 4 4 0 0 1 0-8zM12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4",
    dark: "M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z",
    system: "M3 4h18v12H3zM8 20h8M12 16v4",
    plus: "M12 5v14M5 12h14",
    close: "M6 6l12 12M18 6 6 18",
    check: "M5 12.5l4.5 4.5L19 7",
    folder: "M3 6h6l2 2h10v11H3z",
    file: "M6 3h8l4 4v14H6zM14 3v4h4",
    chevron: "M9 6l6 6-6 6",
    up: "M12 19V5M6 11l6-6 6 6",
    down: "M12 5v14M6 13l6 6 6-6",
    trash: "M4 7h16M10 7V4h4v3M6 7l1 13h10l1-13M10 11v6M14 11v6",
  } as const;

  export type IconName = keyof typeof paths;
</script>

<script lang="ts">
  let { name, size = 18 }: { name: IconName; size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"><path d={paths[name]} /></svg
>

<style>
  svg {
    flex: none;
    vertical-align: middle;
  }
</style>
