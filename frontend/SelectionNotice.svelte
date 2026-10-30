<script lang="ts">
 import type {SelectionState} from './lib/selection';
 let {state,onRetry,onDiscard,onReload}:{state:SelectionState;onRetry:()=>void;onDiscard:()=>void;onReload:()=>void}=$props();
</script>
{#if state.busy||state.notice||state.pending}
 <div class="selection-notice" role="status" data-selection-state>
  <span>{state.busy?'正在处理选区…':state.notice}</span>
  {#if state.pending}
   <details><summary>查看待处理选区</summary><pre>{state.pending.text||state.pending.error}</pre></details>
   {#if state.pending.text}<button disabled={state.busy} onclick={onRetry}>{state.inserted?'重试保存并确认':'已检查草稿，重新添加'}</button>{/if}
   <button disabled={state.busy} onclick={onDiscard}>丢弃待处理选区</button>
  {:else if !state.busy&&state.notice&&!state.notice.includes('未发送')}<button onclick={onReload}>重新读取状态</button>{/if}
 </div>
{/if}
<style>
 .selection-notice{font-size:12px;padding:6px 12px;color:var(--muted,#a9b5b2);overflow-wrap:anywhere;max-height:170px;overflow:auto;flex-shrink:0;}
 pre{white-space:pre-wrap;max-height:72px;overflow:auto;font:inherit;}
 button{font:inherit;margin:4px 6px 0 0;padding:3px 7px;border:1px solid #65756b66;border-radius:6px;background:transparent;color:inherit;cursor:pointer;}
</style>
