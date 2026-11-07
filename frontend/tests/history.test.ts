import {it,expect} from 'vitest';
import {HistoryController} from '../lib/history';
import type {ChatReply} from '../lib/chat';
const reply=(id:string):ChatReply=>({requestId:id,conversationId:id,contextTurns:0,revision:1,answer:id,prompt:id,model:'m',state:'complete',error:null,saved:true,createdAt:0});
it('late history selection cannot replace a newer selection',async()=>{let resolve!:(r:ChatReply)=>void;const pending=new Promise<ChatReply>(r=>resolve=r);const c=new HistoryController({list:async()=>[],get:id=>id==='a'?pending:Promise.resolve(reply(id))},()=>{});const a=c.open('a');await c.open('b');resolve(reply('a'));await a;expect(c.state.selected?.requestId).toBe('b');});
it('history pagination keeps at most twenty summaries',async()=>{const rows=Array.from({length:21},(_,i)=>({requestId:String(i),conversationId:'c',title:'q',model:'m',state:'complete' as const,createdAt:0}));const c=new HistoryController({list:async()=>rows,get:async()=>null},()=>{});await c.load(0);expect(c.state.items.length).toBe(20);expect(c.state.hasMore).toBe(true);});
it('deleting selected history clears it, refreshes an empty final page, and invalidates its parent reference',async()=>{
 const deleted:string[]=[];const parent:string[]=[];const c=new HistoryController({list:async page=>page===1?[]:[{...reply('b'),title:'b'}],get:async id=>reply(id),remove:async id=>{deleted.push(id);}},()=>{},id=>parent.push(id));
 c.state.page=1;await c.open('a');await c.removeSelected();expect(deleted).toEqual(['a']);expect(parent).toEqual(['a']);expect(c.state.selected).toBeNull();expect(c.state.page).toBe(0);expect(c.state.items[0].requestId).toBe('b');expect(c.state.deleting).toBe(false);
});
it('failed history deletion keeps the visible record and never invalidates the parent',async()=>{
 const parent:string[]=[];const c=new HistoryController({list:async()=>[],get:async id=>reply(id),remove:async()=>{throw Error('合成删除失败');}},()=>{},id=>parent.push(id));await c.open('a');await c.removeSelected();expect(c.state.selected?.requestId).toBe('a');expect(c.state.error).toBe('合成删除失败');expect(parent).toEqual([]);expect(c.state.deleting).toBe(false);
});
it('deletion prevents interleaved navigation and rejects streaming records',async()=>{
 let done!:()=>void;const gate=new Promise<void>(r=>done=r);let removes=0;const c=new HistoryController({list:async()=>[],get:async id=>reply(id),remove:async()=>{removes++;await gate;}},()=>{});await c.open('a');const work=c.removeSelected();await c.open('b');expect(c.state.selected?.requestId).toBe('a');done();await work;await c.open('a');c.state.selected!.state='streaming';await c.removeSelected();expect(removes).toBe(1);
});
it('keeps cross-page checks and deletes only the confirmed snapshot while retaining unrelated detail',async()=>{
 const rows=Array.from({length:25},(_,i)=>({...reply(String(i)),title:String(i)}));let remaining=rows;const seen:(string[]|null)[]=[];
 const c=new HistoryController({list:async p=>remaining.slice(p*20,p*20+21),get:async id=>reply(id),removeMany:async ids=>{remaining=remaining.filter(r=>!ids.includes(r.requestId));return ids.length;}},()=>{},()=>{},ids=>seen.push(ids));
 await c.load(0);c.toggleChecked('0');await c.load(1);c.toggleChecked('20');expect(Object.keys(c.state.checked)).toEqual(['0','20']);await c.open('24');const snapshot=Object.keys(c.state.checked);c.toggleChecked('21');await c.removeMany(snapshot);
 expect(seen).toEqual([['0','20']]);expect(c.state.selected?.requestId).toBe('24');expect(c.state.checked).toEqual({'21':true});expect(c.state.page).toBe(0);expect(remaining).toHaveLength(23);
});
it('failed batch keeps checks and details; all uses the global port and detaches every current reference',async()=>{
 let fail=true;const callbacks:(string[]|null)[]=[];let clearCalls=0;
 const c=new HistoryController({list:async()=>[],get:async id=>reply(id),removeMany:async()=>{if(fail)throw Error('整批删除失败');return 2;},clear:async()=>{clearCalls++;return 45;}},()=>{},()=>{},ids=>callbacks.push(ids));await c.open('a');c.state.checked={a:true,b:true};await c.removeMany(['a','b']);expect(c.state.checked).toEqual({a:true,b:true});expect(c.state.selected?.requestId).toBe('a');expect(callbacks).toEqual([]);
 fail=false;await c.clearAll();expect(clearCalls).toBe(1);expect(callbacks).toEqual([null]);expect(c.state.checked).toEqual({});expect(c.state.selected).toBeNull();expect(c.state.notice).toContain('45');
});
it('select-page toggles only eligible current-page records',async()=>{
 const c=new HistoryController({list:async()=>[],get:async()=>null},()=>{});c.state.items=[{...reply('a'),title:'a'},{...reply('live'),title:'live',state:'streaming'}];c.state.checked={other:true};c.togglePage();expect(c.state.checked).toEqual({other:true,a:true});c.togglePage();expect(c.state.checked).toEqual({other:true});
});
