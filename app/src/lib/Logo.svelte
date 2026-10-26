<script lang="ts">
  // The program's icon, from the one source every icon file is made from: the mark beside the
  // name in the title bar, on the first-start cards and in *About*.
  import logo from "../../src-tauri/icons/icon.svg";

  let { size }: { size: number } = $props();
</script>

<img src={logo} alt="" width={size} height={size} />

<style>
  img {
    display: block;
    flex: none;
  }
</style>
