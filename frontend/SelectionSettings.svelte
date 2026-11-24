<script lang="ts">
 import type {Snippet} from 'svelte';
 import {t} from './lib/i18n.svelte';
 let {desktop,enabled,onToggle,children}:{desktop:boolean;enabled:boolean;onToggle:(on:boolean)=>void;children?:Snippet}=$props();
</script>
<section aria-label={t('selection.title')} class="selection-settings">
 <h1>{t('selection.title')}</h1>
 <div class="group-card">
  <h2>{t('selection.triggerGroup')}</h2>
  {#if desktop}
   <div class="setting-row">
    <div class="setting-label">
     <label for="candidate-toggle">{t('selection.autoMenu')}</label>
     <details class="hint-disclosure inline-hint"><summary>{t('general.helpSummary')}</summary><div class="hint-disclosure-body"><p>{t('selection.autoMenuHint')}</p></div></details>
    </div>
    <input id="candidate-toggle" type="checkbox" class="toggle-check" checked={enabled} onchange={e=>onToggle(e.currentTarget.checked)}/>
   </div>
  {:else}
   <p class="preview-note">{t('selection.previewNote')}</p>
  {/if}
 </div>
 {#if children}{@render children()}{/if}
</section>
<style>
 .selection-settings{max-width:600px;margin:20px auto;padding:0 18px 28px;color:var(--text);font-size:1rem}
 h1{font-size:1.35rem;font-weight:600;margin:0 0 18px}
 h2{margin:0 0 6px;font-size:.86rem;font-weight:600;color:var(--muted);letter-spacing:.02em}
 .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px}
 .setting-row{display:flex;align-items:center;justify-content:space-between;gap:14px;padding:6px 0 2px}
 .setting-label{display:flex;align-items:center;gap:10px;min-width:0;flex-wrap:wrap}
 .setting-label label{font-size:.96rem;font-weight:500;color:var(--text);cursor:pointer}
 .inline-hint{margin-top:0}
 .toggle-check{width:16px;height:16px;margin:0;accent-color:var(--accent);cursor:pointer;flex-shrink:0}
 .preview-note{margin:6px 0 0;font-size:.9rem;color:var(--muted)}
 @media(max-width:520px){.selection-settings{padding-inline:0}}
</style>
