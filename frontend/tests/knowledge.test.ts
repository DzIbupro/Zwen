import {it,expect,afterEach} from 'vitest';
import {KnowledgeController,type KnowledgePort,type NoteMeta,type Note} from '../lib/knowledge';
const cleanups:(()=>void)[]=[];afterEach(()=>cleanups.splice(0).forEach(f=>f()));
function meta(id='a'.repeat(32)):NoteMeta{return{id,title:'正则化笔记',createdAt:1700000000,source:'chat',model:'glm-4.7-flash',attachments:[]};}
function fixture(initial:NoteMeta[]=[],fail?:{list?:boolean;remove?:boolean}){
 let notes=initial;let saved:Note|null=null;const edits:Record<string,string>={};let autoFlag=false;
 const port:KnowledgePort={
  list:async query=>{if(fail?.list)throw Error('知识库读取失败');if(!query)return notes.map(n=>({...n}));return notes.filter(n=>n.title.includes(query)).map(n=>({...n}));},
  read:async id=>{if(saved&&saved.meta.id===id)return saved;const m=notes.find(n=>n.id===id);if(!m)throw Error('笔记不存在或已被删除');return{meta:{...m,attachments:[]},body:edits[id]??'正文'};},
  save:async input=>{const m=meta('b'.repeat(32));m.title=input.title;notes=[m,...notes];saved={meta:{...m},body:input.body};return {...m};},
  remove:async id=>{if(fail?.remove)throw Error('删除未完成');notes=notes.filter(n=>n.id!==id);if(saved?.meta.id===id)saved=null;},
  update:async(id,title,body,source)=>{edits[id]=body;if(saved?.meta.id===id)saved={...saved,body};const m=notes.find(n=>n.id===id);if(!m)throw Error('笔记不存在或已被删除');m.title=title;m.sourceTitle=source?.sourceTitle??null;m.sourceUrl=source?.sourceUrl??null;m.sourceLoc=source?.sourceLoc??null;return {...m};},
  updateSource:async(id,source)=>{const m=notes.find(n=>n.id===id);if(!m)throw Error('笔记不存在或已被删除');m.sourceTitle=source.sourceTitle??null;m.sourceUrl=source.sourceUrl??null;m.sourceLoc=source.sourceLoc??null;},
  exportNote:async id=>`/tmp/export-${id.slice(0,8)}.md`,
  exportAll:async()=>'/tmp/export-all',
  rebuild:async()=>notes.length,
  topics:async()=>notes.map(n=>({topic:{id:`t-${n.id}`,name:`主题-${n.title}`,column:'',createdBy:'ai',batchId:'b1',createdAt:1,locked:false},noteIds:[n.id],count:1})),
  topicNotes:async topicId=>{const t=notes.find(n=>`t-${n.id}`===topicId);return t?[{meta:{...t,attachments:[]},body:'主题笔记'}]:[];},
  run:async ids=>({batchId:'batch-1',topics:ids.length,assignments:ids.length}),
  undo:async()=>{},
  renameTopic:async()=>{},setTopicColumn:async()=>{},mergeTopics:async()=>{},splitNote:async()=>{throw Error('x')},
  setLocked:async()=>{},deleteTopic:async()=>{},listRelations:async()=>[],addRelation:async()=>{throw Error('x')},removeRelation:async()=>{},
  setAuto:async (value:boolean)=>{autoFlag=value;},getAuto:async()=>autoFlag,
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

it('saveEdit updates the note, refreshes the list and re-reads the body',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();
 await f.controller.open(meta().id);
 const saved=await f.controller.saveEdit(meta().id,'编辑标题','编辑后的正文',{sourceTitle:'来源',sourceUrl:null,sourceLoc:null});
 expect(saved?.title).toBe('编辑标题');
 expect(f.controller.state.current?.body).toBe('编辑后的正文');
 expect(f.controller.state.notes[0].title).toBe('编辑标题');
 expect(f.controller.state.notice).toBe('');
});

it('a failed edit reports and keeps the current note untouched',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();
 await f.controller.open(meta().id);
 f.port.update=async()=>{throw Error('磁盘满');};
 const result=await f.controller.saveEdit(meta().id,'x','y',{sourceTitle:null,sourceUrl:null,sourceLoc:null});
 expect(result).toBeNull();
 expect(f.controller.state.notice).toContain('磁盘满');
 expect(f.controller.state.current?.body).toBe('正文');
});

it('organize flow: select, confirm gate, run, and undo refresh topics',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();await f.controller.loadTopics();
 f.controller.enterOrganize();
 expect(f.controller.state.organizeMode).toBe(true);
 f.controller.toggleSelect(meta().id);
 f.controller.startConfirm();
 expect(f.controller.state.confirming).toBe(true);
 const summary=await f.controller.runOrganize('fixture-model');
 expect(summary?.batchId).toBe('batch-1');
 expect(f.controller.state.lastBatch).toBe('batch-1');
 expect(f.controller.state.topics.length).toBe(1);
 await f.controller.undoLast();
 expect(f.controller.state.lastBatch).toBeNull();
 expect(f.controller.state.notice).toContain('已撤销');
});
it('selection is capped at 8 and requires the confirm gate',async()=>{
 const many=[...Array(10).keys()].map(i=>meta(String(i).padStart(32,'0')));
 const f=fixture(many);await f.controller.refresh();f.controller.enterOrganize();
 for(const n of many)f.controller.toggleSelect(n.id);
 expect(Object.keys(f.controller.state.selected).length).toBe(8);
 f.controller.toggleSelect(many[0].id);
 expect(f.controller.state.confirming).toBe(false);
 await f.controller.runOrganize('m');
 expect(f.controller.state.lastSummary).toBeNull(),"confirm gate blocks direct run";
});
it('topic overlay loads its notes and closes',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();await f.controller.loadTopics();
 await f.controller.openTopic('t-'+meta().id);
 expect(f.controller.state.topicView?.notes[0].body).toBe('主题笔记');
 f.controller.closeTopic();
 expect(f.controller.state.topicView).toBeNull();
});
it('a failed organize reports and keeps the selection',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();f.controller.enterOrganize();
 f.controller.toggleSelect(meta().id);f.controller.startConfirm();
 f.port.run=async()=>{throw Error('整理模型调用未成功');};
 await f.controller.runOrganize('m');
 expect(f.controller.state.notice).toContain('整理模型调用未成功');
 expect(Object.keys(f.controller.state.selected).length).toBe(1);
 expect(f.controller.state.lastBatch).toBeNull();
});

it('auto toggle persists and runAutoFor runs only when enabled',async()=>{
 let ran=0;let auto=false;
 const f=fixture([meta()]);
 f.port.run=async ids=>{ran++;return{batchId:`b${ran}`,topics:1,assignments:1};};
 f.port.getAuto=async()=>auto;f.port.setAuto=async (value:boolean)=>{auto=value;};
 await f.controller.runAutoFor(meta().id);
 expect(ran).toBe(0);
 await f.controller.setAuto(true);
 expect(f.controller.state.auto).toBe(true);
 await f.controller.runAutoFor(meta().id);
 expect(ran).toBe(1);
});
it('governance wrappers refresh topics and report failures',async()=>{
 const f=fixture([meta()]);await f.controller.refresh();await f.controller.loadTopics();
 await f.controller.renameTopic('t-'+meta().id,'改名');
 expect(f.controller.state.notice).toBe('');
 f.port.renameTopic=async()=>{throw Error('重命名未完成');};
 await f.controller.renameTopic('t-'+meta().id,'x');
 expect(f.controller.state.notice).toContain('重命名未完成');
});

it('save failure releases busy so retry can succeed',async()=>{
 const f=fixture();const save=f.port.save;
 f.port.save=async()=>{throw Error('磁盘满');};
 const input={title:'重试',body:'正文',source:'chat',model:'m',attachmentIds:[]};
 await f.controller.save(input);expect(f.controller.state.busy).toBe(false);
 f.port.save=save;expect((await f.controller.save(input))?.id).toBe('b'.repeat(32));
});
it('save returns persisted metadata even with duplicate titles and a filtered list',async()=>{
 const old=meta();const f=fixture([old]);await f.controller.refresh('不匹配');
 const result=await f.controller.save({title:old.title,body:'新正文',source:'chat',model:'m',attachmentIds:[]});
 expect(result?.id).toBe('b'.repeat(32));expect(f.controller.state.notes).toEqual([]);
});
it('initial refresh restores the persisted auto preference',async()=>{
 const f=fixture();f.port.getAuto=async()=>true;await f.controller.refresh();
 expect(f.controller.state.auto).toBe(true);
});
it('refresh reflects revoked automatic consent after model configuration changes',async()=>{const f=fixture();let consent=true;f.port.getAuto=async()=>consent;await f.controller.refresh();expect(f.controller.state.auto).toBe(true);consent=false;await f.controller.refresh();expect(f.controller.state.auto).toBe(false);});
