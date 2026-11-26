<script lang="ts">
  import {t, type I18nKey} from './lib/i18n.svelte';
  import type {GeneralPreferences} from './lib/preferences';

  let {prefs}: {prefs: GeneralPreferences} = $props();
  const executable = (window as Window & {__ZWEN_EXECUTABLE__?: string | null}).__ZWEN_EXECUTABLE__ || 'zwen';
  const shortcuts: {label: I18nKey; action: string}[] = [
    {label: 'shortcuts.quick', action: 'quick'},
    {label: 'shortcuts.selection', action: 'select'},
    {label: 'shortcuts.explain', action: 'explain-selection'},
    {label: 'shortcuts.translate', action: 'translate-selection'},
    {label: 'shortcuts.screenshot', action: 'screenshot'},
  ];
  const niriExample = `Mod+Alt+Z { spawn ${JSON.stringify(executable)} "quick"; }`;
</script>

<section class="shortcut-settings" aria-label={t('settings.shortcuts')}>
  <h1>{t('settings.shortcuts')}</h1>
  <div class="group-card">
    <h2>{t('shortcuts.application')}</h2>
    <dl>
      <div class="row"><dt>{t('general.sendKey')}</dt><dd><kbd>{prefs.sendKey === 'enter' ? 'Enter' : 'Ctrl + Enter'}</kbd></dd></div>
      <div class="row"><dt>{t('shortcuts.saveDraft')}</dt><dd><kbd>Ctrl + S</kbd></dd></div>
      <div class="row"><dt>{t('shortcuts.hide')}</dt><dd>{#if prefs.escToHide}<kbd>Esc</kbd>{:else}{t('shortcuts.disabled')}{/if}</dd></div>
    </dl>
  </div>
  <div class="group-card" id="niri-shortcuts" tabindex="-1">
    <h2>{t('shortcuts.global')}</h2>
    <dl>
      {#each shortcuts as shortcut (shortcut.action)}
        <div class="row"><dt>{t(shortcut.label)}</dt><dd><code>{shortcut.action}</code></dd></div>
      {/each}
    </dl>
    <p>{t('shortcuts.niriHint')}</p>
    <pre><code>{niriExample}</code></pre>
  </div>
</section>

<style>
  .shortcut-settings{max-width:600px;margin:20px auto;padding:0 18px 28px;color:var(--text);font-size:1rem}
  h1{margin:0 0 18px;font-size:1.35rem;font-weight:600}
  h2{margin:0 0 8px;color:var(--muted);font-size:.86rem;font-weight:600}
  .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px}
  dl,dd{margin:0}
  .row{display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:8px 14px;padding:9px 0;font-size:.95rem}
  .row+.row{border-top:1px solid var(--edge)}
  kbd,code{font-family:var(--font-mono);font-size:.88rem}
  kbd{padding:3px 6px;border:1px solid var(--edge);background:var(--surface-soft);border-radius:var(--radius-control)}
  p{margin:12px 0 8px;font-size:.85rem;color:var(--muted)}
  pre{margin:0;padding:8px 10px;border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface-soft);overflow:auto}
  @media(max-width:520px){.shortcut-settings{padding-inline:0}}
</style>
