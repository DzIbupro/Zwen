import {it,expect,afterEach} from 'vitest';
import {AttachmentController,type AttachmentItem,type AttachmentPort,type AttachmentView} from '../lib/attachments';
const cleanups:(()=>void)[]=[];afterEach(()=>cleanups.splice(0).forEach(f=>f()));
function item(id='a'.repeat(32)):AttachmentItem{return{id,width:320,height:200,bytes:2048,createdAt:1,thumbnail:'data:image/png;base64,AA==',error:null};}
function view(items:AttachmentItem[]=[],pending=false,notice:string|null=null):AttachmentView{return{items,capture:{state:pending?'pending':'idle',requestId:pending?'r':null,replaceId:null},notice,max:3};}
function fixture(initial=view()){
 let current=initial;let listener:()=>void=()=>{};let listens=0;let captures:Array<string|null>=[];
 const port:AttachmentPort={
  list:async()=>({...current,items:current.items.map(x=>({...x})),capture:{...current.capture}}),
  capture:async replace=>{captures.push(replace??null);current={...current,capture:{state:'pending',requestId:'r',replaceId:replace??null}};},
  cancel:async()=>{current={...current,capture:{state:'idle',requestId:null,replaceId:null}};},
  remove:async id=>{current={...current,items:current.items.filter(x=>x.id!==id)};},
  listen:async cb=>{listens++;listener=cb;return()=>{listener=()=>{};};},
 };
 const controller=new AttachmentController(port,()=>{});cleanups.push(()=>controller.dispose());
 return {controller,port,get current(){return current;},set current(next:AttachmentView){current=next;},notify(){listener();},get listens(){return listens;},get captures(){return captures;}};
}
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));

it('loads bounded attachment metadata and the event listener',async()=>{
 const f=fixture(view([item()]));await f.controller.start();
 expect(f.controller.state.loaded).toBe(true);
 expect(f.controller.state.items).toHaveLength(1);
 expect(f.controller.state.max).toBe(3);
 expect(f.listens).toBe(1);
});

it('captures once, shows pending, and adopts the imported item after an event',async()=>{
 const f=fixture();await f.controller.start();
 const pending=f.controller.capture();
 expect(f.controller.state.pending).toBe(true);
 await pending;
 expect(f.captures).toEqual([null]);
 f.current=view([item('b'.repeat(32))]);f.notify();await tick();
 expect(f.controller.state.pending).toBe(false);
 expect(f.controller.state.items[0].id).toBe('b'.repeat(32));
});

it('does not start a second capture while one is pending',async()=>{
 const f=fixture(view([],true));await f.controller.start();
 await f.controller.capture();
 expect(f.captures).toEqual([]);
});

it('retake passes the replaced id and keeps the old item on cancellation',async()=>{
 const old=item('c'.repeat(32));const f=fixture(view([old]));await f.controller.start();
 await f.controller.capture(old.id);
 expect(f.captures).toEqual([old.id]);
 await f.controller.cancel();await tick();
 expect(f.controller.state.items.map(x=>x.id)).toEqual([old.id]);
 expect(f.controller.state.pending).toBe(false);
});

it('removes only the chosen attachment and refreshes',async()=>{
 const first=item('d'.repeat(32));const second=item('e'.repeat(32));const f=fixture(view([first,second]));await f.controller.start();
 await f.controller.remove(first.id);
 expect(f.controller.state.items.map(x=>x.id)).toEqual([second.id]);
});

it('removeSent consumes accepted ids and tolerates concurrent manual deletes',async()=>{
 const first=item('1'.repeat(32));const second=item('2'.repeat(32));const f=fixture(view([first,second]));await f.controller.start();
 // A concurrent manual delete already removed it server-side before removeSent ran.
 f.current={...f.current,items:[second]};
 const original=f.port.remove;
 f.port.remove=async id=>{if(id===first.id)throw Error('截图不存在');return original(id);};
 await f.controller.removeSent([first.id,'9'.repeat(32)]);
 expect(f.controller.state.items.map(x=>x.id)).toEqual([second.id]);
});

it('a failed list is visible without losing a loaded state',async()=>{
 const f=fixture();await f.controller.start();
 f.port.list=async()=>{throw Error('本地数据库暂不可用');};
 await f.controller.load();
 expect(f.controller.state.loaded).toBe(true);
 expect(f.controller.state.notice).toContain('本地数据库暂不可用');
});

it('a late list result cannot update a disposed controller',async()=>{
 const f=fixture();let release!:(value:AttachmentView)=>void;
 f.port.list=()=>new Promise<AttachmentView>(resolve=>release=resolve);
 const loading=f.controller.load();f.controller.dispose();release(view([item()]));await loading;
 expect(f.controller.state.items).toEqual([]);
});
