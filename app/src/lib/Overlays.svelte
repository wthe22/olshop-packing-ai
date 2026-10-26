<script lang="ts">
  // The screens that open over the current one: Search, Sort rules and Settings, in the shared frame with
  // their titles. An open draft is kept; whether it shows is the shell's own state.
  import type { Shell } from "./shell.svelte";
  import Overlay from "./Overlay.svelte";
  import SearchScreen from "./SearchScreen.svelte";
  import SortRulesScreen from "./SortRulesScreen.svelte";
  import SettingsScreen from "./SettingsScreen.svelte";
  import { texts } from "../texts";

  let { shell }: { shell: Shell } = $props();
</script>

{#if shell.overlay === "search"}
  <Overlay title={texts.search.title} onClose={shell.closeOverlay}>
    <SearchScreen day={shell.day} onOpen={shell.openFile} onReprinted={shell.afterReprint} />
  </Overlay>
{:else if shell.overlay === "sort-rules"}
  <Overlay title={texts.sortRules.title} onClose={shell.closeOverlay}>
    <SortRulesScreen
      draftBatch={shell.draftOpen ? (shell.readResult?.amend ?? shell.nextBatch) : null}
      onClose={shell.closeOverlay}
      onSaved={shell.afterRulesSaved}
    />
  </Overlay>
{:else if shell.overlay === "settings"}
  <Overlay title={texts.settings.title} onClose={shell.closeOverlay}>
    <SettingsScreen
      about={shell.about}
      dataFolder={shell.dataFolder}
      onChangeFolder={shell.changeFolder}
      onOpenFolder={() => shell.dataFolder && shell.openFolder(shell.dataFolder)}
    />
  </Overlay>
{/if}
