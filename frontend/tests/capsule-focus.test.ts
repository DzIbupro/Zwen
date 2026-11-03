import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync,tick} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import Capsule from '../Capsule.svelte';
import {reactiveProps} from './fixtures/reactive-props.svelte';
it('focuses the end of an imported multiline draft without resetting the caret during editing',async()=>{
 vi.stubGlobal('ResizeObserver',class{observe(){}disconnect(){}});
 const target=document.createElement('div');document.body.append(target);const noop=()=>{};
 const text='合成选中文字\n'.repeat(30);
 const props=reactiveProps({selectionState:{busy:false,pending:null,inserted:false,notice:''},onSelectionRetry:noop,onSelectionDiscard:noop,onSelectionReload:noop,attachments:{items:[],pending:false,notice:'',loaded:true,max:3},onAttachmentCapture:noop,onAttachmentCancel:noop,onAttachmentRemove:noop,onAttachmentRetake:noop,notice:'',draft:{text,revision:1,status:'saved' as const,error:null,loaded:true},surface:{mode:'capsule' as const,scaleFactor:1,generation:1,bounds:{width:480,initialHeight:96,maxHeight:594}},response:'',running:false,busy:true,replyLabel:'模型回复',replyStatus:'',modelStatus:'已配置',canSend:true,unsaved:false,onSend:noop,onRetrySave:noop,onEdit:noop,onSave:noop,onHide:noop,onExpand:noop,onDemo:noop,onStop:noop,onResize:noop});
 const app=mount(Capsule,{target,props});
 try{flushSync();const input=target.querySelector('textarea')!;Object.defineProperty(input,'scrollHeight',{value:600});await tick();expect(input.disabled).toBe(true);props.busy=false;flushSync();await tick();await Promise.resolve();expect(document.activeElement).toBe(input);expect(input.selectionStart).toBe(text.length);expect(input.selectionEnd).toBe(text.length);expect(input.scrollTop).toBe(600);input.setSelectionRange(2,2);input.dispatchEvent(new Event('input',{bubbles:true}));await tick();expect(input.selectionStart).toBe(2);}finally{await unmount(app);target.remove();vi.unstubAllGlobals();}
});
