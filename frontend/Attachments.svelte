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
