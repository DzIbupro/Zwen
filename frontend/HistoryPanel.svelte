<script lang="ts">
 import {onMount,untrack} from 'svelte';
 import BrandLogo from './BrandLogo.svelte';
 import Icon from './Icon.svelte';
 import {renderMarkdown} from './lib/render';import {proseLinkClick} from './lib/links';import {openExternal} from './lib/bridge';
 import {HistoryController,type HistoryPort,type HistoryState} from './lib/history';import type {ChatReply} from './lib/chat';
 let {port,canContinue,canDelete=true,onContinue,onDeleted,onBulkDeleted,currentRequestId=null,focusRequestId,onFocusConsumed}:{port:HistoryPort;canContinue:boolean;canDelete?:boolean;onDeleted?:(id:string)=>void;onBulkDeleted?:(ids:string[])=>void;currentRequestId?:string|null;onContinue:(reply:ChatReply)=>void;focusRequestId?:string|null;onFocusConsumed?:()=>void}=$props();
 let info=$state<HistoryState>({items:[],selected:null,page:0,hasMore:false,loading:false,deleting:false,checked:{},notice:null,error:null});
 let clearingReply:string|null=null;
 let query=$state('');
 const history=new HistoryController(untrack(()=>port),s=>info=s,id=>onDeleted?.(id),ids=>onBulkDeleted?.(ids??(clearingReply?[clearingReply]:[])));
 type DeleteRequest={kind:'single';id:string}|{kind:'selected';ids:string[]}|{kind:'all'};
 let deletion=$state<DeleteRequest|null>(null);
 const checkedCount=$derived(Object.keys(info.checked).length);
 const visibleItems=$derived(query.trim()?info.items.filter(r=>r.title.toLowerCase().includes(query.trim().toLowerCase())||r.model.toLowerCase().includes(query.trim().toLowerCase())):info.items);
 const pageChecked=$derived(info.items.some(r=>r.state!=='streaming')&&info.items.filter(r=>r.state!=='streaming').every(r=>info.checked[r.requestId]));
 const answer=$derived(info.selected?renderMarkdown(info.selected.answer):'');
 const labels:Record<ChatReply['state'],string>={streaming:'生成中',complete:'已完成',stopped:'已停止',failed:'生成失败',interrupted:'上次中断',limited:'已达限制',filtered:'服务限制'};
 const shortDate=(ts:number)=>new Date(ts*1000).toLocaleString('zh-CN',{month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit'});
 function open(id:string){deletion=null;void history.open(id);}
 function load(page:number){deletion=null;void history.load(page);}
 function confirmDelete(){
  const target=deletion;deletion=null;if(!target||!canDelete)return;
  if(target.kind==='all'){clearingReply=currentRequestId;void history.clearAll();}
  else if(target.kind==='selected')void history.removeMany(target.ids);
  else if(info.selected?.requestId===target.id)void history.removeSelected();
 }
 onMount(()=>{void history.load(0);return()=>history.dispose();});
 $effect(()=>{const id=focusRequestId;if(!id)return;deletion=null;onFocusConsumed?.();untrack(()=>void history.open(id));});
</script>

<section class="history" aria-label="问答历史">
 {#if info.notice}<div role="status" class="banner-notice">{info.notice}</div>{/if}
 {#if info.error}<div class="banner-error" role="alert">{info.error}</div>{/if}
 {#if deletion}
  <div class="delete-confirm" role="alertdialog" aria-labelledby="history-delete-title">
   <div class="delete-copy">
    <h2 id="history-delete-title">{deletion.kind==='all'?'清空全部历史？':deletion.kind==='selected'?`删除已勾选的 ${deletion.ids.length} 条记录？`:'删除这条问答记录？'}</h2>
    <p>{deletion.kind==='all'?'这会删除所有分页中的问答历史，无法恢复。':'确认删除后无法恢复。'}已收藏文档和草稿仍保留。</p>
   </div>
   <div class="delete-actions">
    <button disabled={info.deleting} onclick={()=>deletion=null}>取消</button>
    <button class="danger-solid" data-history-confirm-delete disabled={!canDelete||info.deleting} onclick={confirmDelete}>确认删除</button>
   </div>
  </div>
 {/if}

 <div class="history-split">
  <aside class="history-sidebar" aria-label="历史列表栏">
   <div class="sidebar-top">
    <div class="sidebar-head">
     <h1>问答历史</h1>
     <div class="head-ops">
      <button class="ghost-btn" title="刷新记录" disabled={info.loading||info.deleting} onclick={()=>load(0)}>刷新</button>
      <button class="ghost-btn danger-text" data-history-clear title="清空全部历史" disabled={!canDelete||!port.clear||info.loading||info.deleting||(!info.items.length&&info.page===0&&!info.hasMore)} onclick={()=>deletion={kind:'all'}}>清空</button>
     </div>
    </div>
    <input class="list-search" type="search" aria-label="筛选当前页历史" placeholder="筛选当前页标题或模型…" bind:value={query}/>
    <div class="batch-bar">
     <button class="mini-btn" data-history-check-page disabled={!canDelete||info.loading||info.deleting||!info.items.some(r=>r.state!=='streaming')} onclick={()=>history.togglePage()}>{pageChecked?'取消全选':'全选本页'}</button>
     {#if checkedCount}
      <button class="mini-btn" disabled={info.loading||info.deleting} onclick={()=>history.clearChecked()}>取消</button>
      <button class="mini-btn danger-pill" data-history-bulk disabled={!canDelete||!port.removeMany||info.loading||info.deleting} onclick={()=>deletion={kind:'selected',ids:Object.keys(info.checked)}}>删除 {checkedCount} 条</button>
     {:else}
      <button class="mini-btn sr-only" data-history-bulk disabled>批量删除</button>
     {/if}
    </div>
   </div>

   <div class="records" aria-label="问答记录列表">
    {#each visibleItems as item (item.requestId)}
     <div class="record-row" class:selected={info.selected?.requestId===item.requestId} class:checked={!!info.checked[item.requestId]}>
      <input type="checkbox" class="row-check" data-history-check={item.requestId} aria-label={`勾选：${item.title}`} checked={!!info.checked[item.requestId]} disabled={!canDelete||info.loading||info.deleting||item.state==='streaming'} onchange={()=>history.toggleChecked(item.requestId)}/>
      <button class="row-card" class:selected={info.selected?.requestId===item.requestId} disabled={info.loading||info.deleting} onclick={()=>open(item.requestId)}>
       <strong title={item.title}>{item.title}</strong>
       <div class="row-meta">
        <span class="state-dot" data-state={item.state} aria-hidden="true"></span>
        <span>{labels[item.state]}</span>
        <span class="dot-sep">·</span>
        <span class="model-tag">{item.model}</span>
        {#if item.createdAt}<span class="row-time">{shortDate(item.createdAt)}</span>{/if}
       </div>
      </button>
     </div>
    {/each}
    {#if !visibleItems.length&&!info.loading}
     <div class="list-empty">{query.trim()?'没有匹配的当前页记录':'还没有问答记录'}</div>
    {/if}
   </div>

   <div class="pagination">
    <button class="mini-btn" disabled={info.page===0||info.loading||info.deleting} onclick={()=>load(info.page-1)}>← 上一页</button>
    <span class="page-num">第 {info.page+1} 页</span>
    <button class="mini-btn" disabled={!info.hasMore||info.loading||info.deleting} onclick={()=>load(info.page+1)}>下一页 →</button>
   </div>
  </aside>

  <div class="record-detail">
   {#if info.selected}
    <header class="detail-header record-meta">
     <div class="detail-meta">
      <span class="status-pill" data-state={info.selected.state}>{labels[info.selected.state]}</span>
      <span class="detail-time">{new Date(info.selected.createdAt*1000).toLocaleString()}</span>
     </div>
     <div class="detail-actions">
      <button class="primary-continue" disabled={!canContinue||info.deleting||info.selected.state==='streaming'} onclick={()=>onContinue(info.selected!)}>继续此对话</button>
      <button class="ghost-btn danger-text" data-history-delete disabled={!canDelete||!port.remove||info.loading||info.deleting||!info.selected.saved||info.selected.state==='streaming'} onclick={()=>deletion={kind:'single',id:info.selected!.requestId}}>删除</button>
     </div>
    </header>

    <div class="detail-scroll">
     <article class="detail-thread">
      <div class="thread-question">
       <p class="original-question">{info.selected.prompt}</p>
      </div>
      <div class="thread-answer">
       <div class="answer-head">
        <BrandLogo compact/>
        <strong>{info.selected.model||'模型回复'}</strong>
       </div>
       <div class="prose" role="presentation" onclick={e=>proseLinkClick(e,openExternal)} onkeydown={e=>proseLinkClick(e as unknown as MouseEvent,openExternal)}>{@html answer}</div>
       {#if info.selected.error}<p class="banner-error">{info.selected.error}</p>{/if}
      </div>
     </article>
    </div>
   {:else}
    <div class="empty-detail">
     <div class="empty-icon"><Icon name="history" size={24}/></div>
     <p>{info.loading?'正在读取问答记录…':'从左侧选择一条问答记录查看详情'}</p>
    </div>
   {/if}
  </div>
 </div>
</section>

<style>
 .history{height:100%;min-height:0;display:flex;flex-direction:column;color:var(--text);font-size:14px}
 .banner-notice{flex-shrink:0;margin:0 0 10px;padding:8px 12px;border-radius:var(--radius-control);background:var(--surface-soft);border:1px solid var(--edge);font-size:13px;color:var(--muted)}
 .banner-error{flex-shrink:0;margin:0 0 10px;padding:8px 12px;border-radius:var(--radius-control);background:var(--danger-soft);border:1px solid color-mix(in srgb,var(--danger) 28%,transparent);font-size:13px;color:var(--danger)}
 .delete-confirm{flex-shrink:0;display:flex;align-items:center;justify-content:space-between;gap:14px;flex-wrap:wrap;padding:12px 14px;margin-bottom:12px;border:1px solid color-mix(in srgb,var(--danger) 35%,var(--edge));border-radius:var(--radius-card);background:var(--danger-soft)}
 .delete-copy h2{margin:0 0 4px;font-size:14px;font-weight:600;color:var(--text)}
 .delete-copy p{margin:0;font-size:12.5px;color:var(--muted)}
 .delete-actions{display:flex;gap:8px;align-items:center}
 .danger-solid{background:var(--danger);color:#ffffff;border-color:var(--danger);font-weight:500}
 @media(hover:hover) and (pointer:fine){.danger-solid:hover:not(:disabled){background:var(--danger);opacity:.9;color:#ffffff}}

 .history-split{flex:1;min-height:0;display:grid;grid-template-columns:minmax(260px,310px) minmax(0,1fr);border:1px solid var(--edge);border-radius:var(--radius-card);background:var(--surface);overflow:hidden}
 .history-sidebar{display:flex;flex-direction:column;min-height:0;min-width:0;background:var(--sidebar-bg);border-right:1px solid var(--edge)}
 .sidebar-top{flex-shrink:0;padding:12px 12px 10px;border-bottom:1px solid var(--edge);display:flex;flex-direction:column;gap:8px}
 .sidebar-head{display:flex;align-items:center;justify-content:space-between;gap:8px}
 .sidebar-head h1{margin:0;font-size:15px;font-weight:600;letter-spacing:-0.01em}
 .head-ops{display:flex;align-items:center;gap:2px}
 .list-search{width:100%;padding:6px 10px;border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface);color:var(--text);font:inherit;font-size:12.5px}
 .list-search:focus-visible{outline:2px solid var(--focus-ring);outline-offset:1px}
 .batch-bar{display:flex;align-items:center;gap:6px;flex-wrap:wrap}

 .records{flex:1;min-height:0;overflow-y:auto;padding:8px;display:flex;flex-direction:column;gap:4px;scrollbar-width:thin}
 .record-row{position:relative;display:flex;align-items:flex-start;gap:6px;padding:2px 4px 2px 6px;border-radius:var(--radius-control);border:1px solid transparent;transition:background-color 140ms ease,border-color 140ms ease}
 @media(hover:hover) and (pointer:fine){.record-row:hover{background:var(--surface-hover)}.record-row:hover .row-check{opacity:.9}}
 .record-row.selected{background:var(--surface);border-color:var(--edge-strong);box-shadow:var(--raised-shadow)}
 .record-row.checked .row-check,.record-row.selected .row-check{opacity:1}
 .row-check{margin:11px 0 0;width:14px;height:14px;flex-shrink:0;accent-color:var(--accent);opacity:.28;cursor:pointer;transition:opacity 140ms ease}
 .row-check:focus-visible{opacity:1}
 .row-card{flex:1;min-width:0;border:0;background:transparent;box-shadow:none;padding:8px 6px;text-align:left;display:flex;flex-direction:column;gap:5px;border-radius:6px}
 @media(hover:hover) and (pointer:fine){.row-card:hover:not(:disabled){background:transparent;border-color:transparent}}
 .row-card strong{font-size:13px;font-weight:500;line-height:1.45;color:var(--text);display:-webkit-box;-webkit-line-clamp:2;line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;overflow-wrap:anywhere}
 .record-row.selected .row-card strong{font-weight:600}
 .row-meta{display:flex;align-items:center;gap:5px;min-width:0;font-size:11.5px;color:var(--muted);font-variant-numeric:tabular-nums}
 .state-dot{width:6px;height:6px;border-radius:50%;background:var(--muted);flex-shrink:0}
 .state-dot[data-state="complete"]{background:#22c55e}
 .state-dot[data-state="failed"],.state-dot[data-state="interrupted"]{background:var(--danger)}
 .state-dot[data-state="streaming"]{background:var(--accent)}
 .dot-sep{opacity:.5}
 .model-tag{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;min-width:0;flex:1}
 .row-time{flex-shrink:0;opacity:.8}
 .list-empty{padding:36px 12px;text-align:center;font-size:13px;color:var(--muted)}

 .pagination{flex-shrink:0;display:flex;align-items:center;justify-content:space-between;gap:8px;padding:8px 10px;border-top:1px solid var(--edge);background:var(--sidebar-bg);font-size:12px;color:var(--muted);font-variant-numeric:tabular-nums}
 .page-num{font-weight:500}

 .record-detail{display:flex;flex-direction:column;min-height:0;min-width:0;background:var(--bg)}
 .detail-header{flex-shrink:0;display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap;padding:11px 18px;border-bottom:1px solid var(--edge);background:var(--surface)}
 .detail-meta{display:flex;align-items:center;gap:10px;min-width:0;flex-wrap:wrap;font-size:12.5px;color:var(--muted);font-variant-numeric:tabular-nums}
 .status-pill{display:inline-flex;align-items:center;padding:2px 8px;border-radius:999px;font-size:11.5px;font-weight:600;background:var(--surface-soft);color:var(--text);border:1px solid var(--edge)}
 .status-pill[data-state="failed"],.status-pill[data-state="interrupted"]{background:var(--danger-soft);color:var(--danger);border-color:transparent}
 .detail-actions{display:flex;align-items:center;gap:8px}
 .primary-continue{background:var(--accent);color:var(--on-accent);border-color:var(--accent);font-size:12.5px;font-weight:500;padding:5px 12px}
 @media(hover:hover) and (pointer:fine){.primary-continue:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}

 .detail-scroll{flex:1;min-height:0;overflow-y:auto;padding:20px 24px 28px;scrollbar-width:thin}
 .detail-thread{max-width:var(--reading-width);margin:0 auto;display:flex;flex-direction:column;gap:22px}
 .thread-question{display:flex;justify-content:flex-end}
 .original-question{max-width:84%;margin:0;padding:11px 15px;border-radius:var(--radius-card);background:var(--surface-soft);border:1px solid var(--edge);white-space:pre-wrap;overflow-wrap:anywhere;text-wrap:pretty;font-size:var(--prose-size);line-height:1.65;color:var(--text)}
 .thread-answer{display:flex;flex-direction:column;gap:10px;min-width:0}
 .answer-head{display:flex;align-items:center;gap:8px;font-size:13px;color:var(--text)}
 .answer-head strong{font-weight:600}

 .empty-detail{flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:10px;padding:32px;color:var(--muted);text-align:center}
 .empty-icon{width:44px;height:44px;border-radius:12px;background:var(--surface-soft);border:1px solid var(--edge);display:grid;place-items:center;color:var(--muted)}
 .empty-detail p{margin:0;font-size:13.5px}

 .ghost-btn{border:0;background:transparent;color:var(--muted);padding:5px 8px;font-size:12.5px;font-weight:500;border-radius:6px}
 @media(hover:hover) and (pointer:fine){.ghost-btn:hover:not(:disabled){background:var(--surface-hover);color:var(--text)}}
 .danger-text{color:var(--muted)}
 @media(hover:hover) and (pointer:fine){.danger-text:hover:not(:disabled){color:var(--danger);background:var(--danger-soft)}}
 .mini-btn{padding:4px 9px;font-size:12px;font-weight:500;border-radius:6px;background:var(--surface);border:1px solid var(--edge);color:var(--text)}
 .danger-pill{color:var(--danger);background:var(--danger-soft);border-color:color-mix(in srgb,var(--danger) 30%,transparent)}

 @media(max-width:760px){
  .history-split{grid-template-columns:1fr;grid-template-rows:minmax(220px,42%) minmax(0,1fr)}
  .history-sidebar{border-right:0;border-bottom:1px solid var(--edge)}
  .detail-scroll{padding:16px}
 }
</style>
