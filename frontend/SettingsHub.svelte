<script lang="ts">
 import type {Snippet} from 'svelte';
 import {t} from './lib/i18n.svelte';
 export type SettingsCategory='general'|'provider'|'appearance'|'selection'|'archive';
 let {category=$bindable<SettingsCategory>('provider'),generalPanel,appearancePanel,providerPanel,selectionPanel,archivePanel}:{category?:SettingsCategory;generalPanel:Snippet;appearancePanel:Snippet;providerPanel:Snippet;selectionPanel:Snippet;archivePanel?:Snippet}=$props();
 const categories=$derived([
  {id:'general' as const,label:t('settings.general'),detail:t('settings.generalDetail')},
  {id:'provider' as const,label:t('settings.provider'),detail:t('settings.providerDetail')},
  {id:'appearance' as const,label:t('settings.appearance'),detail:t('settings.appearanceDetail')},
  {id:'selection' as const,label:t('settings.selection'),detail:t('settings.selectionDetail')},
  {id:'archive' as const,label:t('settings.archive'),detail:t('settings.archiveDetail')},
 ]);
 function move(event:KeyboardEvent,index:number){
  let next=index;
  if(event.key==='ArrowDown'||event.key==='ArrowRight')next=(index+1)%categories.length;
  else if(event.key==='ArrowUp'||event.key==='ArrowLeft')next=(index+categories.length-1)%categories.length;
  else if(event.key==='Home')next=0;
  else if(event.key==='End')next=categories.length-1;
  else return;
  event.preventDefault();category=categories[next].id;document.getElementById(`settings-tab-${category}`)?.focus();
 }
</script>
<div class="settings-hub">
 <nav aria-label={t('settings.categories')}><div class="category-list" role="tablist" aria-label={t('settings.categories')}>
  {#each categories as item,index}<button id="settings-tab-{item.id}" type="button" role="tab" title={item.detail} aria-selected={category===item.id} aria-controls="settings-panel-{item.id}" tabindex={category===item.id?0:-1} onclick={()=>category=item.id} onkeydown={event=>move(event,index)}><strong>{item.label}</strong></button>{/each}
 </div></nav>
 <div class="settings-panels">
  {#each categories as item}<div class="settings-panel" id="settings-panel-{item.id}" role="tabpanel" aria-labelledby="settings-tab-{item.id}" tabindex="0" hidden={category!==item.id}>
   {#if item.id==='general'}{@render generalPanel()}{:else if item.id==='appearance'}{@render appearancePanel()}{:else if item.id==='provider'}{@render providerPanel()}{:else if item.id==='selection'}{@render selectionPanel()}{:else if archivePanel}{@render archivePanel()}{/if}
  </div>{/each}
 </div>
</div>
<style>
 .settings-hub{display:grid;grid-template-columns:182px minmax(0,1fr);gap:24px;max-width:980px;margin:0 auto;height:100%;min-height:0;color:var(--text);font-size:1rem}nav{padding-top:20px}.category-list{display:flex;flex-direction:column;gap:4px}button{border:1px solid transparent;border-radius:var(--radius-control);background:transparent;color:var(--muted);text-align:left;padding:9px 12px;font:inherit;cursor:pointer;transition:transform 140ms var(--ease-out),background-color 140ms ease,color 140ms ease,border-color 140ms ease}strong{display:block;font-size:.96rem;font-weight:500}@media(hover:hover) and (pointer:fine){button:hover{background:var(--surface-hover);color:var(--text)}}button:active:not(:disabled){transform:scale(0.97)}button[aria-selected=true]{background:var(--accent-soft);color:var(--text);border-color:transparent;font-weight:600;transform:none}button[aria-selected=true] strong{font-weight:600}button:focus-visible,.settings-panel:focus-visible{outline:2px solid var(--focus-ring);outline-offset:2px}.settings-panels{min-width:0;min-height:0}.settings-panel{height:100%;overflow:auto;overscroll-behavior:contain;scrollbar-gutter:stable;padding:0 8px;box-sizing:border-box}.settings-panel[hidden]{display:none}
 @media(max-width:760px){.settings-hub{grid-template-columns:minmax(0,1fr);grid-template-rows:auto minmax(0,1fr);gap:12px}nav{padding-top:10px;min-width:0}.category-list{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:6px;padding:2px}button{flex-shrink:0;padding:8px 12px}.settings-panel{padding:0 2px}}@media(prefers-reduced-motion:reduce){button{transition-property:background-color,color,opacity;transition-duration:120ms;transform:none}}
</style>
