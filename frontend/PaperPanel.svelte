<script lang="ts">
 import {onMount,untrack} from 'svelte';
 import type {ChatState,ChatReply} from './lib/chat';
 import {defaultReadingOptions,type PaperPort,type PaperMetadata,type ReadingOptions,type LegacyRun,type SessionInfo} from './lib/papers';
 import {renderMarkdown} from './lib/render';import {proseLinkClick} from './lib/links';import {openExternal} from './lib/bridge';
 let {port,chat,desktop,model,onStart,onFollowup,onAdopt,onStop,onSave,onExport}:{port:PaperPort;chat:ChatState;desktop:boolean;model:string;onStart:(id:string,o:ReadingOptions)=>Promise<boolean|undefined>;onFollowup:(q:string)=>Promise<boolean|undefined>;onAdopt:(r:ChatReply)=>void;onStop:()=>void;onSave:()=>Promise<void>;onExport:()=>Promise<string>}=$props();
 let items=$state<PaperMetadata[]>([]),selected=$state<PaperMetadata|null>(null),options=$state(defaultReadingOptions());
 let history=$state<ChatReply[]>([]),pages=$state<string[]>([]),conversation=$state<string|null>(null),starting=$state(false),busy=$state(false),notice=$state(''),question=$state('');
 let oldPapers=$state<PaperMetadata[]>([]);
 let legacy=$state<LegacyRun[]>([]),legacyText=$state(''),savedScope=$state<SessionInfo|null>(null);
 let requirementsReady=$state(false);
 let builtin=$state<ReadingOptions|null>(null);
 let lastTerminal='';let disposed=false;let selectionVersion=0;
 const showing=$derived(!!chat.reply&&chat.reply.conversationId===conversation);
 const answer=$derived(showing?renderMarkdown(chat.answer):'');
 const requirementsChanged=$derived(!!builtin&&(options.mode!==builtin.mode||options.startPage!==1||options.endPage!==selected?.pageCount||options.focus!==builtin.focus||options.outputRequirements!==builtin.outputRequirements||!!options.background.trim()));
 const labels={streaming:'生成中',complete:'已完成',stopped:'已停止',failed:'失败，已接收内容保留',interrupted:'上次中断',limited:'输出达到限制',filtered:'服务限制'};
 async function refresh(){try{items=await port.list();}catch(e){notice=String(e);}}
 async function select(p:PaperMetadata,restore=false){if(!restore&&(chat.running||chat.stopping||busy))return;const ticket=++selectionVersion;selected=p;requirementsReady=false;options=defaultReadingOptions();options.endPage=p.pageCount;history=[];pages=[];conversation=null;savedScope=null;question='';notice='';try{const [runs,text,defaults]=await Promise.all([port.sessions(p.id),port.source(p.id),port.defaults()]);if(disposed||ticket!==selectionVersion)return;builtin=defaults;options={...defaults,endPage:p.pageCount};requirementsReady=true;history=runs;pages=text;if(chat.reply&&runs.some(r=>r.conversationId===chat.reply?.conversationId))conversation=chat.reply.conversationId;}catch(e){if(ticket===selectionVersion)notice=String(e);}}
 async function choose(){busy=true;notice='正在本机提取文字，不会发送模型…';try{const p=await port.choose();if(disposed)return;busy=false;if(p){await refresh();await select(p);}else notice='';}catch(e){notice=String(e);}finally{busy=false;}}
 async function start(){if(!selected||!requirementsReady)return;starting=true;notice='';try{const ok=await onStart(selected.id,{...options});if(ok){conversation=chat.reply?.conversationId??null;if(conversation&&port.sessionInfo)savedScope=await port.sessionInfo(conversation);}}catch(e){notice=String(e);}finally{starting=false;}}
 async function follow(){const q=question.trim();if(!q||!showing)return;const ok=await onFollowup(q);if(ok)question='';}
 async function exportReport(){try{notice=await onExport();}catch(e){notice=String(e);}}
 $effect(()=>{const reply=chat.reply;if(reply&&reply.state!=='streaming'&&selected&&reply.conversationId===conversation){const key=reply.requestId+':'+reply.revision;if(key!==lastTerminal){lastTerminal=key;const id=selected.id;untrack(()=>void port.sessions(id).then(rows=>{if(!disposed&&selected?.id===id)history=rows;}).catch(e=>notice=String(e)));}}});
 onMount(()=>{void refresh();if(port.legacyPapers)void port.legacyPapers().then(rows=>oldPapers=rows).catch(e=>notice=String(e));const current=chat.reply;if(current?.conversationId.startsWith('paper-')&&port.sessionInfo)void port.sessionInfo(current.conversationId).then(async scope=>{if(disposed)return;await select(scope.source,true);if(disposed)return;conversation=current.conversationId;savedScope=scope;}).catch(e=>notice=String(e));if(port.legacy)void port.legacy().then(rows=>legacy=rows).catch(e=>notice=String(e));return()=>{disposed=true;selectionVersion++;};});
</script>
<section class="paper-panel" aria-label="论文阅读">
 <header><h1>论文阅读</h1><p class="description">上传一篇论文，从问题、方法到证据与边界，直接获得分析并继续追问。</p></header>
 <div class="paper-actions"><button disabled={!desktop||busy||chat.running||chat.stopping} onclick={()=>void choose()}>{busy?'正在导入…':'选择 PDF 并导入'}</button><button disabled={busy} onclick={()=>void refresh()}>刷新列表</button></div>
 <p class="description">导入仅保存在本机。明确开始后，才把所选文字、关注点及背景发送到配置的服务。当前读取文字，图表图像未读取，复杂公式需核对。</p>
 {#if legacy.length||oldPapers.length}<details><summary>旧版论文记录（只读，不继续旧分析流程）</summary>{#each oldPapers as p}<button onclick={()=>void port.legacyOpen?.(p.id).catch(e=>notice=String(e))}>打开旧原件：{p.name} · {p.pageCount} 页</button>{/each}{#each legacy as r}<button onclick={()=>void port.legacyReport?.(r.runId).then(text=>legacyText=text).catch(e=>notice=String(e))}>{r.name} · {r.state}</button>{/each}<div class="prose">{@html renderMarkdown(legacyText)}</div></details>{/if}
 {#if !desktop}<p role="status">浏览器仅预览，请在桌面应用中导入和分析论文。</p>{/if}
 {#if notice}<p role="status">{notice}</p>{/if}
 <div class="paper-list">{#each items as p}<button data-paper-id={p.id} class:active={selected?.id===p.id} disabled={busy||chat.running||chat.stopping} onclick={()=>void select(p)}><strong>{p.name}</strong><span>{p.pageCount} 页 · {p.readablePages} 页可提取文字</span></button>{/each}</div>
 {#if selected}
  <h2>{selected.name}</h2>
  {#if builtin}
   <div class="paper-defaults"><h3>已内置论文阅读要求</h3><p class="description">默认全文精读，按你提供的阅读要求分析问题、方法、证据与边界。无需填写，直接开始即可。</p><ol>{#each builtin.outputRequirements.split('\n').filter(line=>/^\d+\./.test(line)) as line}<li>{line.replace(/^\d+\.\s*/,'')}</li>{/each}</ol></div>
  {:else}<p role="status">正在读取内置阅读要求…</p>{/if}
  <details class="paper-customize"><summary>{requirementsChanged?'已调整阅读要求（可展开修改）':'调整阅读要求（可选）'}</summary>
  <div class="paper-fields">
   <label>阅读模式<select bind:value={options.mode} disabled={chat.running||!requirementsReady}><option value="deep">标准精读</option><option value="quick">速读</option><option value="reproduce">复现准备</option><option value="interactive">交互学习</option></select></label>
   <label>起始页<input type="number" min="1" max={selected.pageCount} bind:value={options.startPage} disabled={chat.running||!requirementsReady}></label>
   <label>结束页<input type="number" min={options.startPage} max={selected.pageCount} bind:value={options.endPage} disabled={chat.running||!requirementsReady}></label>
  </div>
  <label class="paper-input">关注问题（已内置，可修改）<textarea data-reading-focus bind:value={options.focus} maxlength="8192" rows="6" disabled={chat.running||!requirementsReady}></textarea></label>
  <label class="paper-input">输出要求（已内置，可修改）<textarea data-reading-output bind:value={options.outputRequirements} maxlength="8192" rows="8" disabled={chat.running||!requirementsReady}></textarea></label>
  <label class="paper-input">补充研究背景（可选）<textarea data-reading-background bind:value={options.background} maxlength="8192" rows="2" disabled={chat.running||!requirementsReady}></textarea></label>
  <button type="button" disabled={!requirementsReady||chat.running||!builtin} onclick={()=>{if(builtin&&selected)options={...builtin,endPage:selected.pageCount};}}>恢复内置要求</button>
  </details>
  <p class="description">PDF物理页 {options.startPage}–{options.endPage} / 共 {selected.pageCount} 页；通过全文文字发送给 {model}。每次追问重新附上本次选择的原文。超出请求上限会提示选页，不会静默截断。</p>
  <button class="primary-button" data-paper-start disabled={!desktop||!requirementsReady||busy||starting||chat.running||chat.stopping||!chat.loaded||(!!chat.reply&&!chat.reply.saved)} onclick={()=>void start()}>开始新分析</button>
  <details class="paper-source"><summary>核对本机提取原文与缺页</summary>{#each pages as text,i}<details><summary>PDF物理页 {i+1}{!text.trim()?' · 无可提取文字':''}</summary><pre>{text||'此页可能为扫描件或图表，文字路径无法读取。'}</pre></details>{/each}</details>
  {#if history.length}<details open><summary>本论文的分析会话</summary>{#each history as r}<button disabled={chat.running||chat.stopping||(!!chat.reply&&!chat.reply.saved)} onclick={()=>{conversation=r.conversationId;question='';onAdopt(r);if(port.sessionInfo)void port.sessionInfo(r.conversationId).then(scope=>{if(conversation===r.conversationId)savedScope=scope;}).catch(e=>notice=String(e));}}>{new Date(r.createdAt*1000).toLocaleString()} · {r.model} · {labels[r.state]}</button>{/each}</details>{/if}
 {/if}
 {#if starting&&!showing}<p role="status">准备原文与请求…</p>{/if}
 {#if showing&&chat.reply}
  <article class="paper-report"><h2>分析报告</h2><p>{chat.reply.model} · {labels[chat.reply.state]}</p>
   {#if savedScope}<details class="paper-scope"><summary>此会话追问的实际发送范围（开始分析时保存）</summary><p>{savedScope.source.name} · PDF物理页 {savedScope.options.startPage}–{savedScope.options.endPage||savedScope.source.pageCount} / {savedScope.source.pageCount}</p><p>服务：{savedScope.endpoint} · 模型：{savedScope.model}</p><p>模式：{savedScope.options.mode}；关注点：{savedScope.options.focus||'按内置关注问题分析'}</p><p>输出要求：{savedScope.options.outputRequirements||'按内置报告结构输出'}</p><p>研究背景：{savedScope.options.background||'未提供'}</p><p>每次追问重新附上上述原文及要求。上方设置仅用于开始新分析。最近一轮附带历史 {chat.reply.contextTurns} 轮；请求空间不足时优先原文，历史可降至零轮。</p></details>{/if}
   <div class="prose" role="presentation" onclick={e=>proseLinkClick(e,openExternal)} onkeydown={e=>proseLinkClick(e as unknown as MouseEvent,openExternal)}>{@html answer}</div>
   {#if chat.running}<button onclick={onStop} disabled={chat.stopping}>停止生成</button>{:else if chat.reply.answer&&chat.reply.saved}<div class="paper-actions"><button onclick={()=>void onSave()}>收藏到知识库</button><button onclick={()=>void exportReport()}>导出 Markdown</button></div>{/if}
   <label class="paper-input">围绕原文追问<textarea bind:value={question} maxlength="8192" rows="2" disabled={chat.running||chat.stopping}></textarea></label>
   <button disabled={!question.trim()||chat.running||chat.stopping||!chat.reply.saved} onclick={()=>void follow()}>发送追问</button>
  </article>
 {/if}
</section>
<style>
 .paper-panel{max-width:960px;margin:0 auto;padding:16px 24px}.paper-panel h1{font-size:1.65rem}.paper-actions,.paper-fields{display:flex;gap:12px;flex-wrap:wrap;align-items:center;margin:16px 0}.paper-list{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:10px;margin:20px 0}.paper-list button{text-align:left;padding:16px;border:1px solid var(--border,#ddd);border-radius:12px;display:grid;gap:8px}.paper-list button.active{outline:2px solid var(--accent,#536bd8)}.paper-list span,.description{font-size:.9rem;opacity:.75}.paper-fields label{display:grid;gap:7px}.paper-fields input{width:100px}.paper-input{display:grid;gap:7px;margin:14px 0}.paper-input textarea{min-height:60px;width:100%;box-sizing:border-box}.paper-panel input,.paper-panel select,.paper-input textarea{font:inherit;border:1px solid var(--border,#aaa);border-radius:8px;padding:10px;background:var(--surface,#fff);color:inherit}.paper-source{margin:20px 0}.paper-source pre{white-space:pre-wrap;max-height:360px;overflow:auto;font-size:.85rem}.paper-panel details{margin:12px 0}.paper-panel summary{cursor:pointer}.paper-panel details>button{display:block;margin:10px 0}.paper-report{border-top:1px solid var(--border,#ddd);margin-top:24px;padding-top:12px}
</style>
