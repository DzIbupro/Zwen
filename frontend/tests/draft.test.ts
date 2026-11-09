import { describe, it, expect, vi, afterEach } from 'vitest';
import { DraftController, type Draft, type DraftPort } from '../lib/draft';
function port(initial=''): DraftPort {
 let saved: Draft = {text:initial,revision:0};
 return {load:async()=>({...saved}),save:async(revision,text)=>{
  if(revision!==saved.revision) throw Error('conflict');saved={text,revision:revision+1};return {...saved};
 }};
}
afterEach(()=>vi.useRealTimers());
describe('draft workflow',()=>{
 it('restores the saved draft before editing',async()=>{
  const c=new DraftController(port('已保存的中文草稿'),()=>{});await c.start();expect(c.snapshot.text).toBe('已保存的中文草稿');expect(c.snapshot.status).toBe('saved');
 });
 it('autosaves after 500ms, retaining Unicode',async()=>{
  vi.useFakeTimers();const p=port();const c=new DraftController(p,()=>{});await c.start();c.edit('你好 🌿');await vi.advanceTimersByTimeAsync(500);expect((await p.load()).text).toBe('你好 🌿');expect(c.snapshot.status).toBe('saved');
 });
 it('serializes edits arriving while a save is pending',async()=>{
  const backing=port();let release!:()=>void;let first=true;
  const delayed:DraftPort={load:()=>backing.load(),save:async(r,t)=>{if(first){first=false;await new Promise<void>(resolve=>release=resolve);}return backing.save(r,t);}};
  const c=new DraftController(delayed,()=>{});await c.start();c.edit('first');const saving=c.flush();await Promise.resolve();c.edit('latest');release();await saving;
  expect((await backing.load()).text).toBe('latest');expect(c.snapshot.text).toBe('latest');expect(c.snapshot.status).toBe('saved');
 });
 it('keeps text and prevents leaving when persistence fails, then allows retry',async()=>{
  let fail=true;const backing=port('old');const p:DraftPort={load:()=>backing.load(),save:async(r,t)=>{if(fail)throw Error('磁盘不可写');return backing.save(r,t);}};
  const c=new DraftController(p,()=>{});await c.start();c.edit('不能丢失');let hidden=false;
  await expect(c.leave(async()=>{hidden=true;})).rejects.toThrow('磁盘不可写');expect(hidden).toBe(false);expect(c.snapshot.text).toBe('不能丢失');expect(c.snapshot.status).toBe('error');
  fail=false;await c.leave(async()=>{hidden=true;});expect(hidden).toBe(true);expect((await backing.load()).text).toBe('不能丢失');
 });
 it('does not announce saved when reverting during a pending write',async()=>{
  let release!:()=>void;const backing=port('original');let first=true;
  const p:DraftPort={load:()=>backing.load(),save:async(r,t)=>{if(first){first=false;await new Promise<void>(resolve=>release=resolve);}return backing.save(r,t);}};
  const c=new DraftController(p,()=>{});await c.start();c.edit('temporary');const pending=c.flush();await Promise.resolve();c.edit('original');
  const status=c.snapshot.status;release();await pending;
  expect(status).toBe('dirty');expect((await backing.load()).text).toBe('original');
 });
 it('does not overwrite a draft changed by another writer' ,async()=>{
  const p=port();const c=new DraftController(p,()=>{});await c.start();await p.save(0,'newer');c.edit('local');await expect(c.flush()).rejects.toThrow('conflict');expect(c.snapshot.text).toBe('local');expect((await p.load()).text).toBe('newer');
 });
});

it('inserts a suggestion without discarding a restored draft',async()=>{
 const p=port('重要的旧草稿');const c=new DraftController(p,()=>{});await c.start();
 c.insertPrompt('请帮我解释：\n\n');await c.flush();
 expect((await p.load()).text).toBe('重要的旧草稿\n\n请帮我解释：\n\n');
});
