import {expect,it,vi} from 'vitest';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import {defaultQuickActions,validateQuickActions,renderQuickActionPrompt} from '../lib/quick-actions';
it('defaults include editable display builtins and a document example',()=>{const a=defaultQuickActions();expect(a.map(x=>x.id)).toEqual(['translate','explain','document']);expect(a[2].output).toBe('document');expect(validateQuickActions(a,'/tmp/docs')).toBeNull();});
it('rejects duplicate IDs, missing builtins and unbounded input',()=>{const a=defaultQuickActions();expect(validateQuickActions([...a,a[0]],'/tmp')).toBeTruthy();expect(validateQuickActions(a.slice(1),'/tmp')).toBeTruthy();a[0].prompt='a'.repeat(16385);expect(validateQuickActions(a,'/tmp')).toBeTruthy();});
it('inserts selection literally without expanding embedded placeholders',()=>{expect(renderQuickActionPrompt('A {{selection}} B {{selection}}','{{selection}}')).toBe('A {{selection}} B {{selection}}');expect(renderQuickActionPrompt('Explain','text')).toBe('Explain\n\ntext');});
it('rejects relative output directories and invalid outputs',()=>{const a=defaultQuickActions();expect(validateQuickActions(a,'../x')).toBeTruthy();a[0].output='shell' as never;expect(validateQuickActions(a,'/tmp')).toBeTruthy();});
import {mount,unmount,flushSync} from 'svelte';
import QuickActionSettings from '../QuickActionSettings.svelte';
it('settings preserve entered prompt and directory after a CAS save error',async()=>{
 const target=document.createElement('div');document.body.append(target);const port={load:async()=>({revision:2,actions:defaultQuickActions(),outputDirectory:'/tmp/docs'}),save:async()=>{throw Error('版本冲突');}};
 const app=mount(QuickActionSettings,{target,props:{port,onChanged:()=>{}}});flushSync();await new Promise(r=>setTimeout(r,0));flushSync();
 const prompt=target.querySelector('textarea') as HTMLTextAreaElement;prompt.value='我修改的提示词 {{selection}}';prompt.dispatchEvent(new Event('input',{bubbles:true}));const directory=target.querySelector('[aria-label="文档输出目录"]') as HTMLInputElement;directory.value='/tmp/newdocs';directory.dispatchEvent(new Event('input',{bubbles:true}));flushSync();
 target.querySelector('form')!.dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));await new Promise(r=>setTimeout(r,0));flushSync();expect(prompt.value).toBe('我修改的提示词 {{selection}}');expect(directory.value).toBe('/tmp/newdocs');expect(target.querySelector('[role="alert"]')?.textContent).toContain('版本冲突');await unmount(app);target.remove();
});
