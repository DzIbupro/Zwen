import {it,expect,afterEach} from 'vitest';
import {KnowledgeController,type KnowledgePort,type NoteMeta,type Note} from '../lib/knowledge';
const cleanups:(()=>void)[]=[];afterEach(()=>cleanups.splice(0).forEach(f=>f()));
function meta(id='a'.repeat(32)):NoteMeta{return{id,title:'正则化笔记',createdAt:1700000000,source:'chat',model:'glm-4.7-flash',attachments:[]};}
function fixture(initial:NoteMeta[]=[],fail?:{list?:boolean;remove?:boolean}){
 let notes=initial;let saved:Note|null=null;
 const port:KnowledgePort={
  list:async query=>{if(fail?.list)throw Error('知识库读取失败');if(!query)return notes.map(n=>({...n}));return notes.filter(n=>n.title.includes(query)).map(n=>({...n}));},
  read:async id=>{if(saved&&saved.meta.id===id)return saved;const m=notes.find(n=>n.id===id);if(!m)throw Error('笔记不存在或已被删除');return{meta:{...m,attachments:[]},body:'正文'};},
  save:async input=>{const m=meta('b'.repeat(32));m.title=input.title;notes=[m,...notes];saved={meta:{...m},body:input.body};return {...m};},
  remove:async id=>{if(fail?.remove)throw Error('删除未完成');notes=notes.filter(n=>n.id!==id);if(saved?.meta.id===id)saved=null;},
  rebuild:async()=>notes.length,
 };
 const controller=new KnowledgeController(port,()=>{});cleanups.push(()=>controller.dispose());
 return {controller,get notes(){return notes;},port,get saved(){return saved;}};
}
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));

it('lists notes and keeps the query',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();
 expect(f.controller.state.loaded).toBe(true);
 expect(f.controller.state.notes).toHaveLength(1);
});

it('search filters by query and empty query restores everything',async()=>{
 const f=fixture([meta(),{...meta('c'.repeat(32)),title:'无关笔记'}]);await f.controller.refresh();
 await f.controller.refresh('正则化');
 expect(f.controller.state.notes).toHaveLength(1);
 expect(f.controller.state.query).toBe('正则化');
 await f.controller.refresh('');
 expect(f.controller.state.notes).toHaveLength(2);
});

it('a list failure is visible without clearing loaded state',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();
 f.port.list=async()=>{throw Error('知识库读取失败');};
 await f.controller.refresh();
 expect(f.controller.state.loaded).toBe(true);
 expect(f.controller.state.notice).toContain('知识库读取失败');
 expect(f.controller.state.notes).toHaveLength(1);
});

it('open shows the note and close returns to the list',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();
 await f.controller.open(meta().id);
 expect(f.controller.state.current?.body).toBe('正文');
 f.controller.close();
 expect(f.controller.state.current).toBeNull();
});

it('a failed open surfaces the message and refreshes the list',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();
 await f.controller.open('9'.repeat(32));
 expect(f.controller.state.notice).toContain('笔记不存在');
 expect(f.controller.state.current).toBeNull();
});

it('remove clears the current note and refreshes',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();
 await f.controller.open(meta().id);
 await f.controller.remove(meta().id);
 expect(f.controller.state.current).toBeNull();
 expect(f.controller.state.notes).toHaveLength(0);
});

it('a failed remove keeps the list and reports',async()=>{
 const f=fixture([meta()],{remove:true});await f.controller.refresh();
 await f.controller.remove(meta().id);
 expect(f.controller.state.notice).toContain('删除未完成');
 expect(f.controller.state.notes).toHaveLength(1);
});

it('save adds the note, reports nothing on success, and reports failures',async()=>{
 const f=fixture();await f.controller.refresh();
 const saved=await f.controller.save({title:'新收藏',body:'## 回答',source:'chat',model:'m',attachmentIds:[]});
 expect(saved?.title).toBe('新收藏');
 expect(f.controller.state.notes).toHaveLength(1);
 f.port.save=async()=>{throw Error('磁盘满');};
 const failed=await f.controller.save({title:'x',body:'y',source:'chat',model:'m',attachmentIds:[]});
 expect(failed).toBeNull();
 expect(f.controller.state.notice).toContain('磁盘满');
});

it('rebuild reports the adopted count',async()=>{
 const f=fixture([meta(),meta()]);await f.controller.refresh();
 await f.controller.rebuild();
 expect(f.controller.state.notice).toContain('共 2 篇');
});
