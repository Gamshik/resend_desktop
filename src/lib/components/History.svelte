<script lang="ts">
  import { api, errorText, type HistoryItem } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import EmailPreview from "./EmailPreview.svelte";
  import Icon from "./Icon.svelte";

  let items = $state<HistoryItem[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedId = $state<number | null>(null);
  let query = $state("");
  let confirmClear = $state(false);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items.filter(
      (i) => i.subject.toLowerCase().includes(q) || [...i.to, ...i.cc, ...i.bcc].some((a) => a.toLowerCase().includes(q)),
    );
  });
  const selected = $derived(items.find((i) => i.id === selectedId) ?? null);

  async function load() {
    loading = true;
    error = null;
    try {
      items = await api.listHistory(500);
      if (selectedId === null && items.length) selectedId = items[0].id;
    } catch (e) {
      error = errorText(e);
    } finally {
      loading = false;
    }
  }

  async function remove(id: number) {
    await api.deleteHistoryItem(id);
    items = items.filter((i) => i.id !== id);
    if (selectedId === id) selectedId = items[0]?.id ?? null;
  }

  async function clearAll() {
    await api.clearHistory();
    items = [];
    selectedId = null;
    confirmClear = false;
  }

  function when(iso: string) {
    const d = new Date(iso);
    const today = new Date().toDateString() === d.toDateString();
    return today
      ? d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
      : d.toLocaleDateString([], { day: "numeric", month: "short", year: "numeric" });
  }

  $effect(() => {
    load();
  });
</script>

<div class="layout">
  <aside class="list">
    <div class="list-head">
      <h1>History</h1>
      <input class="input" type="search" placeholder="Search subject or recipient" bind:value={query} />
    </div>

    {#if loading}
      <p class="empty hint">Loading…</p>
    {:else if error}
      <div class="notice error">{error}</div>
    {:else if filtered.length === 0}
      <div class="empty">
        <Icon name="mail" size={28} />
        <p>{items.length ? "Nothing matches your search." : "Emails you send will show up here."}</p>
      </div>
    {:else}
      <ul>
        {#each filtered as item (item.id)}
          <li>
            <button class="row" class:active={item.id === selectedId} onclick={() => (selectedId = item.id)}>
              <span class="top">
                <span class="to">{item.to.join(", ")}</span>
                <span class="time">{when(item.created_at)}</span>
              </span>
              <span class="subject">
                {#if item.status === "failed"}<span class="badge danger">Failed</span>{/if}
                {item.subject}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}

    {#if items.length}
      <div class="list-foot">
        {#if confirmClear}
          <span class="hint">Delete all local history?</span>
          <button class="btn sm danger" onclick={clearAll}>Delete all</button>
          <button class="btn sm ghost" onclick={() => (confirmClear = false)}>Cancel</button>
        {:else}
          <button class="btn sm ghost" onclick={() => (confirmClear = true)}><Icon name="trash" /> Clear history</button>
        {/if}
      </div>
    {/if}
  </aside>

  <section class="detail">
    {#if selected}
      <div class="detail-head">
        <h2>{selected.subject}</h2>
        <div class="tools">
          <button class="btn sm" onclick={() => app.reuse(selected)}><Icon name="refresh" /> Send again</button>
          <button class="btn sm ghost danger" title="Delete from history" onclick={() => remove(selected.id)}>
            <Icon name="trash" />
          </button>
        </div>
      </div>

      <dl class="meta">
        <dt>From</dt>
        <dd>{selected.from}</dd>
        <dt>To</dt>
        <dd>{selected.to.join(", ")}</dd>
        {#if selected.cc.length}<dt>Cc</dt><dd>{selected.cc.join(", ")}</dd>{/if}
        {#if selected.bcc.length}<dt>Bcc</dt><dd>{selected.bcc.join(", ")}</dd>{/if}
        <dt>Date</dt>
        <dd>{new Date(selected.created_at).toLocaleString()}</dd>
        <dt>Status</dt>
        <dd>
          {#if selected.status === "sent"}
            <span class="badge ok"><Icon name="check" size={12} /> Accepted by Resend</span>
            {#if selected.resend_id}<code class="rid">{selected.resend_id}</code>{/if}
          {:else}
            <span class="badge danger">Failed</span>
          {/if}
        </dd>
        {#if selected.attachments.length}
          <dt>Files</dt>
          <dd>{selected.attachments.join(", ")}</dd>
        {/if}
      </dl>

      {#if selected.error}<div class="notice error">{selected.error}</div>{/if}

      <div class="body">
        {#if selected.html}
          <EmailPreview html={selected.html} />
        {:else}
          <pre>{selected.text}</pre>
        {/if}
      </div>
    {:else if !loading}
      <div class="empty big">
        <Icon name="history" size={32} />
        <p>Select an email to see it here.</p>
      </div>
    {/if}
  </section>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: minmax(260px, 340px) 1fr;
    height: 100%;
    min-height: 0;
  }
  .list {
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: var(--surface);
  }
  .list-head {
    padding: 22px 16px 12px;
    display: grid;
    gap: 12px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0 8px;
    overflow-y: auto;
    flex: 1;
  }
  .row {
    width: 100%;
    text-align: left;
    font: inherit;
    color: inherit;
    border: 0;
    background: transparent;
    border-radius: var(--radius-sm);
    padding: 9px 10px;
    display: grid;
    gap: 2px;
    cursor: pointer;
  }
  .row:hover {
    background: var(--surface-2);
  }
  .row.active {
    background: var(--accent-soft);
  }
  .top {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }
  .to {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .time {
    flex: none;
    font-size: 12px;
    color: var(--text-3);
  }
  .subject {
    color: var(--text-2);
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .list-foot {
    padding: 10px 14px;
    border-top: 1px solid var(--border);
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .detail {
    padding: 24px 28px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
  }
  .detail-head {
    display: flex;
    justify-content: space-between;
    align-items: start;
    gap: 12px;
  }
  .detail-head h2 {
    font-size: 19px;
  }
  .tools {
    display: flex;
    gap: 4px;
    flex: none;
  }
  .meta {
    display: grid;
    grid-template-columns: 64px 1fr;
    gap: 5px 12px;
    margin: 0;
    font-size: 13.5px;
  }
  dt {
    color: var(--text-3);
  }
  dd {
    margin: 0;
    word-break: break-word;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }
  .rid {
    font-size: 12px;
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 300px;
  }
  pre {
    margin: 0;
    flex: 1;
    white-space: pre-wrap;
    font-family: var(--font);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 16px;
  }
  .empty {
    display: grid;
    justify-items: center;
    gap: 8px;
    padding: 40px 16px;
    color: var(--text-3);
    text-align: center;
  }
  .empty.big {
    margin: auto;
  }
</style>
