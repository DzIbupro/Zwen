import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import RetryNotice from '../RetryNotice.svelte';
it('shows retry count and countdown, then releases its timer on unmount',async()=>{vi.useFakeTimers();vi.setSystemTime(new Date('2026-11-04T00:00:00Z'));const target=document.createElement('div');document.body.append(target);const app=mount(RetryNotice,{target,props:{retry:{attempt:2,nextRetryAt:Date.now()+10000,reason:'rate-limit'}}});try{flushSync();expect(target.textContent).toContain('10 秒');expect(target.textContent).toContain('第2次');expect(target.textContent).toContain('已保留');await vi.advanceTimersByTimeAsync(10000);flushSync();expect(target.textContent).toContain('正在重新请求');}finally{await unmount(app);target.remove();expect(vi.getTimerCount()).toBe(0);vi.useRealTimers();}});
