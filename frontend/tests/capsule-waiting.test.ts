import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync} from 'svelte';
import type {ComponentProps} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import Capsule from '../Capsule.svelte';
import {reactiveProps} from './fixtures/reactive-props.svelte';
it('shows waiting before the first token, then replaces it with the streaming answer',async()=>{
 vi.stubGlobal('ResizeObserver',class{observe(){}disconnect(){}});const noop=()=>{};
 const props=reactiveProps<ComponentProps<typeof Capsule>>({selectionState:{busy:false,pending:null,inserted:false,notice:''},onSelectionRetry:noop,onSelectionDiscard:noop,onSelectionReload:noop,attachments:{items:[],pending:false,notice:'',loaded:true,max:3},onAttachmentCapture:noop,onAttachmentCancel:noop,onAttachmentRemove:noop,onAttachmentRetake:noop,notice:'',draft:{text:'',revision:0,status:'saved',error:null,loaded:true},surface:{mode:'capsule',scaleFactor:1,generation:1,bounds:{width:480,initialHeight:96,maxHeight:594}},response:'',running:true,busy:false,replyLabel:'模型回复',replyStatus:'生成中',modelStatus:'已配置',canSend:false,unsaved:false,onSend:noop,onRetrySave:noop,onEdit:noop,onSave:noop,onHide:noop,onExpand:noop,onStop:noop,onResize:noop});
 const target=document.createElement('div');document.body.append(target);const app=mount(Capsule,{target,props});
 try{flushSync();expect(target.querySelector('[data-capsule-waiting]')?.textContent).toContain('正在等待模型回复');expect(target.textContent).toContain('停止');props.selectionState={busy:false,pending:{id:7,mode:'candidate',text:'合成选区',claimed:true,error:null},inserted:false,notice:''};props.actionBusy=true;flushSync();expect(target.querySelector('[data-capsule-waiting]')).not.toBeNull();props.selectionState={busy:false,pending:null,inserted:false,notice:''};props.actionBusy=false;props.response='合成回复首字';flushSync();expect(target.querySelector('[data-capsule-waiting]')).toBeNull();expect(target.querySelector('[data-capsule-answer]')?.textContent).toContain('合成回复首字');props.running=false;props.response='';props.notice='合成请求失败，请手动重试';flushSync();expect(target.querySelector('[data-capsule-waiting]')).toBeNull();expect(target.textContent).toContain('合成请求失败');}finally{await unmount(app);target.remove();vi.unstubAllGlobals();}
});
