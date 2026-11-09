import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync} from 'svelte';
// Vitest uses Node export conditions; DOM mounting needs Svelte's client entry.
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import KnowledgePanel from '../KnowledgePanel.svelte';
import KnowledgeGraph from '../KnowledgeGraph.svelte';
import {KnowledgeController,type KnowledgePort} from '../lib/knowledge';
const settle=async()=>{await new Promise(resolve=>setTimeout(resolve,0));flushSync();};
it('failed edit retains entered text and allows retry',async()=>{
 const meta={id:'n1',title:'原标题',createdAt:1,source:'chat',model:'m',attachments:[]};
 const port={update:vi.fn().mockRejectedValue(Error('磁盘满'))} as unknown as KnowledgePort;
 const controller=new KnowledgeController(port,()=>{});controller.state.current={meta,body:'原正文'};
 const target=document.createElement('div');document.body.append(target);
 const app=mount(KnowledgePanel,{target,props:{controller,info:controller.state,onTrace:()=>{},onSaveEdit:(...args)=>controller.saveEdit(...args),modelLabel:'当前模型'}});
 flushSync();(target.querySelector('[data-note-edit]') as HTMLButtonElement).click();flushSync();
 const input=target.querySelector('[data-note-body-input]') as HTMLTextAreaElement;input.value='未保存的修改';input.dispatchEvent(new Event('input',{bubbles:true}));flushSync();
 (target.querySelector('[data-note-save-edit]') as HTMLButtonElement).click();await settle();
 expect((target.querySelector('[data-note-body-input]') as HTMLTextAreaElement)?.value).toBe('未保存的修改');
 expect((target.querySelector('[data-note-save-edit]') as HTMLButtonElement).disabled).toBe(false);
 await unmount(app);target.remove();
});
it('relation activation exposes reason and opens evidence note',async()=>{
 const controller=new KnowledgeController({} as KnowledgePort,()=>{});
 controller.state.topics=['t1','t2'].map(id=>({topic:{id,name:id,column:'',createdBy:'ai',batchId:null,createdAt:1,locked:false},count:1,noteIds:['n1']}));
 controller.state.relations=[{id:'r1',fromTopic:'t1',toTopic:'t2',reason:'共同论据',evidence:'n1',addedBy:'ai'}];
 const target=document.createElement('div');document.body.append(target);const onOpenNote=vi.fn();
 const app=mount(KnowledgeGraph,{target,props:{controller,info:controller.state,onOpenNote,onOpenTopic:()=>{}}});flushSync();
 (Array.from(target.querySelectorAll('button')).find(b=>b.textContent==='主题') as HTMLButtonElement).click();flushSync();
 target.querySelector('[data-edge="r1"]')!.dispatchEvent(new MouseEvent('click',{bubbles:true}));flushSync();
 expect(target.querySelector('[data-relation-detail]')?.textContent).toContain('共同论据');
 (target.querySelector('[data-relation-evidence]') as HTMLButtonElement).click();expect(onOpenNote).toHaveBeenCalledWith('n1');
 await unmount(app);target.remove();
});
it('unsaved note editing survives leaving and remounting the knowledge view',async()=>{
 const meta={id:'n1',title:'原标题',createdAt:1,source:'chat',model:'m',attachments:[]};
 const controller=new KnowledgeController({} as KnowledgePort,()=>{});controller.state.current={meta,body:'原正文'};
 const target=document.createElement('div');document.body.append(target);
 const props={controller,info:controller.state,onTrace:()=>{},onSaveEdit:async()=>null,modelLabel:'测试模型'};
 let app=mount(KnowledgePanel,{target,props});flushSync();(target.querySelector('[data-note-edit]') as HTMLButtonElement).click();flushSync();
 const input=target.querySelector('[data-note-body-input]') as HTMLTextAreaElement;input.value='切换视图前未保存的修改';input.dispatchEvent(new Event('input',{bubbles:true}));flushSync();
 await unmount(app);app=mount(KnowledgePanel,{target,props:{...props,info:controller.state}});await settle();
 expect((target.querySelector('[data-note-body-input]') as HTMLTextAreaElement)?.value).toBe('切换视图前未保存的修改');
 await unmount(app);target.remove();controller.dispose();
});

it('a pending edit save locks fields until completion',async()=>{
 const meta={id:'pending',title:'标题',createdAt:1,source:'chat',model:'m',attachments:[]};
 const controller=new KnowledgeController({} as KnowledgePort,()=>{});controller.state.current={meta,body:'正文'};
 const target=document.createElement('div');document.body.append(target);
 let finish!:(value:null)=>void;
 const app=mount(KnowledgePanel,{target,props:{controller,info:controller.state,onTrace:()=>{},onSaveEdit:()=>new Promise<null>(resolve=>finish=resolve),modelLabel:'测试'}});
 flushSync();(target.querySelector('[data-note-edit]') as HTMLButtonElement).click();flushSync();
 (target.querySelector('[data-note-save-edit]') as HTMLButtonElement).click();flushSync();
 expect((target.querySelector('[data-note-body-input]') as HTMLTextAreaElement).disabled).toBe(true);
 finish(null);await settle();
 expect((target.querySelector('[data-note-body-input]') as HTMLTextAreaElement).disabled).toBe(false);
 await unmount(app);target.remove();controller.dispose();
});
