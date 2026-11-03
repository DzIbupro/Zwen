<script lang="ts">
 import {untrack} from 'svelte';
 import {renderMarkdown} from './lib/render';import {proseLinkClick} from './lib/links';import {openExternal} from './lib/bridge';
 import type {KnowledgeController,KnowledgeState,NoteMeta} from './lib/knowledge';
 let {controller,info,onTrace,onSaveEdit,modelLabel}:{controller:KnowledgeController;info:KnowledgeState;onTrace:(requestId:string)=>void;onSaveEdit:(id:string,title:string,body:string,source:{sourceTitle?:string|null;sourceUrl?:string|null;sourceLoc?:string|null})=>Promise<NoteMeta|null>;modelLabel:string}=$props();
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
   <div class="knowledge-bar"><button onclick={()=>controller.exitOrganize()}>← 返回列表</button><strong class="organize-title">AI 整理</strong></div>
   {#if info.confirming}
    <div class="organize-confirm" data-organize-confirm-view>
     <p><strong>确认整理</strong> — 本次将把 <b>{selectedCount} 篇</b>笔记的标题与正文发送给模型 <b>{modelLabel}</b> 进行主题归纳。</p>
     <ul class="confirm-list">{#each Object.keys(info.selected) as id}{#if info.notes.find(n=>n.id===id)}<li>{info.notes.find(n=>n.id===id)!.title}</li>{/if}{/each}</ul>
     <p class="organize-hint">整理结果带 AI 标记并可整批撤销；不会修改笔记原文。</p>
     <div class="editor-actions"><button disabled={info.organizeBusy} onclick={()=>controller.cancelConfirm()}>返回</button><button class="primary-edit" data-organize-go disabled={info.organizeBusy} onclick={()=>void controller.runOrganize(modelLabel)}>{info.organizeBusy?'正在整理…':'确认整理'}</button></div>
    </div>
   {:else if info.lastSummary}
    <div class="organize-result" data-organize-result>
     <p role="status">整理完成：新建主题 <b>{info.lastSummary.topics}</b> 个，归属笔记 <b>{info.lastSummary.assignments}</b> 篇次。</p>
     <div class="editor-actions"><button data-organize-undo disabled={info.organizeBusy} onclick={()=>void controller.undoLast()}>{info.organizeBusy?'正在撤销…':'撤销本次整理'}</button><button class="primary-edit" onclick={()=>{controller.exitOrganize();void controller.loadTopics();}}>完成</button></div>
    </div>
   {:else}
    <p class="organize-hint">勾选 1–8 篇笔记（它们将被发送给 {modelLabel} 归纳主题）：</p>
    <label class="auto-row"><input type="checkbox" data-auto-toggle checked={info.auto} onchange={e=>void controller.setAuto(e.currentTarget.checked)}/>自动整理新收藏（开启即同意将以后新收藏的标题与正文发送给当前模型 {modelLabel} 归纳主题；更换模型后需重新开启）</label>
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
  <div class="knowledge-bar">
   <input data-knowledge-search type="search" placeholder="搜索标题或正文…" value={info.query} oninput={e=>{void controller.refresh(e.currentTarget.value);}}/>
   <button data-export-all disabled={info.busy} onclick={()=>void controller.exportAll().then(path=>{if(path)exportMsg=`已导出全部笔记到 ${path}`;})}>导出全部</button>
   <button data-organize-start disabled={info.busy} onclick={()=>{controller.enterOrganize();void controller.loadTopics();}}>AI 整理</button>
   <button data-knowledge-rebuild disabled={info.busy} title="从 Markdown 文件重建索引（正文是权威）" onclick={()=>void controller.rebuild()}>重建索引</button>
  </div>
  {#if info.topics.length>0&&!info.query}
   <div class="topics-strip" data-topics-strip>{#each info.topics as topicView (topicView.topic.id)}<button data-topic-chip onclick={()=>void controller.openTopic(topicView.topic.id)}>{topicView.topic.name}（{topicView.count}）{topicView.topic.locked?' 🔒':''}</button>{/each}</div>
  {/if}
  {#if info.topicView}
   {@const lockedCurrent=info.topics.find(t=>t.topic.id===info.topicView?.id)?.topic.locked??false}
   <div class="topic-view" data-topic-view>
    <div class="note-toolbar"><strong>{info.topicView.title}</strong><button onclick={()=>controller.closeTopic()}>关闭</button></div>
    <ul class="note-list">{#each info.topicView.notes as note (note.meta.id)}<li><button onclick={()=>void controller.open(note.meta.id)}><strong>{note.meta.title}</strong><small>{dateText(note.meta.createdAt)}</small></button></li>{/each}</ul>
    <div class="gov" data-topic-gov>
     <div class="gov-row"><input data-gov-rename placeholder="重命名为…" bind:value={govName} maxlength="200"/><button disabled={info.busy||!govName.trim()} onclick={()=>void controller.renameTopic(info.topicView!.id,govName).then(()=>govName='')}>重命名</button></div>
     <div class="gov-row"><input data-gov-column placeholder="栏目（重新归类）" bind:value={govColumn} maxlength="80"/><button disabled={info.busy} onclick={()=>void controller.setTopicColumn(info.topicView!.id,govColumn).then(()=>govColumn='')}>改栏目</button></div>
     <div class="gov-row"><button data-gov-lock disabled={info.busy} onclick={()=>void controller.lockTopic(info.topicView!.id,!lockedCurrent)}>{lockedCurrent?'解锁主题':'锁定主题'}</button><button data-gov-delete disabled={info.busy} onclick={()=>{if(govArmed){void controller.deleteTopic(info.topicView!.id);govArmed=false;}else{govArmed=true;}}}>{govArmed?'确认删除主题':'删除主题'}</button><span class="gov-hint">锁定后 AI 整理不会写入该主题；删除主题保留笔记原文</span></div>
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
 .knowledge{max-width:760px;margin:22px auto;padding:0 18px 28px}
 .knowledge-bar{display:flex;gap:10px;margin-bottom:14px}
 .knowledge-bar input{flex:1;border:1px solid var(--edge);border-radius:12px;padding:10px 12px;background:var(--surface);font:inherit;font-size:13px;color:var(--text)}
 .knowledge-bar button,.note-toolbar button,.editor-actions button,.organize-confirm button,.organize-result button,.topics-strip button{border:1px solid var(--edge);background:transparent;color:var(--muted);border-radius:12px;padding:9px 13px;font:inherit;font-size:13px;cursor:pointer}
 .knowledge-notice{font-size:13px;color:var(--muted);background:var(--surface-soft);border-radius:12px;padding:7px 12px;margin-bottom:10px}
 .knowledge-empty{color:var(--muted);font-size:13px;line-height:1.9}
 .note-list{list-style:none;margin:0;padding:0;display:flex;flex-direction:column;gap:8px}
 .note-list button{width:100%;text-align:left;border:1px solid var(--edge);border-radius:16px;background:var(--surface);padding:13px 15px;cursor:pointer;font:inherit}
 .note-list button:hover{background:var(--surface);border-color:var(--edge)}
 .note-list strong{display:block;font-size:13px;color:var(--text);margin-bottom:4px}
 .note-list small{color:var(--muted);font-size:13px}
 .note-toolbar{display:flex;align-items:center;gap:12px;margin-bottom:14px}
 .note-meta{margin-right:auto;font-size:13px;color:var(--muted)}
 h2{font-size:22px;letter-spacing:-.5px;margin:0 0 14px}
 .note-editor{display:flex;flex-direction:column;gap:8px}.note-editor label{font-size:13px;color:var(--muted)}.note-editor input,.note-editor textarea{border:1px solid var(--edge);border-radius:12px;padding:10px 12px;background:var(--surface);font:inherit;font-size:13px;color:var(--text);width:100%;box-sizing:border-box}.note-editor textarea{resize:vertical;line-height:1.7;font-family:ui-monospace,SFMono-Regular,Menlo,monospace}.editor-actions{display:flex;gap:10px;justify-content:flex-end;align-items:center}.editor-actions .primary-edit,.organize-confirm .primary-edit{background:var(--accent);color:var(--on-accent);border-color:var(--accent)}
 .topics-strip{display:flex;gap:8px;flex-wrap:wrap;margin-bottom:14px}.topics-strip button{border-radius:14px;background:var(--surface-soft);border-color:var(--edge);color:var(--muted)}
 .organize-title{font-size:15px;color:var(--text)}
 .organize-hint{font-size:13px;color:var(--muted);line-height:1.8}
 .organize-row{display:flex;gap:10px;align-items:flex-start;cursor:pointer}
 .organize-confirm p,.organize-result p{font-size:13px;line-height:1.9;color:var(--text)}
 .confirm-list{margin:8px 0;padding-left:20px;font-size:13px;color:var(--muted);line-height:1.9}
 .organize-confirm,.organize-result{border:1px solid var(--edge);border-radius:16px;background:var(--surface);padding:15px 17px}
 .note-source{font-size:13px;color:var(--muted);margin:-6px 0 12px}.note-source a{color:var(--accent)}
 .topic-view{border:1px solid var(--edge);border-radius:16px;background:var(--surface);padding:13px 15px}
 .gov{margin-top:12px;display:flex;flex-direction:column;gap:6px;border-top:1px dashed var(--edge);padding-top:10px}.gov-row{display:flex;gap:8px;align-items:center;flex-wrap:wrap}.gov-row input{border:1px solid var(--edge);border-radius:12px;padding:7px 10px;font:inherit;font-size:13px;flex:1;min-width:140px;background:var(--surface);color:var(--text)}.gov-hint{font-size:13px;color:var(--muted)}.auto-row{display:flex;gap:8px;align-items:center;font-size:13px;color:var(--muted);margin-bottom:8px}

 button{transition:box-shadow .16s ease,background-color .16s ease;border-radius:12px;cursor:pointer;box-shadow:var(--raised-shadow)}
 button:hover:not(:disabled){box-shadow:var(--hover-shadow)}
 button:active:not(:disabled){box-shadow:var(--pressed-shadow)}
 button:focus-visible,input:focus-visible,textarea:focus-visible{outline:2px solid var(--focus-ring);outline-offset:3px}
 button:disabled{opacity:.5;cursor:default}
 @media(prefers-reduced-motion:reduce){button{transition:none}}
 .knowledge{font-size:16px;line-height:1.65;color:var(--text)}.knowledge-bar input,.note-editor input,.note-editor textarea,.gov-row input{font-size:16px;box-shadow:var(--inset-shadow)}.note-editor label{font-size:14px}.note-list strong{font-size:16px}.knowledge-empty,.organize-confirm p,.organize-result p,.confirm-list{font-size:16px;line-height:1.65}.note-toolbar{flex-wrap:wrap}.note-list button{box-shadow:none;background:transparent;border-color:var(--edge)}.topic-view{border:0;background:transparent;padding:0}.organize-confirm,.organize-result{background:var(--surface-soft);border:0}.organize-title{font-size:16px}.auto-row{font-size:14px}button,.knowledge-bar button,.note-toolbar button,.editor-actions button,.organize-confirm button,.organize-result button,.topics-strip button{font-size:14px;background:var(--surface);color:var(--text)}.editor-actions .primary-edit,.organize-confirm .primary-edit{background:var(--accent);color:var(--on-accent)}.knowledge-bar input{min-width:0}@media(max-width:520px){.knowledge{padding-inline:0}.knowledge-bar{flex-wrap:wrap}.knowledge-bar input{flex-basis:100%}.editor-actions{flex-wrap:wrap}}
</style>
