import {it,expect,vi} from 'vitest';
import {mount,unmount,flushSync} from 'svelte';
vi.mock('svelte',()=>vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import KnowledgeGraph from '../KnowledgeGraph.svelte';
import {KnowledgeController,type KnowledgePort} from '../lib/knowledge';
function setup(columns=['甲','甲','乙']){
 const controller=new KnowledgeController({} as KnowledgePort,()=>{});
 controller.state.topics=columns.map((column,i)=>({topic:{id:`t${i}`,name:`主题${i}`,column,createdBy:'ai',batchId:null,createdAt:1,locked:false},count:1,noteIds:['n1']}));
 controller.state.relations=[{id:'r1',fromTopic:'t0',toTopic:'t2',reason:'真实关系',evidence:'n1',addedBy:'ai'}];
 const target=document.createElement('div');document.body.append(target);const onOpenTopic=vi.fn(),onOpenNote=vi.fn();
 const app=mount(KnowledgeGraph,{target,props:{controller,info:controller.state,onOpenNote,onOpenTopic}});flushSync();
 return{target,app,controller,onOpenTopic,onOpenNote};
}
function pointer(target:Element,type:string,x:number,y:number){
 const event=new MouseEvent(type,{bubbles:true,clientX:x,clientY:y,button:0});
 Object.defineProperties(event,{pointerId:{value:1},isPrimary:{value:true}});target.dispatchEvent(event);flushSync();
}
it('drills actual column assignments and keeps keyboard activation and evidence',async()=>{
 const {target,app,controller,onOpenTopic,onOpenNote}=setup();
 expect(target.querySelectorAll('[data-graph-node]')).toHaveLength(2);
 expect(target.querySelector('rect')).toBeNull();
 target.querySelector('[data-edge=r1]')!.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,key:'Enter'}));flushSync();
 (target.querySelector('[data-relation-evidence]') as HTMLButtonElement).click();expect(onOpenNote).toHaveBeenCalledWith('n1');
 target.querySelector('[data-graph-node="column:甲"]')!.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,key:'Enter'}));flushSync();
 expect(Array.from(target.querySelectorAll('[data-graph-node]')).map(n=>n.getAttribute('data-graph-node'))).toEqual(expect.arrayContaining(['t0','t1']));
 target.querySelector('[data-graph-node=t0]')!.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,key:' '}));flushSync();expect(onOpenTopic).toHaveBeenCalledWith('t0');
 await unmount(app);target.remove();controller.dispose();
});
it('shows unfiled topics directly and prevents drag activation, with cancel recovery',async()=>{
 const {target,app,controller,onOpenTopic}=setup(['','','']);
 expect(target.querySelector('[data-graph-node=t0]')).not.toBeNull();
 const stage=target.querySelector('[data-graph-stage]') as HTMLElement;
 let captured=false;stage.hasPointerCapture=()=>captured;stage.setPointerCapture=()=>{captured=true;};stage.releasePointerCapture=()=>{captured=false;};
 const node=target.querySelector('[data-graph-node=t0]')!;const before=node.getAttribute('transform');
 pointer(node,'pointerdown',10,10);pointer(stage,'pointermove',80,40);pointer(stage,'pointerup',80,40);
 expect(node.getAttribute('transform')).not.toBe(before);expect(captured).toBe(false);
 node.dispatchEvent(new MouseEvent('click',{bubbles:true,detail:1}));flushSync();expect(onOpenTopic).not.toHaveBeenCalled();
 pointer(node,'pointerdown',10,10);pointer(stage,'pointercancel',10,10);expect(captured).toBe(false);
 pointer(node,'pointerdown',10,10);pointer(node,'pointerup',10,10);node.dispatchEvent(new MouseEvent('click',{bubbles:true,detail:1}));flushSync();expect(onOpenTopic).toHaveBeenCalledWith('t0');
 const wheel=new WheelEvent('wheel',{bubbles:true,cancelable:true,deltaY:-150});stage.dispatchEvent(wheel);flushSync();expect(wheel.defaultPrevented).toBe(true);
 await unmount(app);target.remove();controller.dispose();
});

it('compensates SVG fit scaling to keep default and focused labels readable',async()=>{
 let disconnects=0;
 vi.stubGlobal('ResizeObserver',class {
  constructor(private callback:(entries:{contentRect:{width:number;height:number}}[])=>void){}
  observe(){this.callback([{contentRect:{width:400,height:300}}]);}
  disconnect(){disconnects++;}
 });
 const {target,app,controller}=setup();
 try{
  await Promise.resolve();flushSync();
  expect((target.querySelector('.node text') as SVGElement).style.fontSize).toBe('24px');
  target.querySelector('[data-graph-node="column:甲"]')!.dispatchEvent(new FocusEvent('focus'));flushSync();
  expect((target.querySelector('[data-graph-node="column:甲"] text') as SVGElement).style.fontSize).toBe('28px');
 }finally{await unmount(app);target.remove();controller.dispose();vi.unstubAllGlobals();}
 expect(disconnects).toBe(1);
});

it('clears column-only search when drilling into its topics',async()=>{
 const {target,app,controller}=setup(['研究方法','研究方法','其他']);
 const search=target.querySelector('[data-graph-search]') as HTMLInputElement;
 search.value='研究方法';search.dispatchEvent(new Event('input',{bubbles:true}));flushSync();
 target.querySelector('[data-graph-node="column:研究方法"]')!.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,key:'Enter'}));flushSync();
 expect(search.value).toBe('');expect(target.querySelectorAll('[data-graph-node]')).toHaveLength(2);
 await unmount(app);target.remove();controller.dispose();
});
