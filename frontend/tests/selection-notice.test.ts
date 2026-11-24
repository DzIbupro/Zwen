import {it,expect,vi} from 'vitest';import {mount,unmount,flushSync} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import SelectionNotice from '../SelectionNotice.svelte';
import {reactiveProps} from './fixtures/reactive-props.svelte';
it('dismisses the main-window candidate on outside click without a close button',async()=>{
 let discarded=0;const props=reactiveProps({state:{busy:false,pending:{id:3,mode:'candidate' as const,text:'selected sentence',error:null,claimed:false},inserted:false,notice:'选中文字后选择操作'} as import('../lib/selection').SelectionState,onRetry:()=>{},onDiscard:()=>{discarded++;props.state={...props.state,pending:null,notice:''};},onReload:()=>{},onAction:()=>{}});
 const target=document.createElement('div');const outside=document.createElement('div');document.body.append(target,outside);const app=mount(SelectionNotice,{target,props});try{flushSync();expect(target.querySelector('[aria-label="关闭选区提示"]')).toBeNull();outside.dispatchEvent(new Event('pointerdown',{bubbles:true}));flushSync();expect(discarded).toBe(1);expect(target.querySelector('[data-selection-state]')).toBeNull();}finally{await unmount(app);target.remove();outside.remove();}
});
