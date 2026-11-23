<script lang="ts">
 import {knowledgeColumns} from './lib/knowledge-columns';
 import OrganizeRoundPanel from './OrganizeRoundPanel.svelte';
 import {untrack} from 'svelte';
 import {renderMarkdown} from './lib/render';import {proseLinkClick} from './lib/links';import {openExternal} from './lib/bridge';
 import type {KnowledgeController,KnowledgeState,NoteMeta} from './lib/knowledge';
 let {controller,info,onTrace,onSaveEdit,modelLabel}:{controller:KnowledgeController;info:KnowledgeState;onTrace:(requestId:string)=>void;onSaveEdit:(id:string,title:string,body:string,source:{sourceTitle?:string|null;sourceUrl?:string|null;sourceLoc?:string|null})=>Promise<NoteMeta|null>;modelLabel:string}=$props();
 let selectedColumn=$state<string|null>(null);
 const columns=$derived(knowledgeColumns(info.topics));
 const columnTopics=$derived(columns.find(c=>c.name===selectedColumn)?.topics??[]);
 $effect(()=>{if(selectedColumn&&!columns.some(c=>c.name===selectedColumn))selectedColumn=null;});
 let armed: string|null = $state<string|null>(null);
 const initialDraft=untrack(()=>info.current?controller.getEditorDraft(info.current.meta.id):undefined);
 let editorId=untrack(()=>info.current?.meta.id);
 $effect(()=>{const id=info.current?.meta.id;if(id!==editorId){untrack(()=>{const draft=id?controller.getEditorDraft(id):undefined;editing=!!draft;editTitle=draft?.title??'';editBody=draft?.body??'';srcTitle=draft?.sourceTitle??'';srcUrl=draft?.sourceUrl??'';srcLoc=draft?.sourceLoc??'';editorId=id;});}});
 let editing: boolean = $state(!!initialDraft);
 $effect(()=>{const id=info.current?.meta.id;if(!info.busy&&!editBusy&&editing&&id&&!controller.getEditorDraft(id))editing=false;});
 let editTitle: string = $state(initialDraft?.title??'');
 let editBody: string = $state(initialDraft?.body??'');
 let editBusy: boolean = $state(false);
 let srcTitle: string = $state(initialDraft?.sourceTitle??'');
 let srcUrl: string = $state(initialDraft?.sourceUrl??'');
 let srcLoc: string = $state(initialDraft?.sourceLoc??'');
 let exportMsg: string = $state('');
 let govName: string = $state('');
 let govColumn: string = $state('');
 let govArmed: boolean = $state(false);
 function remember(){if(info.current)controller.rememberEditorDraft(info.current.meta.id,{title:editTitle,body:editBody,sourceTitle:srcTitle,sourceUrl:srcUrl,sourceLoc:srcLoc});}
 const dateText=(value:number)=>new Date(value*1000).toLocaleString('zh-CN',{dateStyle:'short',timeStyle:'short'});
 const selectedCount=$derived(Object.keys(info.selected).length);
</script>
<section class="knowledge" data-knowledge aria-label="知识库">
 {#if info.current}
  <div class="note-detail" data-note-detail>
   {#if info.notice}<div class="knowledge-notice" role="status">{info.notice}</div>{/if}
   <div class="note-toolbar"><button onclick={()=>{armed=null;editing=false;controller.close();}}>← 返回列表</button><button data-note-export disabled={info.busy} onclick={()=>void controller.exportNote(info.current!.meta.id).then(path=>{exportMsg=path?`已导出到 ${path}`:'';})}>{exportMsg?'已导出':'导出'}</button>{#if info.current.meta.source||info.current.meta.model}<span class="note-meta">{info.current.meta.source}{info.current.meta.model?` · ${info.current.meta.model}`:''}</span>{/if}<button data-note-trace disabled={!info.current.meta.request} title={info.current.meta.request?'跳转到原始问答':'本笔记没有来源链接'} onclick={()=>{if(info.current?.meta.request)onTrace(info.current.meta.request);}}>查看原始问答</button><button data-note-edit disabled={editing||info.busy} onclick={()=>{editTitle=info.current!.meta.title;editBody=info.current!.body;srcTitle=info.current!.meta.sourceTitle??'';srcUrl=info.current!.meta.sourceUrl??'';srcLoc=info.current!.meta.sourceLoc??'';editing=true;armed=null;remember();}}>编辑</button><button data-note-delete onclick={()=>{if(armed===info.current?.meta.id){void controller.remove(info.current!.meta.id);armed=null;}else{armed=info.current!.meta.id;}}}>{armed===info.current?.meta.id?'确认删除':'删除'}</button></div>
   {#if editing}
    <div class="note-editor" data-note-editor>
     <label for="note-title-input">标题</label><input id="note-title-input" data-note-title-input maxlength="200" value={editTitle} disabled={editBusy||info.busy} oninput={e=>{editTitle=e.currentTarget.value;remember();}}/>
     <label for="note-body-input">正文（Markdown）</label>
     <textarea id="note-body-input" data-note-body-input rows="12" value={editBody} disabled={editBusy||info.busy} oninput={e=>{editBody=e.currentTarget.value;remember();}}></textarea>
     <label for="note-source-title">来源标题（可选）</label><input id="note-source-title" data-note-source-title maxlength="200" value={srcTitle} disabled={editBusy||info.busy} oninput={e=>{srcTitle=e.currentTarget.value;remember();}}/>
     <label for="note-source-url">来源 URL（可选）</label><input id="note-source-url" data-note-source-url maxlength="2048" value={srcUrl} disabled={editBusy||info.busy} oninput={e=>{srcUrl=e.currentTarget.value;remember();}}/>
     <label for="note-source-loc">页码或位置（可选）</label><input id="note-source-loc" data-note-source-loc maxlength="200" value={srcLoc} disabled={editBusy||info.busy} oninput={e=>{srcLoc=e.currentTarget.value;remember();}}/>
     <div class="editor-actions"><button disabled={editBusy||info.busy} onclick={()=>{if(info.current)controller.discardEditorDraft(info.current.meta.id);editing=false;}}>取消</button><button class="primary-edit" data-note-save-edit disabled={editBusy||info.busy||!editTitle.trim()} onclick={()=>{const savingId=info.current!.meta.id;editBusy=true;void onSaveEdit(savingId,editTitle,editBody,{sourceTitle:srcTitle||null,sourceUrl:srcUrl||null,sourceLoc:srcLoc||null}).then(saved=>{if(saved&&info.current?.meta.id===savingId){editing=false;srcTitle='';srcUrl='';srcLoc='';}}).catch(()=>{}).finally(()=>{editBusy=false;});}}>{editBusy?'正在保存…':'保存编辑'}</button></div>
    </div>
   {:else}
    <h2>{info.current.meta.title}</h2>
    {#if info.current.meta.sourceTitle||info.current.meta.sourceUrl||info.current.meta.sourceLoc}
     <div class="note-source" data-note-source><span>来源：</span>{#if info.current.meta.sourceUrl}<a href={info.current.meta.sourceUrl} onclick={e=>{e.preventDefault();openExternal(info.current!.meta.sourceUrl!);}}>{info.current.meta.sourceTitle||info.current.meta.sourceUrl}</a>{:else if info.current.meta.sourceTitle}{info.current.meta.sourceTitle}{/if}{#if info.current.meta.sourceLoc}<span> · {info.current.meta.sourceLoc}</span>{/if}</div>
    {/if}
    <div class="prose" role="presentation" onclick={e=>proseLinkClick(e,openExternal)} onkeydown={e=>proseLinkClick(e as unknown as MouseEvent,openExternal)}>{@html renderMarkdown(info.current.body)}</div>
   {/if}
  </div>
 {:else if info.organizeMode}
  <div class="organize" data-organize-panel>
   <div class="knowledge-bar organize-toolbar"><button onclick={()=>controller.exitOrganize()}>← 返回列表</button><strong class="organize-title">AI 整理</strong></div>
   {#if controller.rounds}
    <label class="auto-row"><input type="checkbox" data-auto-toggle checked={info.auto} onchange={e=>void controller.setAuto(e.currentTarget.checked)}/>自动整理新收藏（开启即同意将以后新收藏的标题与正文及已有主题目录发送给当前模型 {modelLabel}；更换模型后需重新开启）</label>
    <OrganizeRoundPanel controller={controller.rounds} state={info.roundState??controller.rounds.state} notes={info.notes} selected={info.selected} onToggle={id=>controller.toggleSelect(id)}/>
   {:else if info.confirming}
    <div class="organize-confirm" data-organize-confirm-view>
     <p><strong>确认整理</strong> — 本次将把 <b>{selectedCount} 篇</b>笔记的标题与正文、已有主题目录发送给模型 <b>{modelLabel}</b> 进行主题归纳。</p>
     <ul class="confirm-list">{#each Object.keys(info.selected) as id}{#if info.notes.find(n=>n.id===id)}<li>{info.notes.find(n=>n.id===id)!.title}</li>{/if}{/each}</ul>
     <details class="hint-disclosure"><summary>整理规则说明</summary><div class="hint-disclosure-body"><p>整理结果带 AI 标记并可整批撤销；不会修改笔记原文。</p></div></details>
     <div class="editor-actions"><button disabled={info.organizeBusy} onclick={()=>controller.cancelConfirm()}>返回</button><button class="primary-edit" data-organize-go disabled={info.organizeBusy} onclick={()=>void controller.runOrganize(modelLabel)}>{info.organizeBusy?'正在整理…':'确认整理'}</button></div>
    </div>
   {:else if info.lastSummary}
    <div class="organize-result" data-organize-result>
     <p role="status">整理完成：新建主题 <b>{info.lastSummary.topics}</b> 个，归属笔记 <b>{info.lastSummary.assignments}</b> 篇次。</p>
     <div class="editor-actions"><button data-organize-undo disabled={info.organizeBusy} onclick={()=>void controller.undoLast()}>{info.organizeBusy?'正在撤销…':'撤销本次整理'}</button><button class="primary-edit" onclick={()=>{controller.exitOrganize();void controller.loadTopics();}}>完成</button></div>
    </div>
   {:else}
    <label class="auto-row" title={`开启即同意将以后新收藏的标题与正文及已有主题目录发送给当前模型 ${modelLabel} 归纳主题；更换模型后需重新开启`}><input type="checkbox" data-auto-toggle checked={info.auto} onchange={e=>void controller.setAuto(e.currentTarget.checked)}/>自动整理新收藏</label>
    <details class="hint-disclosure"><summary>手动与自动整理说明</summary><div class="hint-disclosure-body"><p>勾选 1–8 篇笔记后将发送给 {modelLabel} 归纳主题；开启自动整理后，新收藏的笔记也会自动归纳（更换模型后需重新开启）。</p></div></details>
    <ul class="note-list organize-select">
     {#each info.notes as note (note.id)}
      <li><label class="organize-row"><input type="checkbox" data-select={note.id} checked={info.selected[note.id]===true} onchange={()=>controller.toggleSelect(note.id)}/><span><strong>{note.title}</strong><small>{dateText(note.createdAt)}</small></span></label></li>
     {/each}
    </ul>
    <div class="editor-actions"><span class="organize-hint">已选 {selectedCount}/8</span><button disabled={selectedCount===0} onclick={()=>controller.startConfirm()} data-organize-confirm>发起整理</button></div>
   {/if}
   {#if info.notice}<div class="knowledge-notice" role="status">{info.notice}</div>{/if}
  </div>
 {:else}
  {#if exportMsg}<div class="knowledge-notice" role="status">{exportMsg}</div>{/if}
  <div class="knowledge-bar knowledge-functions" role="group" aria-label="知识库操作">
   <input aria-label="搜索知识笔记" data-knowledge-search type="search" placeholder="搜索标题或正文…" value={info.query} oninput={e=>{void controller.refresh(e.currentTarget.value);}}/>
   <button data-export-all disabled={info.busy} onclick={()=>void controller.exportAll().then(path=>{if(path)exportMsg=`已导出全部笔记到 ${path}`;})}>导出全部</button>
   <button data-organize-start disabled={info.busy} onclick={()=>{controller.enterOrganize();void controller.loadTopics();}}>AI 整理</button>
   <button data-knowledge-rebuild disabled={info.busy} title="从 Markdown 文件重建索引（正文是权威）" onclick={()=>void controller.rebuild()}>重建索引</button>
  </div>
  {#if info.topics.length>0&&!info.query}
   <div class="column-heading"><p class="topics-label">知识栏目</p></div>
   <div class="column-browser" data-column-browser>
    <div class="topics-strip column-strip" data-topics-strip role="group" aria-label="知识栏目">{#each columns as column (column.name)}<button data-column-chip aria-pressed={selectedColumn===column.name} class:column-selected={selectedColumn===column.name} onclick={()=>selectedColumn=selectedColumn===column.name?null:column.name}>{column.name}<small>{column.topics.length}</small></button>{/each}</div>
    {#if selectedColumn}<div class="column-topics" data-column-topics><div class="column-heading"><strong>{selectedColumn}</strong><button class="column-close" aria-label="收起栏目主题" onclick={()=>selectedColumn=null}>收起</button></div><div class="topics-strip" role="group" aria-label={`${selectedColumn}的主题`}>{#each columnTopics as topicView (topicView.topic.id)}<button data-topic-chip onclick={()=>void controller.openTopic(topicView.topic.id)}>{topicView.topic.name}（{topicView.count}）{topicView.topic.locked?' 🔒':''}</button>{/each}</div></div>{/if}
   </div>
  {/if}
  {#if info.topicView}
   {@const lockedCurrent=info.topics.find(t=>t.topic.id===info.topicView?.id)?.topic.locked??false}
   <div class="topic-view" data-topic-view>
    <div class="note-toolbar"><strong>{info.topicView.title}</strong><button onclick={()=>controller.closeTopic()}>关闭</button></div>
    <ul class="note-list">{#each info.topicView.notes as note (note.meta.id)}<li><button onclick={()=>void controller.open(note.meta.id)}><strong>{note.meta.title}</strong><small>{dateText(note.meta.createdAt)}</small></button></li>{/each}</ul>
    <div class="gov" data-topic-gov>
     <div class="gov-row"><input data-gov-rename placeholder="重命名为…" bind:value={govName} maxlength="200"/><button disabled={info.busy||!govName.trim()} onclick={()=>void controller.renameTopic(info.topicView!.id,govName).then(()=>govName='')}>重命名</button></div>
     <div class="gov-row"><input data-gov-column placeholder="栏目（重新归类）" bind:value={govColumn} maxlength="80"/><button disabled={info.busy} onclick={()=>void controller.setTopicColumn(info.topicView!.id,govColumn).then(()=>govColumn='')}>改栏目</button></div>
     <div class="gov-row"><button data-gov-lock disabled={info.busy} onclick={()=>void controller.lockTopic(info.topicView!.id,!lockedCurrent)}>{lockedCurrent?'解锁主题':'锁定主题'}</button><button data-gov-delete disabled={info.busy} onclick={()=>{if(govArmed){void controller.deleteTopic(info.topicView!.id);govArmed=false;}else{govArmed=true;}}}>{govArmed?'确认删除主题':'删除主题'}</button><details class="hint-disclosure inline-hint"><summary>说明</summary><div class="hint-disclosure-body"><p>锁定后 AI 整理不会写入该主题；删除主题仅移除分组，保留笔记原文。</p></div></details></div>
     {#each info.relations.filter(r=>r.fromTopic===info.topicView?.id||r.toTopic===info.topicView?.id) as rel (rel.id)}
      <div class="gov-row" data-relation={rel.id}><span class="gov-hint">{rel.reason}</span><button onclick={()=>void controller.removeRelation(rel.id)}>删关系</button></div>
     {/each}
    </div>
   </div>
  {:else}
   {#if info.notice}<div class="knowledge-notice" role="status">{info.notice}</div>{/if}
   {#if info.loaded&&info.notes.length===0}
    <p class="knowledge-empty">{info.query?'没有匹配的笔记。':'还没有收藏。在模型回答下点「收藏到知识库」，问答与截图就会保存在这里。'}</p>
   {:else}
    <ul class="note-list">
     {#each info.notes as note (note.id)}
      <li><button data-note-item onclick={()=>void controller.open(note.id)}><strong>{note.title}</strong><small>{dateText(note.createdAt)}{note.source?` · ${note.source}`:''}</small></button></li>
     {/each}
    </ul>
   {/if}
  {/if}
 {/if}
</section>
<style>
 .knowledge{max-width:var(--reading-width);margin:14px auto;padding:0 12px 28px;color:var(--text);font-size:14px;line-height:1.6}
 .knowledge-bar{display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin-bottom:14px}
 .knowledge-functions{padding:10px;border:1px solid var(--edge);border-radius:var(--radius-card);background:var(--surface);box-shadow:var(--raised-shadow);margin-bottom:18px}
 .knowledge-functions input{flex:1;min-width:160px;height:36px;border:1px solid var(--edge);border-radius:var(--radius-control);padding:6px 11px;background:var(--surface-soft);color:var(--text);font:inherit;font-size:13.5px}
 .knowledge-functions input:focus-visible{outline:2px solid var(--focus-ring);outline-offset:1px;background:var(--surface)}
 .knowledge-functions button{min-height:36px;padding:6px 12px;font-size:13px;font-weight:500;white-space:nowrap;display:inline-flex;align-items:center;justify-content:center}
 .knowledge-functions [data-organize-start]{background:var(--accent);color:var(--on-accent);border-color:var(--accent)}
 @media(hover:hover) and (pointer:fine){.knowledge-functions [data-organize-start]:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}
 .knowledge-notice{font-size:13px;color:var(--muted);background:var(--surface-soft);border:1px solid var(--edge);border-radius:var(--radius-control);padding:8px 12px;margin-bottom:12px}
 .knowledge-empty{color:var(--muted);font-size:13.5px;line-height:1.7;text-align:center;padding:42px 16px;border:1px dashed var(--edge);border-radius:var(--radius-card);background:var(--surface)}

 .column-heading{display:flex;align-items:center;justify-content:space-between;gap:10px;margin-bottom:6px}
 .topics-label{font-size:12px;font-weight:600;color:var(--muted);margin:0;letter-spacing:.02em}
 .column-browser{max-height:220px;overflow-y:auto;overscroll-behavior:contain;scrollbar-width:thin;padding:4px 0 12px;margin-bottom:16px;border-bottom:1px solid var(--edge)}
 .topics-strip{display:flex;gap:6px;flex-wrap:wrap;padding:4px 0}
 .topics-strip button{border-radius:999px;background:var(--surface);border:1px solid var(--edge);color:var(--text);font-size:12.5px;font-weight:500;padding:4px 11px;line-height:1.45}
 @media(hover:hover) and (pointer:fine){.topics-strip button:hover:not(:disabled){background:var(--surface-hover);border-color:var(--edge-strong)}}
 .column-strip button{display:inline-flex;align-items:center;gap:6px}
 .column-strip button small{font-size:11px;color:var(--muted);font-variant-numeric:tabular-nums}
 .column-strip .column-selected{background:var(--accent-soft);border-color:var(--accent);color:var(--text);font-weight:600}
 .column-topics{margin-top:8px;padding:10px 12px;border-radius:var(--radius-control);background:var(--surface-soft);border:1px solid var(--edge)}
 .column-topics strong{font-size:13px;font-weight:600}
 .column-close{border:0;background:transparent;color:var(--muted);padding:3px 7px;font-size:12px}

 .note-list{list-style:none;margin:0;padding:0;display:flex;flex-direction:column;gap:8px}
 .note-list button{width:100%;text-align:left;border:1px solid var(--edge);border-radius:var(--radius-card);background:var(--surface);padding:12px 15px;display:flex;flex-direction:column;gap:4px;transition:transform 140ms var(--ease-out),background-color 140ms ease,border-color 140ms ease}
 @media(hover:hover) and (pointer:fine){.note-list button:hover:not(:disabled){background:var(--surface-soft);border-color:var(--edge-strong)}}
 .note-list strong{display:block;font-size:14px;font-weight:600;color:var(--text);line-height:1.45}
 .note-list small{color:var(--muted);font-size:12px;font-variant-numeric:tabular-nums}

 .note-detail{display:flex;flex-direction:column;gap:14px}
 .note-toolbar{display:flex;align-items:center;gap:8px;flex-wrap:wrap;padding-bottom:12px;border-bottom:1px solid var(--edge)}
 .note-toolbar button{font-size:12.5px;padding:5px 10px}
 .note-meta{margin-right:auto;font-size:12px;color:var(--muted);font-variant-numeric:tabular-nums}
 h2{font-size:21px;font-weight:600;letter-spacing:-0.015em;margin:4px 0 4px}
 .note-source{font-size:12.5px;color:var(--muted);margin:-6px 0 6px}.note-source a{color:var(--accent)}

 .note-editor{display:flex;flex-direction:column;gap:8px;background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:16px}
 .note-editor label{font-size:12.5px;font-weight:500;color:var(--muted);margin-top:4px}
 .note-editor input,.note-editor textarea{border:1px solid var(--edge);border-radius:var(--radius-control);padding:8px 11px;background:var(--surface-soft);font:inherit;font-size:13.5px;color:var(--text);width:100%;box-sizing:border-box}
 .note-editor textarea{resize:vertical;line-height:1.65;font-family:var(--font-mono);font-size:13px}
 .editor-actions{display:flex;gap:8px;justify-content:flex-end;align-items:center;margin-top:8px;flex-wrap:wrap}
 .editor-actions .primary-edit,.organize-confirm .primary-edit{background:var(--accent);color:var(--on-accent);border-color:var(--accent)}

 .organize-toolbar{gap:10px;padding-bottom:12px;border-bottom:1px solid var(--edge);margin-bottom:14px}
 .organize-title{margin:0;font-size:16px;font-weight:600}
 .organize-hint{font-size:12.5px;color:var(--muted)}
 .organize-row{display:flex;gap:10px;align-items:flex-start;padding:10px 12px;border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface);cursor:pointer}
 .organize-confirm,.organize-result{border:1px solid var(--edge);border-radius:var(--radius-card);background:var(--surface);padding:16px}
 .organize-confirm p,.organize-result p{font-size:13.5px;line-height:1.65;margin:0 0 10px}
 .confirm-list{margin:8px 0;padding-left:20px;font-size:13px;color:var(--muted);line-height:1.7}
 .auto-row{display:flex;gap:8px;align-items:center;font-size:13.5px;font-weight:500;color:var(--text);margin-bottom:8px;cursor:pointer}

 .topic-view{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:16px}
 .gov{margin-top:14px;display:flex;flex-direction:column;gap:8px;border-top:1px solid var(--edge);padding-top:12px}
 .gov-row{display:flex;gap:8px;align-items:center;flex-wrap:wrap}
 .gov-row input{border:1px solid var(--edge);border-radius:var(--radius-control);padding:6px 10px;font:inherit;font-size:13px;flex:1;min-width:140px;background:var(--surface-soft);color:var(--text)}
 .gov-hint{font-size:12.5px;color:var(--muted)}
 .inline-hint{margin-top:0}

 @media(max-width:720px){.knowledge-functions input{flex-basis:100%;width:100%}}
 @media(max-width:520px){.knowledge{padding-inline:0}.knowledge-functions button{flex:1 1 auto}}
</style>
