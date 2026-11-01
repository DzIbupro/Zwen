<script lang="ts">
 import {renderMarkdown} from './lib/render';
 import type {KnowledgeController,KnowledgeState} from './lib/knowledge';
 let {controller,info}:{controller:KnowledgeController;info:KnowledgeState}=$props();
 let armed: string|null = $state<string|null>(null);
 const dateText=(value:number)=>new Date(value*1000).toLocaleString('zh-CN',{dateStyle:'short',timeStyle:'short'});
 function onSearchInput(value:string){controller.refresh(value);void 0;}
</script>
<section class="knowledge" data-knowledge aria-label="知识库">
 {#if info.current}
  <div class="note-detail" data-note-detail>
   <div class="note-toolbar"><button onclick={()=>{armed=null;controller.close();}}>← 返回列表</button>{#if info.current.meta.source||info.current.meta.model}<span class="note-meta">{info.current.meta.source}{info.current.meta.model?` · ${info.current.meta.model}`:''}</span>{/if}<button data-note-delete onclick={()=>{if(armed===info.current?.meta.id){void controller.remove(info.current!.meta.id);armed=null;}else{armed=info.current!.meta.id;}}}>{armed===info.current?.meta.id?'确认删除':'删除'}</button></div>
   <h2>{info.current.meta.title}</h2>
   <div class="prose">{@html renderMarkdown(info.current.body)}</div>
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
</style>
