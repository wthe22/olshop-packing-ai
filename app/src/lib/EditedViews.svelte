<script lang="ts">
  // The date's files changed outside the program: each named with the first lines that
  // differ from the journal, then **Rebuild**, **Read only** and **Ignore**, each titled with what
  // it does. Once the owner answered *Read only*, one line says so and no button is left.
  import { texts } from "../texts";
  import type { EditedChoice, EditedNote } from "./api/day";
  import { access } from "./readOnly.svelte";

  let {
    note,
    onEdited,
  }: {
    note: EditedNote;
    onEdited: (choice: EditedChoice) => void;
  } = $props();

  const answers: { choice: EditedChoice; label: string; title: string }[] = [
    { choice: "rebuild", label: texts.day.rebuild, title: texts.day.rebuildTitle },
    { choice: "read_only", label: texts.day.readOnly, title: texts.day.readOnlyTitle },
    { choice: "ignore", label: texts.day.ignore, title: texts.day.ignoreTitle },
  ];
</script>

{#if note.files.length > 0}
  <section class="edited">
    <p class="head">{texts.day.edited(note.files.map((file) => file.file))}</p>
    {#each note.files as file (file.file)}
      {#if file.lines.length > 0}
        <table>
          <thead>
            <tr>
              <th>{file.file}</th>
              <th>{texts.day.editedJournal}</th>
              <th>{texts.day.editedFile}</th>
            </tr>
          </thead>
          <tbody>
            {#each file.lines as line, i (i)}
              <tr>
                <td class="number">{texts.day.editedLine(line.number, line.column)}</td>
                <td><code>{line.written}</code></td>
                <td><code>{line.found}</code></td>
              </tr>
            {/each}
          </tbody>
        </table>
        {#if file.more > 0}
          <p class="more">{texts.day.editedMore(file.more)}</p>
        {/if}
      {/if}
    {/each}
    {#if note.read_only}
      <p class="foot">{texts.day.editedReadOnly}</p>
    {:else if !access.readOnly}
      <div class="foot">
        {#each answers as answer (answer.choice)}
          <button title={answer.title} onclick={() => onEdited(answer.choice)}>{answer.label}</button>
        {/each}
      </div>
    {/if}
  </section>
{/if}

<style>
  .edited {
    margin: 0 0 0.9rem;
    padding: 0.6rem 0.75rem;
    border: 1px solid var(--warn-border);
    border-radius: var(--radius-lg);
    background: var(--warn-bg);
    color: var(--warn);
  }

  .head,
  .more {
    margin: 0 0 0.5rem;
  }

  table {
    width: 100%;
    margin-bottom: 0.5rem;
    border: 1px solid var(--border);
    border-collapse: separate;
    border-spacing: 0;
    border-radius: var(--radius);
    background: var(--surface);
    color: var(--text);
    overflow: hidden;
  }

  th {
    padding: 0.3rem 0.5rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-muted);
    font-size: 0.78rem;
    font-weight: 600;
    text-align: left;
  }

  td {
    padding: 0.3rem 0.5rem;
    vertical-align: top;
    word-break: break-all;
  }

  td.number {
    width: 1%;
    color: var(--text-muted);
    white-space: nowrap;
    word-break: normal;
  }

  .foot {
    display: flex;
    gap: 0.5rem;
    margin: 0;
  }
</style>
