<script lang="ts">
 import type {RoundController,RoundState} from './lib/organize-round';
 import type {NoteMeta} from './lib/knowledge';
 import {getLocale} from './lib/i18n.svelte';
 let {
  controller,
  state,
  uncategorizedNotes,
  organizedCount,
  selected,
  onToggle,
  onSelectAll,
  onClearSelection,
  onDone
 }:{
  controller:RoundController;
  state:RoundState;
  uncategorizedNotes:NoteMeta[];
  organizedCount:number;
  selected:Record<string,boolean>;
  onToggle:(id:string)=>void;
  onSelectAll:(ids:string[])=>void;
  onClearSelection:()=>void;
  onDone:()=>void;
 }=$props();
 const view=$derived(state.view);
 const active=$derived(['running','stopping'].includes(view?.status??''));
 const remaining=$derived(view?Math.max(0,view.round.noteIds.length-view.round.completed):0);
 const progressPct=$derived(view&&view.round.noteIds.length>0?Math.round((view.round.completed/view.round.noteIds.length)*100):0);
 const hasPaused=$derived(Boolean(view&&!view.round.undone&&remaining>0&&['stopped','failed','interrupted'].includes(view.status)&&!state.dismissedPaused));
 const showJustCompleted=$derived(Boolean(view&&!view.round.undone&&view.status==='complete'&&state.justCompleted));
 const canUndoLast=$derived(Boolean(view&&!view.round.undone&&view.status==='complete'));
 const selectedIds=$derived(uncategorizedNotes.filter(note=>selected[note.id]===true).map(note=>note.id));
 const allSelected=$derived(uncategorizedNotes.length>0&&selectedIds.length===uncategorizedNotes.length);
 const dateText=(value:number)=>new Date(value*1000).toLocaleString(getLocale(),{dateStyle:'short',timeStyle:'short'});
</script>
{#if state.confirming&&view}
 <div class="round-card" data-organize-confirm-view>
  <p class="round-title">确认整理 <b>{remaining} 篇</b>未归类笔记</p>
  <p class="round-meta">使用模型：<b>{view.model}</b></p>
  <details class="hint-disclosure">
   <summary>整理说明</summary>
   <div class="hint-disclosure-body">
    <p>仅整理尚未归类的笔记，已经归类过的不会重复整理；整理过程不会修改笔记原文，完成后可随时撤销。模型接口：{view.baseUrl}</p>
   </div>
  </details>
  <div class="round-actions">
   <button disabled={state.busy} onclick={()=>controller.cancelConfirm()}>取消</button>
   <button class="primary" data-organize-go disabled={state.busy} onclick={()=>void controller.start()}>开始整理</button>
  </div>
 </div>
{:else if active&&view}
 <div class="round-card" data-organize-round-progress role="status">
  <p class="round-title">{view.status==='stopping'?'正在停止整理…':'正在整理未归类笔记…'}</p>
  <p class="round-meta">已完成 <b>{view.round.completed} / {view.round.noteIds.length}</b> 篇 · 模型 {view.model}</p>
  <div class="progress-track" aria-hidden="true"><div class="progress-fill" style:width={`${progressPct}%`}></div></div>
  <div class="round-actions">
   <button disabled={state.busy||view.status==='stopping'} onclick={()=>void controller.stop()}>停止整理</button>
  </div>
 </div>
{:else if hasPaused&&view}
 <div class="round-card" data-organize-round-progress role="status">
  <p class="round-title">上次整理未完成</p>
  <p class="round-meta">已归类 <b>{view.round.completed} / {view.round.noteIds.length}</b> 篇，剩余 <b>{remaining}</b> 篇 · 模型 {view.model}</p>
  {#if view.error}<p class="round-error" role="alert">{view.error}</p>{/if}
  <div class="round-actions">
   <button class="primary" disabled={state.busy} onclick={()=>controller.confirmRetry()}>继续整理剩余 {remaining} 篇</button>
   {#if view.round.completed>0}<button data-organize-undo disabled={state.busy} onclick={()=>void controller.undo()}>撤销已整理部分</button>{/if}
   <button disabled={state.busy} onclick={()=>controller.dismissPaused()}>忽略并重新选择</button>
  </div>
 </div>
{:else if showJustCompleted&&view}
 <div class="round-card" data-organize-round-progress data-organize-result role="status">
  <p class="round-title">整理完成</p>
  <p class="round-meta">本次已归类 <b>{view.round.completed}</b> 篇笔记。</p>
  <div class="round-actions">
   <button data-organize-undo disabled={state.busy} onclick={()=>void controller.undo()}>撤销本次整理</button>
   <button class="primary" disabled={state.busy} onclick={onDone}>完成</button>
  </div>
 </div>
{:else if uncategorizedNotes.length===0}
 <div class="round-card round-empty" data-organize-empty role="status">
  <p class="round-title">全部笔记均已归类</p>
  <p class="round-meta">当前共有 {organizedCount} 篇已归类笔记，暂无需要整理的新笔记。</p>
  <div class="round-actions">
   {#if canUndoLast}<button data-organize-undo disabled={state.busy} onclick={()=>void controller.undo()}>撤销上次整理</button>{/if}
   <button class="primary" onclick={onDone}>返回知识库</button>
  </div>
 </div>
{:else}
 <div class="round-hero">
  <div class="round-hero-info">
   <p class="round-title">待整理：<b>{uncategorizedNotes.length} 篇</b>未归类笔记</p>
   <p class="round-meta">{organizedCount>0?`已跳过 ${organizedCount} 篇已归类笔记，只整理未归类内容。`:'AI 会自动归纳主题并建立关联，不会修改笔记原文。'}</p>
  </div>
  <div class="round-hero-actions">
   <button class="primary" data-organize-select-all disabled={state.busy} onclick={()=>void controller.prepare(null)}>一键整理全部未归类（{uncategorizedNotes.length} 篇）</button>
   {#if canUndoLast}<button data-organize-undo disabled={state.busy} onclick={()=>void controller.undo()}>撤销上次整理</button>{/if}
  </div>
 </div>
 <div class="manual-section">
  <div class="manual-bar">
   <span class="round-meta">或手动勾选需要整理的笔记（已选 {selectedIds.length} / {uncategorizedNotes.length}）</span>
   <div class="manual-bar-actions">
    <button class="ghost-btn" disabled={state.busy} onclick={()=>allSelected?onClearSelection():onSelectAll(uncategorizedNotes.map(n=>n.id))}>{allSelected?'取消全选':'全选未归类'}</button>
    <button data-organize-confirm disabled={state.busy||selectedIds.length===0} onclick={()=>void controller.prepare(selectedIds)}>整理已选（{selectedIds.length} 篇）</button>
   </div>
  </div>
  <ul class="note-list organize-select">
   {#each uncategorizedNotes as note (note.id)}
    <li>
     <label class="organize-row">
      <input type="checkbox" data-select={note.id} checked={selected[note.id]===true} onchange={()=>onToggle(note.id)}/>
      <span class="note-row-text"><strong>{note.title}</strong><small>{dateText(note.createdAt)}</small></span>
     </label>
    </li>
   {/each}
  </ul>
 </div>
{/if}
{#if state.error}<p class="round-error" role="alert">{state.error}</p>{/if}
<style>
 .round-card,.round-hero{padding:16px;border:1px solid var(--edge);border-radius:var(--radius-card);background:var(--surface);margin-bottom:14px}
 .round-hero{display:flex;align-items:center;justify-content:space-between;gap:14px;flex-wrap:wrap}
 .round-hero-info{display:flex;flex-direction:column;gap:4px;min-width:200px;flex:1}
 .round-hero-actions{display:flex;align-items:center;gap:8px;flex-wrap:wrap}
 .round-title{margin:0;font-size:15px;font-weight:600;color:var(--text);line-height:1.45}
 .round-meta{margin:0;font-size:13px;color:var(--muted);line-height:1.55;font-variant-numeric:tabular-nums}
 .round-actions{display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin-top:14px}
 .round-empty{text-align:center;padding:28px 18px}
 .round-empty .round-actions{justify-content:center}
 .progress-track{height:6px;border-radius:999px;background:var(--surface-soft);border:1px solid var(--edge);overflow:hidden;margin-top:12px}
 .progress-fill{height:100%;background:var(--accent);border-radius:999px;transition:width 240ms var(--ease-out)}
 .manual-section{display:flex;flex-direction:column;gap:10px}
 .manual-bar{display:flex;align-items:center;justify-content:space-between;gap:10px;flex-wrap:wrap}
 .manual-bar-actions{display:flex;align-items:center;gap:8px;flex-wrap:wrap}
 button{font:inherit;font-size:13px;font-weight:500;min-height:34px;padding:6px 12px;border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface);color:var(--text);cursor:pointer;transition:background-color 140ms ease,border-color 140ms ease}
 @media(hover:hover) and (pointer:fine){button:hover:not(:disabled){background:var(--surface-hover);border-color:var(--edge-strong)}}
 button:disabled{opacity:.48;cursor:default}
 button:focus-visible,input:focus-visible{outline:2px solid var(--focus-ring);outline-offset:2px}
 button.primary{background:var(--accent);color:var(--on-accent);border-color:var(--accent)}
 @media(hover:hover) and (pointer:fine){button.primary:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}
 .ghost-btn{background:transparent;border-color:transparent;color:var(--muted)}
 @media(hover:hover) and (pointer:fine){.ghost-btn:hover:not(:disabled){background:var(--surface-hover);color:var(--text);border-color:var(--edge)}}
 .note-list{list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:6px;max-height:48vh;overflow-y:auto}
 .organize-row{display:flex;align-items:flex-start;gap:10px;padding:10px 12px;border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface);cursor:pointer;overflow-wrap:anywhere}
 @media(hover:hover) and (pointer:fine){.organize-row:hover{background:var(--surface-soft);border-color:var(--edge-strong)}}
 .note-row-text{display:flex;flex-direction:column;gap:2px}
 .note-row-text strong{font-size:13.5px;font-weight:600;color:var(--text);line-height:1.45}
 .note-row-text small{font-size:12px;color:var(--muted);font-variant-numeric:tabular-nums}
 input[type="checkbox"]{flex:none;margin-top:3px}
 .round-error{margin:10px 0 0;font-size:13px;color:var(--text);border-left:3px solid var(--muted);padding:6px 12px;background:var(--surface-soft);border-radius:0 var(--radius-control) var(--radius-control) 0}
</style>
