<script lang="ts">
  // The screen that opens over the current one: Settings, in the shared frame with its title. An
  // open draft is kept; whether it shows is the shell's own state.
  import type { Shell } from "./shell.svelte";
  import Overlay from "./Overlay.svelte";
  import SettingsScreen from "./SettingsScreen.svelte";
  import { texts } from "../texts";

  let { shell }: { shell: Shell } = $props();
</script>

{#if shell.overlay === "settings"}
  <Overlay title={texts.settings.title} onClose={shell.closeOverlay}>
    <SettingsScreen
      about={shell.about}
      dataFolder={shell.dataFolder}
      onChangeFolder={shell.changeFolder}
      onOpenFolder={() => shell.dataFolder && shell.openFolder(shell.dataFolder)}
    />
  </Overlay>
{/if}
