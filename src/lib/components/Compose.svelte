<script lang="ts">
  import { api, errorText, isAddress, type Attachment } from "$lib/api";
  import { app } from "$lib/state.svelte";
  import EmailPreview from "./EmailPreview.svelte";
  import Icon from "./Icon.svelte";
  import RecipientInput from "./RecipientInput.svelte";

  let from = $state(app.defaultFrom ?? app.lastFrom ?? "");
  let to = $state<string[]>([]);
  let cc = $state<string[]>([]);
  let bcc = $state<string[]>([]);
  let replyTo = $state("");
  let subject = $state("");
  let mode = $state<"text" | "html">("text");
  let body = $state("");
  let files = $state<(Attachment & { size: number })[]>([]);
  let showCopies = $state(false);

  let sending = $state(false);
  let error = $state<string | null>(null);
  let sentId = $state<string | null>(null);

  let toInput: RecipientInput | undefined = $state();
  let ccInput: RecipientInput | undefined = $state();
  let bccInput: RecipientInput | undefined = $state();
  let fileInput: HTMLInputElement | undefined = $state();

  // Load a draft handed over from History ("Send again").
  $effect(() => {
    const d = app.pendingDraft;
    if (!d) return;
    from = d.from;
    to = d.to;
    cc = d.cc;
    bcc = d.bcc;
    showCopies = d.cc.length > 0 || d.bcc.length > 0;
    subject = d.subject;
    mode = d.mode;
    body = d.body;
    files = [];
    error = null;
    sentId = null;
    app.pendingDraft = null;
  });

  const fromSuggestions = $derived(app.verifiedDomains.map((d) => `you@${d.name}`));
  const noVerifiedDomain = $derived(app.keyAccess === "full" && app.domains !== null && app.verifiedDomains.length === 0);
  const totalSize = $derived(files.reduce((n, f) => n + f.size, 0));
  // `to` isn't required here: half-typed recipients are only committed when Send flushes the inputs.
  const canSend = $derived(!sending && from.trim() !== "" && subject.trim() !== "" && body.trim() !== "");

  function fmtSize(n: number) {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
    return `${(n / 1024 / 1024).toFixed(1)} MB`;
  }

  function readFile(file: File): Promise<Attachment & { size: number }> {
    return new Promise((resolve, reject) => {
      const r = new FileReader();
      r.onload = () => {
        const url = String(r.result);
        resolve({ filename: file.name, content: url.slice(url.indexOf(",") + 1), size: file.size });
      };
      r.onerror = () => reject(r.error);
      r.readAsDataURL(file);
    });
  }

  async function addFiles(list: FileList | null) {
    if (!list) return;
    const read = await Promise.all([...list].map(readFile));
    files = [...files, ...read];
    if (fileInput) fileInput.value = "";
  }

  async function send() {
    toInput?.flush();
    ccInput?.flush();
    bccInput?.flush();
    error = null;
    sentId = null;

    const bad = [...to, ...cc, ...bcc].filter((a) => !isAddress(a));
    if (to.length === 0) return void (error = "Add at least one recipient.");
    if (bad.length) return void (error = `These addresses look wrong: ${bad.join(", ")}`);
    if (!isAddress(from)) return void (error = "The From address looks wrong. Use you@domain.com or Name <you@domain.com>.");

    sending = true;
    try {
      const res = await api.sendEmail({
        from,
        to,
        cc,
        bcc,
        reply_to: replyTo.trim() ? [replyTo.trim()] : [],
        subject,
        html: mode === "html" ? body : null,
        text: mode === "text" ? body : null,
        attachments: files.map(({ filename, content }) => ({ filename, content })),
      });
      sentId = res.id;
      app.lastFrom = from;
      to = [];
      cc = [];
      bcc = [];
      subject = "";
      body = "";
      files = [];
    } catch (e) {
      error = errorText(e);
    } finally {
      sending = false;
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter" && canSend) {
      e.preventDefault();
      send();
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="page">
  <header>
    <h1>New email</h1>
    <span class="hint">Ctrl + Enter to send</span>
  </header>

  {#if noVerifiedDomain}
    <div class="notice warn">
      No verified domain yet. Until one is verified, send from <code>onboarding@resend.dev</code> to your own Resend
      account email.
      <button class="linkish" onclick={() => (app.view = "guide")}>How to verify a domain</button>
    </div>
  {/if}

  <div class="card form">
    <div class="field-row">
      <label class="label" for="from">From</label>
      <input
        id="from"
        class="input"
        bind:value={from}
        list="from-suggestions"
        placeholder="Your Name <you@yourdomain.com>"
        spellcheck="false"
      />
      <datalist id="from-suggestions">
        {#each fromSuggestions as s (s)}<option value={s}></option>{/each}
        <option value="onboarding@resend.dev"></option>
      </datalist>
    </div>

    <div class="field-row">
      <label class="label" for="to">To</label>
      <div class="with-action">
        <RecipientInput id="to" bind:this={toInput} bind:values={to} placeholder="name@example.com" />
        {#if !showCopies}
          <button class="btn ghost sm" onclick={() => (showCopies = true)}>Cc / Bcc</button>
        {/if}
      </div>
    </div>

    {#if showCopies}
      <div class="field-row">
        <label class="label" for="cc">Cc</label>
        <RecipientInput id="cc" bind:this={ccInput} bind:values={cc} />
      </div>
      <div class="field-row">
        <label class="label" for="bcc">Bcc</label>
        <RecipientInput id="bcc" bind:this={bccInput} bind:values={bcc} />
      </div>
      <div class="field-row">
        <label class="label" for="reply-to">Reply-To</label>
        <input id="reply-to" class="input" bind:value={replyTo} placeholder="Optional" spellcheck="false" />
      </div>
    {/if}

    <div class="field-row">
      <label class="label" for="subject">Subject</label>
      <input id="subject" class="input" bind:value={subject} />
    </div>

    <div class="body-head">
      <div class="seg" role="tablist">
        <button role="tab" aria-selected={mode === "text"} class:on={mode === "text"} onclick={() => (mode = "text")}>
          Plain text
        </button>
        <button role="tab" aria-selected={mode === "html"} class:on={mode === "html"} onclick={() => (mode = "html")}>
          HTML
        </button>
      </div>
      <button class="btn ghost sm" onclick={() => fileInput?.click()}><Icon name="paperclip" /> Attach</button>
      <input type="file" multiple hidden bind:this={fileInput} onchange={(e) => addFiles(e.currentTarget.files)} />
    </div>

    {#if mode === "text"}
      <textarea class="textarea" bind:value={body} placeholder="Write your message…"></textarea>
    {:else}
      <div class="split">
        <textarea class="textarea mono" bind:value={body} placeholder="<h1>Hello</h1>&#10;<p>…</p>"></textarea>
        <EmailPreview html={body} />
      </div>
    {/if}

    {#if files.length}
      <ul class="files">
        {#each files as f, i (f.filename + i)}
          <li>
            <Icon name="paperclip" size={13} />
            <span class="name">{f.filename}</span>
            <span class="hint">{fmtSize(f.size)}</span>
            <button class="x" aria-label="Remove {f.filename}" onclick={() => (files = files.filter((_, j) => j !== i))}>
              <Icon name="x" size={13} />
            </button>
          </li>
        {/each}
      </ul>
      {#if totalSize > 40 * 1024 * 1024}
        <div class="notice error">Attachments total {fmtSize(totalSize)}. Resend's limit is 40 MB.</div>
      {/if}
    {/if}

    {#if error}<div class="notice error">{error}</div>{/if}
    {#if sentId}
      <div class="notice success sent">
        <Icon name="check" /> Sent. Resend ID <code>{sentId}</code>
      </div>
    {/if}

    <div class="actions">
      <button class="btn primary" onclick={send} disabled={!canSend}>
        {#if sending}<span class="spinner"></span> Sending…{:else}<Icon name="send" /> Send{/if}
      </button>
    </div>
  </div>
</div>

<style>
  .page {
    max-width: 920px;
    margin: 0 auto;
    padding: 28px 32px 40px;
    display: grid;
    gap: 14px;
  }
  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .form {
    padding: 20px 22px;
    display: grid;
    gap: 14px;
  }
  .field-row {
    display: grid;
    grid-template-columns: 76px 1fr;
    align-items: center;
    gap: 10px;
  }
  .field-row .label {
    margin: 0;
  }
  .with-action {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .with-action > :global(.chips) {
    flex: 1;
  }
  .body-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 4px;
  }
  .seg {
    display: inline-flex;
    background: var(--surface-2);
    border-radius: var(--radius-sm);
    padding: 3px;
  }
  .seg button {
    font: inherit;
    font-size: 13px;
    font-weight: 550;
    border: 0;
    background: transparent;
    color: var(--text-2);
    padding: 4px 12px;
    border-radius: 5px;
    cursor: pointer;
  }
  .seg button.on {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow);
  }
  .split {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    min-height: 300px;
  }
  .split .textarea {
    min-height: 300px;
  }
  .files {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .files li {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 4px 6px 4px 9px;
    font-size: 13px;
  }
  .files .name {
    max-width: 240px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .x {
    border: 0;
    background: transparent;
    color: var(--text-3);
    padding: 2px;
    display: grid;
    cursor: pointer;
    border-radius: 4px;
  }
  .x:hover {
    color: var(--danger);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
  .sent {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .linkish {
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    font-weight: 600;
    text-decoration: underline;
    cursor: pointer;
    padding: 0;
    margin-left: 4px;
  }
</style>
