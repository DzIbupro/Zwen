<script lang="ts">
 import type {ActionOutputState} from './lib/action-output';
 let {state,onRetry,onDismiss}:{state:ActionOutputState|null;onRetry:()=>void;onDismiss:()=>void}=$props();
</script>
{#if state}
 <div class="action-output" role="status" data-action-output>
  {#if state.stage==='saved'}文档已保存：<span data-action-output-path>{state.path}</span>
  {:else if state.stage==='saving'}正在保存文档…
  {:else if state.stage==='error'}{state.error} <button data-action-output-retry onclick={onRetry}>重试保存文档</button>
  {:else}完整回答生成并保存后，将自动保存为文档。{/if}
  <button disabled={state.stage==='saving'} data-action-output-dismiss onclick={onDismiss}>{state.stage==='waiting'?'取消自动保存':'关闭提示'}</button>
 </div>
{/if}
<style>.action-output{padding:8px 12px;font-size:13px;line-height:1.6;overflow-wrap:anywhere;color:var(--muted);flex-shrink:0}button{font:inherit;border:1px solid var(--edge);background:transparent;border-radius:12px;color:inherit;margin:4px;padding:4px 7px;cursor:pointer}
 button{transition:box-shadow .16s ease,background-color .16s ease;border-radius:12px;cursor:pointer;box-shadow:var(--raised-shadow)}
 button:hover:not(:disabled){box-shadow:var(--hover-shadow)}
 button:active:not(:disabled){box-shadow:var(--pressed-shadow)}
 button:focus-visible{outline:2px solid var(--focus-ring);outline-offset:3px}
 button:disabled{opacity:.5;cursor:default}
 @media(prefers-reduced-motion:reduce){button{transition:none}}
 .action-output{font-size:14px;line-height:1.65;color:var(--muted)}button{font-size:14px;background:var(--surface);color:var(--text);padding:5px 10px}
</style>
