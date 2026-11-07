<script lang="ts">
 import type {RoundController,RoundState} from './lib/organize-round';
 import type {NoteMeta} from './lib/knowledge';
 let {controller,state,notes,selected,onToggle}:{controller:RoundController;state:RoundState;notes:NoteMeta[];selected:Record<string,boolean>;onToggle:(id:string)=>void}=$props();
 const active=$derived(['running','stopping'].includes(state.view?.status??''));
 const view=$derived(state.view);
 const remaining=$derived(view?view.round.noteIds.length-view.round.completed:0);
</script>
{#if state.confirming&&view}
 <div data-organize-confirm-view>
  <p>确认将 <b>{remaining} 篇</b>笔记的标题和正文发送至 <b>{view.baseUrl}</b> 的模型 <b>{view.model}</b>，分 {Math.ceil(remaining/8)} 批处理，每批最多 8 篇。每篇正文最多发送前 4000 字符，不发送图片附件。</p>
  <p>已完成批次保留；剩余部分可停止后重试，可撤销本轮全部已完成批次。模型配置在本轮固定，笔记原文不会修改。</p>
  <button disabled={state.busy} onclick={()=>controller.cancelConfirm()}>返回</button>
  <button data-organize-go disabled={state.busy} onclick={()=>void controller.start()}>确认整理</button>
 </div>
{:else if view&&!view.round.undone&&view.status!=='prepared'}
 <div data-organize-round-progress data-organize-result={view.status==='complete'?true:undefined} role="status">
  <p>本轮已处理 {view.round.completed}/{view.round.noteIds.length} 篇，已提交 {view.round.batchIds.length} 批。</p>
  <p>固定模型：{view.model} · {view.baseUrl}</p>
  {#if view.status==='complete'}<p>本轮整理完成。</p>
  {:else if active}<p>{view.status==='stopping'?'正在停止，已提交批次会保留。':'正在整理…'}</p>
  {:else}<p>本轮已暂停，剩余 {remaining} 篇可重试。{view.status==='interrupted'?'应用重启后不会自动继续发送。':''}</p>{/if}
  {#if view.error}<p role="alert">{view.error}</p>{/if}
  {#if active}<button disabled={state.busy||view.status==='stopping'} onclick={()=>void controller.stop()}>停止本轮</button>
  {:else}
   {#if remaining>0}<button disabled={state.busy} onclick={()=>controller.confirmRetry()}>确认重试剩余 {remaining} 篇</button>{/if}
   <button data-organize-undo disabled={state.busy} onclick={()=>void controller.undo()}>撤销本轮</button>
   {#if remaining===0}<button disabled={state.busy} onclick={()=>void controller.prepare(null)}>再次选择全部收藏</button>{/if}
  {/if}
 </div>
{:else}
 <p>勾选收藏后确认整理，或选择全库全部收藏。当前浏览列表最多显示 200 篇，全选范围不受列表或搜索过滤限制。</p>
 <button data-organize-select-all disabled={state.busy} onclick={()=>void controller.prepare(null)}>选择全部收藏并核对数量</button>
 <ul class="note-list organize-select">
  {#each notes as note (note.id)}<li><label class="organize-row"><input type="checkbox" data-select={note.id} checked={selected[note.id]===true} onchange={()=>onToggle(note.id)}/><span>{note.title}</span></label></li>{/each}
 </ul>
 <p>已选 {Object.keys(selected).length} 篇；每批最多 8 篇，按批处理。</p>
 <button data-organize-confirm disabled={state.busy||Object.keys(selected).length===0} onclick={()=>void controller.prepare(Object.keys(selected))}>核对所选范围与模型</button>
{/if}
{#if state.error}<p role="alert">{state.error}</p>{/if}
<style>
 p{line-height:1.65;overflow-wrap:anywhere;color:var(--text)}
 button{font:inherit;font-size:14px;min-height:40px;padding:8px 14px;margin:4px 8px 4px 0;border:1px solid var(--edge);border-radius:12px;background:var(--surface);color:var(--text);cursor:pointer}
 button:disabled{opacity:.5;cursor:default}button:focus-visible,input:focus-visible{outline:2px solid var(--accent);outline-offset:3px}
 [data-organize-confirm-view],[data-organize-round-progress]{padding:16px;border-radius:16px;background:var(--surface-soft)}
 [data-organize-go]{background:var(--accent);color:var(--on-accent)}
 .note-list{list-style:none;padding:0;margin:16px 0;display:flex;flex-direction:column;gap:8px;max-height:45vh;overflow-y:auto}
 .organize-row{display:flex;align-items:flex-start;gap:10px;padding:8px 4px;cursor:pointer;overflow-wrap:anywhere}
 input{flex:none;margin-top:5px}[role=alert]{color:var(--text);border-left:3px solid var(--muted);padding-left:12px}
</style>
