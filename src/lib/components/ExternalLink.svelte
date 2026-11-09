<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  // Opens in the system browser, never inside the app window.
  let { href, children }: { href: string; children: Snippet } = $props();
</script>

<a
  {href}
  onclick={(e) => {
    e.preventDefault();
    openUrl(href);
  }}
>
  {@render children()}<Icon name="external" size={12} />
</a>

<style>
  a {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    text-decoration: none;
    font-weight: 550;
  }
  a:hover {
    text-decoration: underline;
  }
</style>
