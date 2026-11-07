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
