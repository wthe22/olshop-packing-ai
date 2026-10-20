<script lang="ts">
  // The screens that open over the current one (08 › *Window frame*): Categories, Search and
  // Settings, each in the shared frame with its title. An open draft is kept; which one shows and
  // the set it opened on are the shell's own state, and the screens do their own work.
  import type { Shell } from "./shell.svelte";
  import Overlay from "./Overlay.svelte";
  import CategoriesScreen from "./CategoriesScreen.svelte";
  import SearchScreen from "./SearchScreen.svelte";
  import SettingsScreen from "./SettingsScreen.svelte";
  import { texts } from "../texts";

  let { shell }: { shell: Shell } = $props();
</script>

{#if shell.overlay === "categories"}
  <Overlay title={texts.categories.title} onClose={shell.closeOverlay}>
    <CategoriesScreen
      draftBatch={shell.draftBatch}
      openGrouping={shell.categoriesGrouping}
      onClose={shell.closeOverlay}
      onSaved={shell.onCategoriesSaved}
    />
  </Overlay>
{:else if shell.overlay === "search"}
  <Overlay title={texts.search.title} onClose={shell.closeOverlay}>
    <SearchScreen onOpen={shell.openFile} />
  </Overlay>
{:else if shell.overlay === "settings"}
  <Overlay title={texts.settings.title} onClose={shell.closeOverlay}>
    <SettingsScreen
      settings={shell.settings}
      about={shell.about}
      dataFolder={shell.dataFolder}
      onChangeFolder={shell.changeFolder}
      onOpenFolder={() => shell.dataFolder && shell.openFolder(shell.dataFolder)}
      onSave={shell.saveSettings}
    />
  </Overlay>
{/if}
