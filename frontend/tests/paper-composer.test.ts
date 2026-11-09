import {it,expect,vi} from 'vitest';import {mount,unmount,flushSync} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import PaperComposer from '../PaperComposer.svelte';import {PaperDraftController} from '../lib/paper-draft';import {defaultReadingOptions} from '../lib/papers';import {reactiveProps} from './fixtures/reactive-props.svelte';
it('offers a small chat tool, imports a removable PDF and never starts analysis itself',async()=>{
 const paper={id:'p',name:'fixture.pdf',pageCount:2,readablePages:2};const defaults={...defaultReadingOptions(),focus:'已内置关注问题',outputRequirements:'已内置报告结构'};
 const port={defaults:async()=>defaults,choose:async()=>paper,list:async()=>[paper]};
 const controller=new PaperDraftController(port,s=>props.state=s);const props=reactiveProps({controller,state:controller.state,port,context:null,disabled:false,desktop:true,onExit:()=>controller.close()});
 const target=document.createElement('div');document.body.append(target);const app=mount(PaperComposer,{target,props});
 try{flushSync();expect(target.querySelector('[data-paper-upload]')).toBeNull();(target.querySelector('[data-paper-toggle]') as HTMLButtonElement).click();await new Promise(r=>setTimeout(r,10));flushSync();(target.querySelector('[data-paper-upload]') as HTMLButtonElement).click();await new Promise(r=>setTimeout(r,10));flushSync();expect(target.textContent).toContain('fixture.pdf');expect(target.textContent).toContain('点击发送');expect(target.querySelector('details[data-paper-options]')?.hasAttribute('open')).toBe(false);expect(target.querySelector('[data-paper-start]')).toBeNull();(target.querySelector('[aria-label="移除论文附件"]') as HTMLButtonElement).click();flushSync();expect(controller.state.paper).toBeNull();}finally{await unmount(app);controller.dispose();target.remove();}
});
it('can replace a PDF from a restored paper conversation before defaults are loaded',async()=>{
 const paper={id:'new',name:'replacement.pdf',pageCount:3,readablePages:2};let chosen=0;const defaults={...defaultReadingOptions(),focus:'default',outputRequirements:'report'};
 const port={defaults:async()=>defaults,choose:async()=>{chosen++;return paper;},list:async()=>[paper]};
 const controller=new PaperDraftController(port,s=>props.state=s);const props=reactiveProps({controller,state:controller.state,port,context:{source:{...paper,id:'old',name:'old.pdf'},options:defaults,endpoint:'https://example.test/v1',model:'saved'},reading:true,disabled:false,desktop:true,onExit:()=>{}});
 const target=document.createElement('div');document.body.append(target);const app=mount(PaperComposer,{target,props});try{flushSync();const upload=target.querySelector('[data-paper-upload]') as HTMLButtonElement;expect(upload.disabled).toBe(false);upload.click();await new Promise(r=>setTimeout(r,10));flushSync();expect(chosen).toBe(1);expect(target.textContent).toContain('可提取 2/3 页');}finally{await unmount(app);controller.dispose();target.remove();}
});
