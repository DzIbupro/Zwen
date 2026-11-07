<script lang="ts">
 import {onMount,untrack} from 'svelte';import {renderMarkdown} from './lib/render';import {proseLinkClick} from './lib/links';import {openExternal} from './lib/bridge';import {HistoryController,type HistoryPort,type HistoryState} from './lib/history';import type {ChatReply} from './lib/chat';
 let {port,canContinue,canDelete=true,onContinue,onDeleted,focusRequestId,onFocusConsumed}:{port:HistoryPort;canContinue:boolean;canDelete?:boolean;onDeleted?:(id:string)=>void;onContinue:(reply:ChatReply)=>void;focusRequestId?:string|null;onFocusConsumed?:()=>void}=$props();
 let info=$state<HistoryState>({items:[],selected:null,page:0,hasMore:false,loading:false,deleting:false,notice:null,error:null});
 const history=new HistoryController(untrack(()=>port),s=>info=s,id=>onDeleted?.(id));
 let confirming=$state(false);
 $effect(()=>{info.selected?.requestId;confirming=false;});
 const answer=$derived(info.selected?renderMarkdown(info.selected.answer):'');
 const labels:Record<ChatReply['state'],string>={streaming:'生成中',complete:'已完成',stopped:'已停止',failed:'生成失败',interrupted:'上次中断',limited:'已达限制',filtered:'服务限制'};
 onMount(()=>{void history.load(0);return()=>history.dispose();});
 $effect(()=>{const id=focusRequestId;if(!id)return;onFocusConsumed?.();untrack(()=>void history.open(id));});
</script>
<section class="history" aria-label="问答历史">
 <div class="history-head"><div><h1>问答历史</h1></div><button disabled={info.loading||info.deleting} onclick={()=>void history.load(0)}>刷新记录</button></div>
 <p class="description">问答保存在本机。按需查看，或继续同一服务配置下的会话。</p>
 {#if info.notice}<p role="status" class="description">{info.notice}</p>{/if}
 {#if info.error}<div class="history-error" role="alert">{info.error}</div>{/if}
 <div class="history-grid">
  <div><div class="records" aria-label="问答记录列表">{#each info.items as item (item.requestId)}<button class:selected={info.selected?.requestId===item.requestId} disabled={info.loading||info.deleting} onclick={()=>void history.open(item.requestId)}><strong title={item.title}>{item.title}</strong><small>{labels[item.state]} · {item.model}</small></button>{/each}{#if !info.items.length&&!info.loading}<p>还没有问答记录。</p>{/if}</div>
  <div class="pagination"><button disabled={info.page===0||info.loading||info.deleting} onclick={()=>void history.load(info.page-1)}>上一页</button><span>{info.page+1}</span><button disabled={!info.hasMore||info.loading||info.deleting} onclick={()=>void history.load(info.page+1)}>下一页</button></div></div>
  <div class="record-detail">{#if info.selected}<div class="record-meta"><span>{labels[info.selected.state]} · {new Date(info.selected.createdAt*1000).toLocaleString()}</span><button disabled={!canContinue||info.deleting||info.selected.state==='streaming'} onclick={()=>onContinue(info.selected!)}>继续此对话</button><button data-history-delete disabled={!canDelete||!port.remove||info.loading||info.deleting||!info.selected.saved||info.selected.state==='streaming'} onclick={()=>confirming=true}>删除此记录</button></div>{#if confirming}<div class="delete-confirm" role="alertdialog" aria-label="确认删除问答记录"><p>删除这条问答记录？删除后无法恢复，已收藏文档仍保留。</p><button disabled={info.deleting} onclick={()=>confirming=false}>取消</button><button data-history-confirm-delete disabled={!canDelete||info.deleting} onclick={()=>{confirming=false;void history.removeSelected();}}>确认删除</button></div>{/if}<h2>你问过</h2><p class="original-question">{info.selected.prompt}</p><div class="prose" role="presentation" onclick={e=>proseLinkClick(e,openExternal)} onkeydown={e=>proseLinkClick(e as unknown as MouseEvent,openExternal)}>{@html answer}</div>{#if info.selected.error}<p class="history-error">{info.selected.error}</p>{/if}{:else}<div class="empty-detail">{info.loading?'正在读取…':'选择左侧的一个问题。'}</div>{/if}</div>
 </div>
</section>
<style>
 .delete-confirm{padding:14px;margin-top:12px;border:1px solid var(--edge);border-radius:12px;background:var(--surface-soft)}.delete-confirm p{margin:0 0 10px}.delete-confirm button{margin-right:8px}
 .history{padding:24px 0}.history-head{display:flex;align-items:center;justify-content:space-between;gap:12px}h1{font-size:26px;font-weight:550;margin:10px 0}.description{font-size:13px;color:var(--muted)}.history-grid{display:grid;grid-template-columns:minmax(280px,38%) minmax(0,1fr);gap:25px;margin-top:25px}.records{display:flex;flex-direction:column;gap:12px;max-height:620px;overflow:auto;padding:8px}.records button{text-align:left;border:1px solid transparent;background:transparent;padding:16px;border-radius:16px;display:flex;flex-direction:column;gap:10px;min-width:0;min-height:116px}.records button.selected{background:var(--surface-soft);border-color:var(--edge)}.records strong{font-size:13px;font-weight:500;line-height:1.6;overflow:hidden;display:-webkit-box;-webkit-line-clamp:4;line-clamp:4;-webkit-box-orient:vertical;overflow-wrap:anywhere}.records small,.record-meta,.pagination{font-size:13px;color:var(--muted)}.record-detail{min-width:0}.record-meta{display:flex;align-items:center;justify-content:space-between;gap:8px}button{border:1px solid var(--edge);border-radius:12px;padding:7px 10px;color:var(--muted);background:transparent;font:inherit;font-size:13px}.pagination{display:flex;align-items:center;justify-content:space-between;margin-top:15px}h2{font-size:13px;color:var(--muted);font-weight:500;margin-top:22px}.original-question{white-space:pre-wrap;max-height:160px;overflow:auto;font-size:13px;line-height:1.8;border-bottom:1px solid var(--edge);padding-bottom:16px}.history-error{font-size:13px;color:var(--danger);line-height:1.7}.empty-detail{padding:40px 15px;color:var(--muted);font-size:13px;text-align:center}@media(max-width:720px){.history-grid{grid-template-columns:1fr}.records{max-height:360px}}

 button{transition:box-shadow .16s ease,background-color .16s ease;border-radius:12px;cursor:pointer;box-shadow:var(--raised-shadow)}
 button:hover:not(:disabled){box-shadow:var(--hover-shadow)}
 button:active:not(:disabled){box-shadow:var(--pressed-shadow)}
 button:focus-visible{outline:2px solid var(--focus-ring);outline-offset:3px}
 button:disabled{opacity:.5;cursor:default}
 @media(prefers-reduced-motion:reduce){button{transition:none}}
 .history{font-size:16px;line-height:1.65;color:var(--text)}h1{font-size:26px;margin:0}.description{font-size:14px}.records strong{font-size:16px;font-weight:600;line-height:1.65;white-space:normal}.records button.selected{box-shadow:var(--inset-shadow)}.original-question{font-size:16px;line-height:1.65}h2{font-size:14px}.record-meta{flex-wrap:wrap}.pagination{gap:8px}button{font-size:14px;background:var(--surface);color:var(--text)}.history-grid{gap:24px}@media(max-width:520px){.history-head{align-items:flex-start}.history-grid{margin-top:20px}}
</style>
