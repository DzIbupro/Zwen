<script lang="ts">
 import type {SelectionState} from './lib/selection';
 import type {QuickAction} from './lib/quick-actions';
 let {state,onRetry,onDiscard,onAction,onSettings,actions=[],actionBusy=false}:{state:SelectionState;onRetry:()=>void;onDiscard:()=>void;onReload:()=>void;onAction?:(id:string)=>void;onSettings?:()=>void;actions?:QuickAction[];actionBusy?:boolean}=$props();
</script>
{#if state.busy||state.notice||state.pending}
 <div class="selection-notice" role="status" data-selection-state>
  <button class="selection-close" aria-label="关闭选区提示" title="取消待处理选区" disabled={state.busy||actionBusy} onclick={onDiscard}>×</button>
  <span>{state.busy?'正在处理选区…':state.notice}</span>
  {#if state.pending}
   <details><summary>查看待处理选区</summary><pre>{state.pending.text||state.pending.error}</pre></details>
   {#if state.pending.text&&state.pending.mode==='candidate'&&onAction}
    <div class="selection-actions" aria-label="划词操作" data-selection-menu>
     <button data-selection-action="ask" disabled={state.busy||actionBusy} onclick={()=>onAction?.('ask')}>问一问 Zwen</button>
     <button data-selection-action="translate" disabled={state.busy||actionBusy||!actions.some(a=>a.id==='translate'&&a.enabled)} onclick={()=>onAction?.('translate')}>翻译</button>
     <button data-selection-action="explain" disabled={state.busy||actionBusy||!actions.some(a=>a.id==='explain'&&a.enabled)} onclick={()=>onAction?.('explain')}>详细解释</button>
     <details class="other-actions"><summary>其他</summary><div>
      {#each actions.filter(a=>a.enabled&&a.id!=='translate'&&a.id!=='explain') as action(action.id)}<button data-selection-action={action.id} disabled={state.busy||actionBusy} onclick={()=>onAction?.(action.id)}>{action.name}{action.output==='document'?' · 保存文档':''}</button>{/each}
      <button data-selection-settings onclick={onSettings}>管理快捷提示词…</button>
     </div></details>
    </div>
   {:else if state.pending.text}<button disabled={state.busy} onclick={onRetry}>{state.inserted?'重试保存并确认':state.pending.mode==='candidate'?'添加到草稿':'已检查草稿，重新添加'}</button>{/if}
  {/if}
 </div>
{/if}
<style>
 .selection-close{position:relative;float:right;margin:0 0 4px 10px;width:28px;height:28px;padding:0;font-size:20px;line-height:1;border-radius:50%}.selection-close::after{content:'';position:absolute;inset:-6px}
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
