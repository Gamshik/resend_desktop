<script lang="ts">
  import { app } from "$lib/state.svelte";
  import ExternalLink from "./ExternalLink.svelte";
  import Icon from "./Icon.svelte";
  import KeyForm from "./KeyForm.svelte";
</script>

<div class="page">
  <header>
    <h1>Get set up</h1>
    <p class="lead">
      Four steps to send email from your own domain. Takes about 10 minutes, most of it waiting for DNS.
    </p>
  </header>

  <ol class="steps">
    <li class="card">
      <span class="num">1</span>
      <div>
        <h2>Create a Resend account</h2>
        <p>Sign up for free. The free plan covers 3,000 emails a month (100 a day).</p>
        <p class="links"><ExternalLink href="https://resend.com/signup">resend.com/signup</ExternalLink></p>
      </div>
    </li>

    <li class="card">
      <span class="num">2</span>
      <div>
        <h2>Add and verify your domain</h2>
        <p>
          Go to <b>Domains → Add Domain</b> and enter a domain you own (a subdomain like <code>mail.example.com</code>
          works well). Resend shows a few DNS records:
        </p>
        <ul>
          <li><b>SPF</b> (TXT + MX): lets Resend send for your domain.</li>
          <li><b>DKIM</b> (TXT): signs your emails so they aren't marked as spam.</li>
          <li><b>DMARC</b> (TXT, optional): tells inboxes what to do with unsigned mail.</li>
        </ul>
        <p>
          Copy them into your DNS provider (Cloudflare, Namecheap, …) and click <b>Verify</b>. It usually takes a few
          minutes, sometimes up to a few hours.
        </p>
        <p class="links">
          <ExternalLink href="https://resend.com/domains">Open Domains</ExternalLink>
          <ExternalLink href="https://resend.com/docs/dashboard/domains/introduction">Domain guide</ExternalLink>
        </p>
        <div class="notice warn">
          Without a verified domain you can only send from <code>onboarding@resend.dev</code>, and only to the email
          address of your Resend account.
        </div>
      </div>
    </li>

    <li class="card">
      <span class="num">3</span>
      <div>
        <h2>Create a sending-only API key</h2>
        <p>
          Go to <b>API Keys → Create API Key</b>. Set permission to <b>Sending access</b> and, if you like, limit it to
          your domain. Copy the key right away: Resend shows it only once.
        </p>
        <div class="why">
          <Icon name="shield" size={18} />
          <p>
            <b>Why sending-only?</b> If the key ever leaks, it can't read your emails, delete domains or create new
            keys. The worst case is someone sending mail as you, and you can revoke the key in one click.
          </p>
        </div>
        <p class="links"><ExternalLink href="https://resend.com/api-keys">Open API Keys</ExternalLink></p>
      </div>
    </li>

    <li class="card">
      <span class="num">4</span>
      <div class="grow">
        <h2>Paste the key here</h2>
        {#if app.keySaved}
          <div class="notice success saved">
            <Icon name="check" /> Key saved: <code>{app.keyMasked}</code>
            <button class="btn sm primary" onclick={() => (app.view = "compose")}>Write an email</button>
          </div>
        {:else}
          <KeyForm onsaved={() => (app.view = "compose")} />
        {/if}
      </div>
    </li>
  </ol>
</div>

<style>
  .page {
    max-width: 760px;
    margin: 0 auto;
    padding: 32px 32px 48px;
  }
  header {
    margin-bottom: 22px;
  }
  .lead {
    color: var(--text-2);
    margin-top: 6px;
  }
  .steps {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 14px;
  }
  .steps > li {
    display: flex;
    gap: 16px;
    padding: 20px 22px;
  }
  .steps > li > div {
    display: grid;
    gap: 10px;
    align-content: start;
    min-width: 0;
  }
  .grow {
    flex: 1;
  }
  .num {
    flex: none;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-weight: 650;
    font-size: 13px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  p {
    color: var(--text-2);
  }
  ul {
    margin: 0;
    padding-left: 18px;
    color: var(--text-2);
    display: grid;
    gap: 3px;
  }
  b {
    color: var(--text);
    font-weight: 600;
  }
  .links {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }
  .why {
    display: flex;
    gap: 10px;
    background: var(--surface-2);
    border-radius: var(--radius-sm);
    padding: 11px 13px;
    color: var(--accent);
  }
  .why p {
    font-size: 13.5px;
  }
  .saved {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .saved .btn {
    margin-left: auto;
  }
</style>
