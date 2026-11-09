<script lang="ts">
  import { api, errorText } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import Icon from "./Icon.svelte";

  let { onsaved }: { onsaved?: () => void } = $props();

  let value = $state("");
  let reveal = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    if (!value.trim()) return;
    busy = true;
    error = null;
    try {
      const res = await api.saveKey(value);
      app.keyStored(res.masked, res.check);
      value = ""; // drop the key from webview memory as soon as Rust has it
      onsaved?.();
    } catch (err) {
      error = errorText(err);
    } finally {
      busy = false;
    }
  }
</script>

<form class="key-form" onsubmit={submit}>
  <label class="label" for="api-key">Resend API key</label>
  <div class="row">
    <div class="field">
      <input
        id="api-key"
        class="input mono"
        type={reveal ? "text" : "password"}
        placeholder="re_xxxxxxxxxxxxxxxxxxxxxxxx"
        autocomplete="off"
        spellcheck="false"
        bind:value
        disabled={busy}
      />
      <button
        type="button"
        class="reveal"
        title={reveal ? "Hide key" : "Show key"}
        onclick={() => (reveal = !reveal)}
      >
        <Icon name={reveal ? "eye-off" : "eye"} />
      </button>
    </div>
    <button class="btn primary" type="submit" disabled={busy || !value.trim()}>
      {#if busy}<span class="spinner"></span> Checking…{:else}<Icon name="key" /> Save key{/if}
    </button>
  </div>
  <p class="hint">
    The key is checked with Resend, then stored in your system credential store (Credential Manager on Windows, Keychain on macOS). It never
    leaves this computer except to talk to Resend.
  </p>
  {#if error}<div class="notice error">{error}</div>{/if}
</form>

<style>
  .key-form {
    display: grid;
    gap: 8px;
  }
  .row {
    display: flex;
    gap: 8px;
  }
  .field {
    position: relative;
    flex: 1;
  }
  .mono {
    font-family: var(--mono);
    font-size: 13px;
    padding-right: 38px;
  }
  .reveal {
    position: absolute;
    right: 4px;
    top: 50%;
    translate: 0 -50%;
    border: 0;
    background: transparent;
    color: var(--text-3);
    padding: 6px;
    border-radius: 6px;
    cursor: pointer;
    display: grid;
  }
  .reveal:hover {
    color: var(--text);
    background: var(--surface-2);
  }
</style>
