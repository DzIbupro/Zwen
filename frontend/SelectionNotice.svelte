<script lang="ts">
  import type {SelectionState} from './lib/selection';
 import type {QuickAction} from './lib/quick-actions';
 import {t} from './lib/i18n.svelte';
 let {state:selState,onRetry,onDiscard,onAction,onSettings,actions=[],actionBusy=false}:{state:SelectionState;onRetry:()=>void;onDiscard:()=>void;onReload:()=>void;onAction?:(id:string)=>void;onSettings?:()=>void;actions?:QuickAction[];actionBusy?:boolean}=$props();
 const translateLabel=$derived(actions.find(a=>a.id==='translate')?.name||'翻译');
 const explainLabel=$derived(actions.find(a=>a.id==='explain')?.name||'详细解释');
 let noticeEl=$state<HTMLDivElement|undefined>(undefined);
 function handleOutsidePointer(event:PointerEvent){
  if(!selState.pending||selState.busy||actionBusy||!noticeEl)return;
  if(event.target instanceof Node&&!noticeEl.contains(event.target)){onDiscard();}
 }
</script>
<svelte:window onpointerdown={handleOutsidePointer}/>
{#if selState.busy||selState.notice||selState.pending}
 <div bind:this={noticeEl} class="selection-notice" role="status" data-selection-state>
  <span>{selState.busy?'正在处理选区…':selState.notice}</span>
  {#if selState.pending}
   <details><summary>查看待处理选区</summary><pre>{selState.pending.text||selState.pending.error}</pre></details>
   {#if selState.pending.text&&selState.pending.mode==='candidate'&&onAction}
    <div class="selection-actions" aria-label="划词操作" data-selection-menu>
     <button data-selection-action="ask" disabled={selState.busy||actionBusy} onclick={()=>onAction?.('ask')}>{t('selection.askZwen')}</button>
     <button data-selection-action="translate" disabled={selState.busy||actionBusy||!actions.some(a=>a.id==='translate'&&a.enabled)} onclick={()=>onAction?.('translate')}>{translateLabel}</button>
     <button data-selection-action="explain" disabled={selState.busy||actionBusy||!actions.some(a=>a.id==='explain'&&a.enabled)} onclick={()=>onAction?.('explain')}>{explainLabel}</button>
     <details class="other-actions"><summary>{t('selection.more')}</summary><div>
      {#each actions.filter(a=>a.enabled&&a.id!=='translate'&&a.id!=='explain') as action(action.id)}<button data-selection-action={action.id} disabled={selState.busy||actionBusy} onclick={()=>onAction?.(action.id)}>{action.name}{action.output==='document'?' · 保存文档':''}</button>{/each}
      <button data-selection-settings onclick={onSettings}>{t('selection.manage')}</button>
     </div></details>
    </div>
   {:else if selState.pending.text}<button disabled={selState.busy} onclick={onRetry}>{selState.inserted?'重试保存并确认':selState.pending.mode==='candidate'?'添加到草稿':'已检查草稿，重新添加'}</button>{/if}
  {/if}
 </div>
{/if}
<style>
 button:disabled{opacity:.5;cursor:default;}
 summary{display:block;width:auto;height:auto;letter-spacing:normal;font-size:13px;line-height:1.6;white-space:nowrap;cursor:pointer;}
 .selection-actions{display:flex;gap:5px;align-items:center;flex-wrap:wrap;min-width:0}.other-actions{display:inline-block}.other-actions summary{cursor:pointer;padding:5px}.other-actions div{display:flex;flex-wrap:wrap;gap:4px}.selection-notice{font-size:13px;padding:6px 12px;color:var(--text);overflow-wrap:anywhere;max-height:240px;overflow:auto;flex-shrink:0;}
 pre{white-space:pre-wrap;max-height:72px;overflow:auto;font:inherit;}
 button{flex-shrink:0;white-space:nowrap;line-height:1.6;font:inherit;margin:4px 6px 0 0;padding:3px 7px;border:1px solid var(--edge);border-radius:12px;background:var(--surface);color:var(--muted);cursor:pointer;}

 button{transition:transform 140ms var(--ease-out),box-shadow 160ms var(--ease-out),background-color 160ms ease;border-radius:12px;cursor:pointer;box-shadow:var(--raised-shadow)}
 @media(hover:hover) and (pointer:fine){button:hover:not(:disabled){box-shadow:var(--hover-shadow)}}
 button:active:not(:disabled){box-shadow:var(--pressed-shadow);transform:scale(0.96)}
 button:focus-visible,summary:focus-visible{outline:2px solid var(--focus-ring);outline-offset:3px}
 button:disabled{opacity:.5;cursor:default}
 @media(prefers-reduced-motion:reduce){button{transition-property:background-color,color,opacity;transition-duration:120ms;transform:none}}
 .selection-notice{font-size:14px;line-height:1.65;color:var(--text)}summary{font-size:14px}button{font-size:14px;background:var(--surface);color:var(--text);padding:5px 10px}.selection-actions{gap:8px}
</style>
