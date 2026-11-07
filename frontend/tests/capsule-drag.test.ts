import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
vi.mock('../lib/bridge',async()=>({...await vi.importActual<typeof import('../lib/bridge')>('../lib/bridge'),startCapsuleDrag:vi.fn(async()=>{})}));
import {startCapsuleDrag} from '../lib/bridge';
import Capsule from '../Capsule.svelte';
import {reactiveProps} from './fixtures/reactive-props.svelte';
it('starts native dragging from the empty or filled capsule header while leaving controls and text interactive',async()=>{
 vi.stubGlobal('ResizeObserver',class{observe(){}disconnect(){}});
 const noop=()=>{};const target=document.createElement('div');document.body.append(target);
 const props=reactiveProps({selectionState:{busy:false,pending:null,inserted:false,notice:''},onSelectionRetry:noop,onSelectionDiscard:noop,onSelectionReload:noop,attachments:{items:[],pending:false,notice:'',loaded:true,max:3},onAttachmentCapture:noop,onAttachmentCancel:noop,onAttachmentRemove:noop,onAttachmentRetake:noop,notice:'',draft:{text:'合成草稿',revision:1,status:'saved' as const,error:null,loaded:true},surface:{mode:'capsule' as const,scaleFactor:1,generation:1,bounds:{width:480,initialHeight:96,maxHeight:594}},response:'',running:false,busy:false,replyLabel:'模型回复',replyStatus:'',modelStatus:'已配置',canSend:true,unsaved:false,onSend:noop,onRetrySave:noop,onEdit:noop,onSave:noop,onHide:noop,onExpand:noop,onStop:noop,onResize:noop});
 const app=mount(Capsule,{target,props});const down=(el:Element,button=0,detail=1)=>el.dispatchEvent(new MouseEvent('mousedown',{bubbles:true,cancelable:true,button,detail}));
 try{
  flushSync();vi.mocked(startCapsuleDrag).mockClear();down(target.querySelector('.capsule-brand')!);expect(startCapsuleDrag).toHaveBeenCalledTimes(1);
  down(target.querySelector('button')!);down(target.querySelector('textarea')!);down(target.querySelector('.capsule-head')!,2);down(target.querySelector('.capsule-head')!,0,2);expect(startCapsuleDrag).toHaveBeenCalledTimes(1);
  props.response='完整合成回复';flushSync();down(target.querySelector('[data-capsule-answer]')!);expect(startCapsuleDrag).toHaveBeenCalledTimes(1);
  down(target.querySelector('.capsule-head')!);expect(startCapsuleDrag).toHaveBeenCalledTimes(2);
  props.running=true;flushSync();down(target.querySelector('.capsule-head')!);expect(startCapsuleDrag).toHaveBeenCalledTimes(3);
 }finally{await unmount(app);target.remove();vi.unstubAllGlobals();}
});
