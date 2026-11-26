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
  key('appearance','End');expect(document.activeElement?.id).toBe('settings-tab-archive');expect(target.querySelector<HTMLDivElement>('#settings-panel-archive')!.hidden).toBe(false);
  key('archive','Home');expect(document.activeElement?.id).toBe('settings-tab-general');expect(target.querySelector<HTMLDivElement>('#settings-panel-general')!.hidden).toBe(false);
  key('general','ArrowLeft');expect(document.activeElement?.id).toBe('settings-tab-archive');
 }finally{await unmount(app);target.remove();}
});

import DesktopSettings from '../DesktopSettings.svelte';
import {defaultQuickActions} from '../lib/quick-actions';

const desktop = vi.hoisted(() => ({
  chooseExportDirectory: vi.fn(),
  openLocalDirectory: vi.fn(),
  loadBackgroundNotifications: vi.fn(),
  saveBackgroundNotifications: vi.fn(),
  loadKeepAwake: vi.fn(async () => false),
  saveKeepAwake: vi.fn(),
  loadQuickSurface: vi.fn(async () => 'capsule'),
  saveQuickSurface: vi.fn(),
}));

vi.mock('../lib/bridge', async importOriginal => ({
  ...await importOriginal<typeof import('../lib/bridge')>(),
  ...desktop,
}));

it('loads and saves the desktop notification opt-in and targets the configured folders', async () => {
  const target = document.createElement('div');
  document.body.append(target);
  const config = {revision: 4, actions: defaultQuickActions(), outputDirectory: '/tmp/zwen-export'};
  const updated = {...config, revision: 5, outputDirectory: '/tmp/zwen-next'};
  desktop.loadBackgroundNotifications.mockResolvedValue(false);
  desktop.saveBackgroundNotifications.mockResolvedValue(true);
  desktop.chooseExportDirectory.mockResolvedValue(updated);
  desktop.openLocalDirectory.mockResolvedValue(undefined);
  const onDirectoryChange = vi.fn();
  const app = mount(DesktopSettings, {target, props: {config, onDirectoryChange}});
  const render = async () => {
    await new Promise(resolve => setTimeout(resolve, 0));
    flushSync();
  };
  try {
    flushSync();
    await render();
    const toggle = target.querySelector<HTMLInputElement>('#background-notification-toggle')!;
    expect(toggle.checked).toBe(false);
    expect(toggle.disabled).toBe(false);
    expect(target.textContent).toContain(config.outputDirectory);
    toggle.click();
    await render();
    expect(desktop.saveBackgroundNotifications).toHaveBeenCalledWith(true);
    expect(toggle.checked).toBe(true);
    const button = (text: string) => [...target.querySelectorAll('button')].find(node => node.textContent === text)!;
    button('更改').click();
    await render();
    expect(desktop.chooseExportDirectory).toHaveBeenCalledWith(4);
    expect(onDirectoryChange).toHaveBeenCalledWith(updated);
    button('打开导出目录').click();
    await render();
    button('打开应用数据目录').click();
    await render();
    expect(desktop.openLocalDirectory.mock.calls).toEqual([[true], [false]]);
  } finally {
    await unmount(app);
    target.remove();
  }
});

it('uses a search result to select the category and focus the matching control', async () => {
  const target = document.createElement('div');
  document.body.append(target);
  const app = mount(Fixture, {target});
  try {
    flushSync();
    target.querySelector<HTMLButtonElement>('#settings-tab-appearance')!.click();
    const query = target.querySelector<HTMLInputElement>('#settings-search')!;
    query.value = '模型';
    query.dispatchEvent(new Event('input', {bubbles: true}));
    flushSync();
    const result = target.querySelector<HTMLButtonElement>('[data-setting-result="model-name"]')!;
    result.click();
    await new Promise(resolve => setTimeout(resolve, 0));
    flushSync();
    expect(target.querySelector('[aria-selected=true]')?.id).toBe('settings-tab-provider');
    expect(document.activeElement?.id).toBe('model-name');
  } finally {
    await unmount(app);
    target.remove();
  }
});
