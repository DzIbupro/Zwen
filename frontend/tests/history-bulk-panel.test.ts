import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync,tick} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import HistoryPanel from '../HistoryPanel.svelte';
import {reactiveProps} from './fixtures/reactive-props.svelte';
it('clear confirmation invalidates only the reply attached when confirming, preserving a later reply',async()=>{
 let finish!:(n:number)=>void;let cleared=false;const gate=new Promise<number>(r=>finish=r);const removed:string[][]=[];
 const props=reactiveProps({port:{list:async()=>cleared?[]:[{requestId:'old',conversationId:'old',title:'合成旧记录',model:'fixture',state:'complete' as const,createdAt:1}],get:async()=>null,clear:async()=>{const n=await gate;cleared=true;return n;}},canContinue:true,canDelete:true,currentRequestId:'old',onContinue:()=>{},onBulkDeleted:(ids:string[])=>removed.push(ids)});
 const target=document.createElement('div');document.body.append(target);const app=mount(HistoryPanel,{target,props});const settle=async()=>{for(let i=0;i<8;i++){await tick();await Promise.resolve();flushSync();}};
 try{await settle();const clear=target.querySelector<HTMLButtonElement>('[data-history-clear]')!;expect(clear.disabled).toBe(false);clear.click();flushSync();expect(target.querySelector('.delete-confirm')?.textContent).toContain('所有分页');target.querySelector<HTMLButtonElement>('[data-history-confirm-delete]')!.click();props.currentRequestId='new';flushSync();finish(1);await settle();expect(removed).toEqual([['old']]);expect(target.querySelector('.records button')).toBeNull();}finally{await unmount(app);target.remove();}
});
