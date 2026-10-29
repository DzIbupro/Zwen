import {it,expect,vi,afterEach} from 'vitest';
import {DemoStream} from '../lib/demo';
afterEach(()=>vi.useRealTimers());
it('streams incrementally and stops without losing received text',async()=>{vi.useFakeTimers();let value='';const s=new DemoStream(v=>value=v);s.start();expect(value).toBe('');await vi.advanceTimersByTimeAsync(200);expect(value.length).toBeGreaterThan(0);const partial=value;s.stop();await vi.advanceTimersByTimeAsync(1000);expect(value).toBe(partial);});
