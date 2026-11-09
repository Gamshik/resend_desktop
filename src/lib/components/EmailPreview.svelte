<script lang="ts">
  // Renders untrusted email HTML in a fully sandboxed iframe:
  // `sandbox=""` blocks scripts, forms, popups, top navigation and same-origin access,
  // so nothing in the preview can reach the Tauri IPC bridge.
  let { html }: { html: string } = $props();

  const srcdoc = $derived(
    `<!doctype html><html><head><meta charset="utf-8"><base target="_blank">` +
      `<style>body{margin:16px;font-family:system-ui,sans-serif;color:#111;background:#fff}</style>` +
      `</head><body>${html}</body></html>`,
  );
</script>

<iframe title="Email preview" sandbox="" {srcdoc}></iframe>

<style>
  iframe {
    width: 100%;
    height: 100%;
    min-height: 260px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: #fff;
  }
</style>
