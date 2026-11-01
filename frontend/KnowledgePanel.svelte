<script lang="ts">
 import {renderMarkdown} from './lib/render';
 import type {KnowledgeController,KnowledgeState} from './lib/knowledge';
 let {controller,info,onTrace,onSaveEdit}:{controller:KnowledgeController;info:KnowledgeState;onTrace:(requestId:string)=>void;onSaveEdit:(id:string,title:string,body:string)=>Promise<void>}=$props();
 let armed: string|null = $state<string|null>(null);
 let editing: boolean = $state(false);
 let editTitle: string = $state('');
 let editBody: string = $state('');
 let editBusy: boolean = $state(false);
 const dateText=(value:number)=>new Date(value*1000).toLocaleString('zh-CN',{dateStyle:'short',timeStyle:'short'});
 function onSearchInput(value:string){controller.refresh(value);void 0;}
</script>
<section class="knowledge" data-knowledge aria-label="知识库">
 {#if info.current}
  <div class="note-detail" data-note-detail>
   <div class="note-toolbar"><button onclick={()=>{armed=null;editing=false;controller.close();}}>← 返回列表</button>{#if info.current.meta.source||info.current.meta.model}<span class="note-meta">{info.current.meta.source}{info.current.meta.model?` · ${info.current.meta.model}`:''}</span>{/if}<button data-note-trace disabled={!info.current.meta.request} title={info.current.meta.request?'跳转到原始问答':'本笔记没有来源链接'} onclick={()=>{if(info.current?.meta.request)onTrace(info.current.meta.request);}}>查看原始问答</button><button data-note-edit onclick={()=>{editTitle=info.current!.meta.title;editBody=info.current!.body;editing=true;armed=null;}}>编辑</button><button data-note-delete onclick={()=>{if(armed===info.current?.meta.id){void controller.remove(info.current!.meta.id);armed=null;}else{armed=info.current!.meta.id;}}}>{armed===info.current?.meta.id?'确认删除':'删除'}</button></div>
   {#if editing}
    <div class="note-editor" data-note-editor>
     <label for="note-title-input">标题</label><input id="note-title-input" data-note-title-input maxlength="200" value={editTitle} oninput={e=>editTitle=e.currentTarget.value}/>
     <label for="note-body-input">正文（Markdown）</label>
     <textarea id="note-body-input" data-note-body-input rows="12" value={editBody} oninput={e=>editBody=e.currentTarget.value}></textarea>
     <div class="editor-actions"><button disabled={editBusy} onclick={()=>{editing=false;}}>取消</button><button class="primary-edit" data-note-save-edit disabled={editBusy||!editTitle.trim()} onclick={()=>{editBusy=true;void onSaveEdit(info.current!.meta.id,editTitle,editBody).then(()=>{editBusy=false;editing=false;});}}>{editBusy?'正在保存…':'保存编辑'}</button></div>
    </div>
   {:else}
    <h2>{info.current.meta.title}</h2>
    <div class="prose">{@html renderMarkdown(info.current.body)}</div>
   {/if}
  </div>
 {:else}
  <div class="knowledge-bar">
   <input data-knowledge-search type="search" placeholder="搜索标题或正文…" value={info.query} oninput={e=>{void controller.refresh(e.currentTarget.value);}}/>
   <button data-knowledge-rebuild disabled={info.busy} title="从 Markdown 文件重建索引（正文是权威）" onclick={()=>void controller.rebuild()}>重建索引</button>
  </div>
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
</section>
<style>
 .knowledge{max-width:760px;margin:22px auto;padding:0 18px 28px}
 .knowledge-bar{display:flex;gap:10px;margin-bottom:14px}
 .knowledge-bar input{flex:1;border:1px solid #d7dece;border-radius:9px;padding:10px 12px;background:#fffef9;font:inherit;font-size:13px;color:#333d36}
 .knowledge-bar button,.note-toolbar button{border:1px solid #d5ddcb;background:transparent;color:#55634b;border-radius:8px;padding:9px 13px;font:inherit;font-size:12px;cursor:pointer}
 .knowledge-notice{font-size:12px;color:#55634b;background:#eff3e6;border-radius:7px;padding:7px 12px;margin-bottom:10px}
 .knowledge-empty{color:#697663;font-size:13px;line-height:1.9}
 .note-list{list-style:none;margin:0;padding:0;display:flex;flex-direction:column;gap:8px}
 .note-list button{width:100%;text-align:left;border:1px solid #dce2d2;border-radius:11px;background:#fbfcf6;padding:13px 15px;cursor:pointer;font:inherit}
 .note-list button:hover{background:#fffef7;border-color:#abc19f}
 .note-list strong{display:block;font-size:13px;color:#333d36;margin-bottom:4px}
 .note-list small{color:#8b967e;font-size:11px}
 .note-toolbar{display:flex;align-items:center;gap:12px;margin-bottom:14px}
 .note-meta{margin-right:auto;font-size:11px;color:#8b967e}
 h2{font-size:22px;letter-spacing:-.5px;margin:0 0 14px}
 .note-editor{display:flex;flex-direction:column;gap:8px}.note-editor label{font-size:11px;color:#697663}.note-editor input,.note-editor textarea{border:1px solid #d7dece;border-radius:9px;padding:10px 12px;background:#fffef9;font:inherit;font-size:13px;color:#333d36;width:100%;box-sizing:border-box}.note-editor textarea{resize:vertical;line-height:1.7;font-family:ui-monospace,SFMono-Regular,Menlo,monospace}.editor-actions{display:flex;gap:10px;justify-content:flex-end}.editor-actions .primary-edit{background:#3b7157;color:#f4f7ec;border-color:#3b7157}
</style>
