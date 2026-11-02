<script lang="ts">
 import type {SelectionState} from './lib/selection';
 import type {QuickAction} from './lib/quick-actions';
 let {state,onRetry,onDiscard,onReload,onAction,onSettings,actions=[],actionBusy=false}:{state:SelectionState;onRetry:()=>void;onDiscard:()=>void;onReload:()=>void;onAction?:(id:string)=>void;onSettings?:()=>void;actions?:QuickAction[];actionBusy?:boolean}=$props();
</script>
{#if state.busy||state.notice||state.pending}
 <div class="selection-notice" role="status" data-selection-state>
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
   <button disabled={state.busy||actionBusy} onclick={onDiscard}>丢弃待处理选区</button>
  {:else if !state.busy&&state.notice&&!state.notice.includes('未发送')}<button onclick={onReload}>重新读取状态</button>{/if}
 </div>
{/if}
<style>
 .selection-actions{display:flex;gap:5px;align-items:center;flex-wrap:wrap}.other-actions{display:inline-block}.other-actions summary{cursor:pointer;padding:5px}.other-actions div{display:flex;flex-wrap:wrap;gap:4px}.selection-notice{font-size:12px;padding:6px 12px;color:var(--muted,#a9b5b2);overflow-wrap:anywhere;max-height:240px;overflow:auto;flex-shrink:0;}
 pre{white-space:pre-wrap;max-height:72px;overflow:auto;font:inherit;}
 button{font:inherit;margin:4px 6px 0 0;padding:3px 7px;border:1px solid #65756b66;border-radius:6px;background:transparent;color:inherit;cursor:pointer;}
</style>
