import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import Fixture from './fixtures/SettingsHubFixture.svelte';
it('defaults to model settings and preserves unsaved model and secret inputs across categories',async()=>{
 const target=document.createElement('div');document.body.append(target);const save=vi.fn(),reload=vi.fn();const app=mount(Fixture,{target,props:{onSave:save,onReload:reload}});flushSync();
 try{
  expect(target.querySelector('[role=tab][aria-selected=true]')?.textContent).toContain('模型与凭据');
  const model=target.querySelector<HTMLInputElement>('#model-name')!,secret=target.querySelector<HTMLInputElement>('#api-secret')!;
  model.value='unsaved-model';model.dispatchEvent(new Event('input',{bubbles:true}));secret.value='unsaved-secret';secret.dispatchEvent(new Event('input',{bubbles:true}));flushSync();
  target.querySelector<HTMLButtonElement>('#settings-tab-appearance')!.click();flushSync();
  expect(target.querySelector<HTMLDivElement>('#settings-panel-provider')!.hidden).toBe(true);
  target.querySelector<HTMLButtonElement>('#settings-tab-provider')!.click();flushSync();
  expect(target.querySelector('#api-secret')).toBe(secret);expect(secret.value).toBe('unsaved-secret');expect(model.value).toBe('unsaved-model');expect(save).not.toHaveBeenCalled();expect(reload).not.toHaveBeenCalled();
  target.querySelector<HTMLButtonElement>('.actions button')!.click();expect(reload).toHaveBeenCalledOnce();
 }finally{await unmount(app);target.remove();}
});
it('supports arrow, Home and End keyboard navigation with selected tab focus',async()=>{
 const target=document.createElement('div');document.body.append(target);const app=mount(Fixture,{target});flushSync();
 try{
  const key=(id:string,key:string)=>{target.querySelector(`#settings-tab-${id}`)!.dispatchEvent(new KeyboardEvent('keydown',{key,bubbles:true}));flushSync();};
  key('provider','ArrowRight');expect(document.activeElement?.id).toBe('settings-tab-appearance');expect(target.querySelector('[aria-selected=true]')?.id).toBe('settings-tab-appearance');
  key('appearance','End');expect(document.activeElement?.id).toBe('settings-tab-quick');expect(target.querySelector<HTMLDivElement>('#settings-panel-quick')!.hidden).toBe(false);
  key('quick','Home');expect(document.activeElement?.id).toBe('settings-tab-provider');
  key('provider','ArrowLeft');expect(document.activeElement?.id).toBe('settings-tab-quick');
 }finally{await unmount(app);target.remove();}
});
