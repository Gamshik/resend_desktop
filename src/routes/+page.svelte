<script lang="ts">
  import { errorText } from "$lib/api";
  import Compose from "$lib/components/Compose.svelte";
  import Guide from "$lib/components/Guide.svelte";
  import History from "$lib/components/History.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import { app, type View } from "$lib/state.svelte";

  let initError = $state<string | null>(null);

  $effect(() => {
    (async () => {
      // In a plain browser (`pnpm dev` without Tauri) swap in a fake backend for UI work.
      if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
        (await import("$lib/dev-mock")).installDevMock();
      }
      await app.init();
    })().catch((e) => (initError = errorText(e)));
  });

  const nav: {
    view: View;
    label: string;
    short: string;
    icon: "send" | "history" | "settings" | "book";
    needsKey: boolean;
  }[] = [
    { view: "compose", label: "Compose", short: "Compose", icon: "send", needsKey: true },
    { view: "history", label: "History", short: "History", icon: "history", needsKey: false },
    { view: "settings", label: "Settings", short: "Settings", icon: "settings", needsKey: false },
    { view: "guide", label: "Setup guide", short: "Guide", icon: "book", needsKey: false },
  ];
</script>

<div class="shell">
  <nav>
    <div class="brand">
      <span class="logo"><Icon name="mail" size={16} /></span>
      <span>Resend Desktop</span>
    </div>

    <ul>
      {#each nav as item (item.view)}
        <li>
          <button
            class:active={app.view === item.view}
            disabled={item.needsKey && !app.keySaved}
            title={item.needsKey && !app.keySaved ? "Add an API key first" : undefined}
            onclick={() => (app.view = item.view)}
          >
            <Icon name={item.icon} />
            <span class="long">{item.label}</span>
            <span class="short">{item.short}</span>
          </button>
        </li>
      {/each}
    </ul>

    <div class="status">
      {#if app.keySaved}
        <span class="dot ok"></span>
        <span>Key <code>{app.keyMasked}</code></span>
      {:else if app.ready}
        <span class="dot"></span>
        <span>No API key</span>
      {/if}
    </div>
  </nav>

  <main>
    {#if initError}
      <div class="notice error boot">Couldn't start: {initError}</div>
    {:else if !app.ready}
      <div class="boot"><span class="spinner"></span></div>
    {:else if app.view === "compose" && app.keySaved}
      <Compose />
    {:else if app.view === "history"}
      <History />
    {:else if app.view === "settings"}
      <Settings />
    {:else}
      <Guide />
    {/if}
  </main>
</div>

<style>
  .shell {
    display: grid;
    grid-template-columns: 212px 1fr;
    height: 100vh;
    height: 100dvh;
  }
  .short {
    display: none;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 16px 10px;
    border-right: 1px solid var(--border);
    background: var(--bg);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    font-weight: 650;
    padding: 4px 8px 16px;
  }
  .logo {
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    background: var(--text);
    color: var(--bg);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }
  nav button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    font: inherit;
    font-weight: 500;
    color: var(--text-2);
    background: transparent;
    border: 0;
    border-radius: var(--radius-sm);
    padding: 7px 10px;
    cursor: pointer;
    text-align: left;
  }
  nav button:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--text);
  }
  nav button.active {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow);
  }
  nav button:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .status {
    margin-top: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-3);
    padding: 8px;
  }
  .status code {
    font-size: 11px;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-3);
    flex: none;
  }
  .dot.ok {
    background: var(--ok);
  }
  main {
    overflow-y: auto;
    min-height: 0;
  }
  main:has(> :global(.layout)) {
    overflow: hidden;
  }
  .boot {
    display: grid;
    place-items: center;
    height: 100%;
    color: var(--text-3);
  }
  .notice.boot {
    height: auto;
    margin: 32px;
  }

  /* Phones: sidebar becomes a bottom tab bar. */
  @media (max-width: 720px) {
    .shell {
      grid-template-columns: 1fr;
      grid-template-rows: 1fr auto;
    }
    main {
      grid-row: 1;
      padding-top: env(safe-area-inset-top);
    }
    nav {
      grid-row: 2;
      flex-direction: row;
      padding: 4px 6px calc(4px + env(safe-area-inset-bottom));
      border-right: 0;
      border-top: 1px solid var(--border);
      background: var(--surface);
    }
    .brand,
    .status,
    .long {
      display: none;
    }
    .short {
      display: inline;
    }
    ul {
      flex: 1;
      grid-auto-flow: column;
      grid-auto-columns: 1fr;
    }
    nav button {
      flex-direction: column;
      gap: 3px;
      padding: 7px 4px;
      font-size: 11px;
      font-weight: 600;
      text-align: center;
    }
    nav button.active {
      background: transparent;
      box-shadow: none;
      color: var(--accent);
    }
  }
</style>
