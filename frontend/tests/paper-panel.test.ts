import {it,expect,vi} from 'vitest';import {mount,unmount,flushSync} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import PaperPanel from '../PaperPanel.svelte';
import {reactiveProps} from './fixtures/reactive-props.svelte';
it('imports locally, waits for explicit analysis, and retains Markdown without JSON parsing',async()=>{
 let starts=0;const source={id:'source',name:'fixture.pdf',pageCount:2,readablePages:2};
 const props=reactiveProps({port:{list:async()=>[source],choose:async()=>source,sessions:async()=>[],source:async()=>['first','second']},chat:{answer:'',running:false,stopping:false,loaded:true,error:null,reply:null} as import('../lib/chat').ChatState,desktop:true,model:'fixture-model',onStart:async()=>{starts++;props.chat={...props.chat,answer:'# 论文报告\n\n正文',reply:{requestId:'r',conversationId:'r',contextTurns:0,revision:1,answer:'# 论文报告\n\n正文',prompt:'read',model:'fixture-model',state:'complete',error:null,saved:true,createdAt:0}};return true;},onFollowup:async()=>true,onAdopt:()=>{},onStop:()=>{},onSave:async()=>{},onExport:async()=>''});
 const target=document.createElement('div');document.body.append(target);const app=mount(PaperPanel,{target,props});
 try{await new Promise(r=>setTimeout(r,10));flushSync();expect(starts).toBe(0);(target.querySelector('[data-paper-id]') as HTMLButtonElement).click();await new Promise(r=>setTimeout(r,10));flushSync();expect(target.textContent).toContain('图表图像');(target.querySelector('[data-paper-start]') as HTMLButtonElement).click();await new Promise(r=>setTimeout(r,10));flushSync();expect(starts).toBe(1);expect(target.querySelector('.prose h1')?.textContent).toBe('论文报告');}finally{await unmount(app);target.remove();}
});
