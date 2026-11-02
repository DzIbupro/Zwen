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
