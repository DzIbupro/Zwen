<script lang="ts">
 import type {AttachmentState} from './lib/attachments';
 let {state,onCancel,onRemove,onRetake,disabled=false}:{
  state:AttachmentState;onCancel:()=>void;onRemove:(id:string)=>void;onRetake:(id:string)=>void;disabled?:boolean;
 }=$props();
</script>
{#if state.loaded&&state.notice&&!state.pending}
 <div class="attachment-note" role="status">{state.notice}</div>
{/if}
{#if state.loaded&&(state.items.length>0||state.pending)}
 <div class="attachments" data-attachments>
  {#if state.pending}
   <div class="attachment-pending" role="status"><span>正在等待截图…</span><button disabled={disabled} onclick={onCancel}>取消等待</button></div>
  {/if}
  {#each state.items as item (item.id)}
   <figure class="attachment" data-attachment={item.id}>
    {#if item.thumbnail}<img src={item.thumbnail} alt="截图附件预览"/>{:else}<div class="attachment-missing" title={item.error??''}>图片不可用</div>{/if}
    <div class="attachment-size">{item.width}×{item.height}</div>
    <div class="attachment-actions"><button disabled={disabled||state.pending} onclick={()=>onRetake(item.id)}>重截</button><button disabled={disabled||state.pending} onclick={()=>onRemove(item.id)}>删除</button></div>
   </figure>
  {/each}
 </div>
{/if}

<style>
 .attachments{display:flex;flex-wrap:wrap;gap:14px;padding:12px;flex-shrink:0;max-height:180px;overflow:auto;scrollbar-width:thin}.attachment{margin:0;position:relative;background:var(--surface-soft);border:1px solid var(--edge);border-radius:16px;padding:8px;max-width:180px;box-shadow:var(--inset-shadow)}.attachment img{display:block;width:148px;height:92px;object-fit:cover;border-radius:12px}.attachment-size{font-size:13px;line-height:1.65;color:var(--muted);margin-top:6px}.attachment-actions{display:flex;gap:8px;margin-top:6px}.attachment-note,.attachment-pending{font-size:14px;line-height:1.65;color:var(--muted);overflow-wrap:anywhere}.attachment-note{padding:8px 12px}.attachment-pending{display:flex;gap:12px;align-items:center;flex-wrap:wrap}.attachment-missing{padding:20px 12px;color:var(--muted);font-size:14px}button{font:inherit;font-size:14px;line-height:1.65;padding:5px 10px;border:1px solid var(--edge);background:var(--surface);color:var(--text);border-radius:12px;cursor:pointer;box-shadow:var(--raised-shadow)}button:hover:not(:disabled){box-shadow:var(--hover-shadow)}button:active:not(:disabled){box-shadow:var(--pressed-shadow)}button:focus-visible{outline:2px solid var(--focus-ring);outline-offset:3px}button:disabled{opacity:.5;cursor:default}
</style>
