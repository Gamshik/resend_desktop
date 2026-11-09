<script lang="ts">
  import { api, errorText, isAddress } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import ExternalLink from "./ExternalLink.svelte";
  import Icon from "./Icon.svelte";
  import KeyForm from "./KeyForm.svelte";

  let replacing = $state(false);
  let confirmRemove = $state(false);
  let checking = $state(false);
  let checkMsg = $state<{ kind: "success" | "error"; text: string } | null>(null);

  let defaultFrom = $state(app.defaultFrom ?? "");
  let fromSaved = $state(false);
  let fromError = $state<string | null>(null);

  async function testKey() {
    checking = true;
    checkMsg = null;
    try {
      const c = await api.checkKey();
      app.applyCheck(c);
      checkMsg = { kind: "success", text: "Resend accepted the key." };
    } catch (e) {
      checkMsg = { kind: "error", text: errorText(e) };
    } finally {
      checking = false;
    }
  }

  async function removeKey() {
    await api.deleteKey();
    app.keyRemoved();
    confirmRemove = false;
    checkMsg = null;
  }

  async function saveFrom() {
    fromError = null;
    if (defaultFrom.trim() && !isAddress(defaultFrom)) {
      fromError = "Use you@domain.com or Name <you@domain.com>.";
      return;
    }
    try {
      await api.setDefaultFrom(defaultFrom);
      app.defaultFrom = defaultFrom.trim() || null;
      fromSaved = true;
      setTimeout(() => (fromSaved = false), 1600);
    } catch (e) {
      fromError = errorText(e);
    }
  }
</script>

<div class="page">
  <h1>Settings</h1>

  <section class="card">
    <div class="sec-head">
      <h2>API key</h2>
      {#if app.keyAccess === "sending_only"}
        <span class="badge ok"><Icon name="shield" size={12} /> Sending only</span>
      {:else if app.keyAccess === "full"}
        <span class="badge warn">Full access</span>
      {/if}
    </div>

    {#if app.keySaved && !replacing}
      <div class="key-row">
        <Icon name="key" />
        <code class="masked">{app.keyMasked}</code>
        <span class="hint">Stored in secure system storage</span>
      </div>

      {#if app.keyAccess === "full"}
        <div class="notice warn">
          This key has full access: it can also read emails, manage domains and create keys. For day-to-day sending, a
          <b>Sending access</b> key is safer. <ExternalLink href="https://resend.com/api-keys">Create one</ExternalLink>
        </div>
      {/if}

      <div class="btns">
        <button class="btn" onclick={testKey} disabled={checking}>
          {#if checking}<span class="spinner"></span>{:else}<Icon name="check" />{/if} Test key
        </button>
        <button class="btn" onclick={() => (replacing = true)}><Icon name="refresh" /> Replace</button>
        {#if confirmRemove}
          <span class="hint confirm">Remove the key from this computer?</span>
          <button class="btn danger" onclick={removeKey}>Remove</button>
          <button class="btn ghost" onclick={() => (confirmRemove = false)}>Cancel</button>
        {:else}
          <button class="btn ghost danger" onclick={() => (confirmRemove = true)}><Icon name="trash" /> Remove</button>
        {/if}
      </div>
      {#if checkMsg}<div class="notice {checkMsg.kind}">{checkMsg.text}</div>{/if}
    {:else}
      <KeyForm onsaved={() => (replacing = false)} />
      {#if replacing}
        <div><button class="btn ghost sm" onclick={() => (replacing = false)}>Cancel</button></div>
      {/if}
    {/if}
  </section>

  {#if app.keySaved}
    <section class="card">
      <div class="sec-head">
        <h2>Domains</h2>
        <ExternalLink href="https://resend.com/domains">Manage in Resend</ExternalLink>
      </div>
      {#if app.keyAccess === "sending_only"}
        <p class="hint">
          Sending-only keys can't list domains, which is part of what makes them safe. Check your domains on the Resend
          dashboard.
        </p>
      {:else if app.domains === null}
        <p class="hint">Click <b>Test key</b> to load your domains.</p>
      {:else if app.domains.length === 0}
        <p class="hint">No domains yet. Add one in Resend to send from your own address.</p>
      {:else}
        <ul class="domains">
          {#each app.domains as d (d.name)}
            <li>
              <span>{d.name}</span>
              <span class="badge {d.status === 'verified' ? 'ok' : d.status === 'failed' ? 'danger' : 'warn'}">
                {d.status}
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}

  <section class="card">
    <div class="sec-head"><h2>Default sender</h2></div>
    <p class="hint">Pre-filled in the From field of every new email.</p>
    <div class="from-row">
      <input class="input" bind:value={defaultFrom} placeholder="Your Name <you@yourdomain.com>" spellcheck="false" />
      <button class="btn" onclick={saveFrom}>
        {#if fromSaved}<Icon name="check" /> Saved{:else}Save{/if}
      </button>
    </div>
    {#if fromError}<div class="notice error">{fromError}</div>{/if}
  </section>
</div>

<style>
  .page {
    max-width: 720px;
    margin: 0 auto;
    padding: 28px 32px 48px;
    display: grid;
    gap: 16px;
  }
  section {
    padding: 20px 22px;
    display: grid;
    gap: 12px;
  }
  .sec-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .key-row {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--text-2);
  }
  .masked {
    font-size: 14px;
    padding: 4px 9px;
  }
  .btns {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
  }
  .confirm {
    margin-left: 4px;
  }
  .domains {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 6px;
  }
  .domains li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--surface-2);
    border-radius: var(--radius-sm);
  }
  .from-row {
    display: flex;
    gap: 8px;
  }
  b {
    font-weight: 600;
  }

  @media (max-width: 720px) {
    .page {
      padding: 16px 12px 24px;
      gap: 12px;
    }
    section {
      padding: 16px 14px;
    }
    .key-row {
      flex-wrap: wrap;
      gap: 6px 10px;
    }
  }
</style>
