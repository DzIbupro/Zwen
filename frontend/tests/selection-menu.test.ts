import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import SelectionNotice from '../SelectionNotice.svelte';
it('candidate exposes the four actions and custom prompt settings without sending on render',async()=>{
 const target=document.createElement('div');document.body.append(target);const calls:string[]=[];
 const app=mount(SelectionNotice,{target,props:{state:{busy:false,pending:{id:1,mode:'candidate',text:'测试选区',claimed:false,error:null},inserted:false,notice:''},onRetry:()=>{},onDiscard:()=>{},onReload:()=>{},onAction:(id:string)=>calls.push(id),onSettings:()=>calls.push('settings'),actions:[{id:'translate',name:'翻译',prompt:'{{selection}}',output:'display',enabled:true},{id:'explain',name:'详细解释',prompt:'{{selection}}',output:'display',enabled:true},{id:'document',name:'整理成文档',prompt:'{{selection}}',output:'document',enabled:true}]}});flushSync();
 expect(calls).toEqual([]);expect(target.textContent).toContain('问一问 Zwen');expect(target.textContent).toContain('详细解释');expect(target.textContent).toContain('其他');
 (target.querySelector('[data-selection-action="translate"]') as HTMLButtonElement).click();expect(calls).toEqual(['translate']);
 await unmount(app);target.remove();
});
it('automatic candidate capsule hides old answers and the question composer',async()=>{
 vi.stubGlobal('ResizeObserver',class{observe(){}disconnect(){}});
 const {default:Capsule}=await import('../Capsule.svelte');const noop=()=>{};
 const target=document.createElement('div');document.body.append(target);
 const app=mount(Capsule,{target,props:{selectionState:{busy:false,pending:{id:7,mode:'candidate',text:'新选区',claimed:false,error:null},inserted:false,notice:''},onSelectionRetry:noop,onSelectionDiscard:noop,onSelectionReload:noop,attachments:{items:[],pending:false,notice:'',loaded:true,max:3},onAttachmentCapture:noop,onAttachmentCancel:noop,onAttachmentRemove:noop,onAttachmentRetake:noop,notice:'',draft:{text:'',revision:0,status:'saved',error:null,loaded:true},surface:{mode:'capsule',scaleFactor:1,generation:1,bounds:{width:480,initialHeight:96,maxHeight:594}},response:'上一次的长回复绝对不应出现在候选菜单',running:false,busy:false,replyLabel:'模型回复',replyStatus:'已完成',modelStatus:'已配置',canSend:true,unsaved:false,onSend:noop,onRetrySave:noop,onEdit:noop,onSave:noop,onHide:noop,onExpand:noop,onDemo:noop,onStop:noop,onResize:noop,onSelectionAction:noop}});flushSync();
 expect(target.querySelector('[data-capsule-answer]')).toBeNull();expect(target.querySelector('#capsule-question')).toBeNull();expect(target.textContent).not.toContain('上一次的长回复');
 await unmount(app);target.remove();vi.unstubAllGlobals();
});
