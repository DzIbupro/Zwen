import {it,expect} from 'vitest';
import {HistoryController} from '../lib/history';
import type {ChatReply} from '../lib/chat';
const reply=(id:string):ChatReply=>({requestId:id,conversationId:id,contextTurns:0,revision:1,answer:id,prompt:id,model:'m',state:'complete',error:null,saved:true,createdAt:0});
it('late history selection cannot replace a newer selection',async()=>{let resolve!:(r:ChatReply)=>void;const pending=new Promise<ChatReply>(r=>resolve=r);const c=new HistoryController({list:async()=>[],get:id=>id==='a'?pending:Promise.resolve(reply(id))},()=>{});const a=c.open('a');await c.open('b');resolve(reply('a'));await a;expect(c.state.selected?.requestId).toBe('b');});
it('history pagination keeps at most twenty summaries',async()=>{const rows=Array.from({length:21},(_,i)=>({requestId:String(i),conversationId:'c',title:'q',model:'m',state:'complete' as const,createdAt:0}));const c=new HistoryController({list:async()=>rows,get:async()=>null},()=>{});await c.load(0);expect(c.state.items.length).toBe(20);expect(c.state.hasMore).toBe(true);});
