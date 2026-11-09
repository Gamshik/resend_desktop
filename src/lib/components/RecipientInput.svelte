<script lang="ts">
  import { isAddress } from "$lib/api";
  import Icon from "./Icon.svelte";

  let {
    id,
    values = $bindable([]),
    placeholder = "",
  }: { id: string; values: string[]; placeholder?: string } = $props();

  let draft = $state("");

  /** Adds everything typed so far, split on commas, semicolons, spaces or newlines. */
  function commit() {
    const parts = draft
      .split(/[,;\s]+/)
      .map((s) => s.trim())
      .filter(Boolean);
    if (parts.length) {
      values = [...values, ...parts.filter((p) => !values.includes(p))];
    }
    draft = "";
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === "," || e.key === ";" || (e.key === " " && draft.includes("@"))) {
      if (draft.trim()) {
        e.preventDefault();
        commit();
      }
    } else if (e.key === "Backspace" && !draft && values.length) {
      values = values.slice(0, -1);
    }
  }

  function onpaste(e: ClipboardEvent) {
    const text = e.clipboardData?.getData("text") ?? "";
    if (/[,;\s]/.test(text.trim())) {
      e.preventDefault();
      draft += text;
      commit();
    }
  }

  function remove(v: string) {
    values = values.filter((x) => x !== v);
  }

  /** Lets the parent flush half-typed input before sending. */
  export function flush() {
    if (draft.trim()) commit();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="chips" onclick={() => document.getElementById(id)?.focus()}>
  {#each values as v (v)}
    <span class="chip" class:bad={!isAddress(v)} title={isAddress(v) ? v : "This doesn't look like an email address"}>
      {v}
      <button type="button" aria-label="Remove {v}" onclick={() => remove(v)}><Icon name="x" size={12} /></button>
    </span>
  {/each}
  <input
    {id}
    bind:value={draft}
    {onkeydown}
    {onpaste}
    onblur={commit}
    placeholder={values.length ? "" : placeholder}
    autocomplete="off"
    spellcheck="false"
  />
</div>

<style>
  .chips {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 5px;
    min-height: 38px;
    padding: 4px 6px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    cursor: text;
    transition: border-color 0.12s, box-shadow 0.12s;
  }
  .chips:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  input {
    flex: 1;
    min-width: 140px;
    border: 0;
    outline: none;
    background: transparent;
    font: inherit;
    color: var(--text);
    padding: 4px 5px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: var(--accent-soft);
    color: var(--text);
    border-radius: 999px;
    padding: 2px 4px 2px 10px;
    font-size: 13px;
  }
  .chip.bad {
    background: var(--danger-soft);
    color: var(--danger);
  }
  .chip button {
    border: 0;
    background: transparent;
    color: inherit;
    opacity: 0.6;
    border-radius: 50%;
    padding: 3px;
    display: grid;
    cursor: pointer;
  }
  .chip button:hover {
    opacity: 1;
    background: rgb(0 0 0 / 0.08);
  }
</style>
