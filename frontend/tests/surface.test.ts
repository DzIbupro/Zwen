import {it,expect} from 'vitest';
import {boundedHeight,canApplySurface} from '../lib/surface';
it('clamps content and malformed heights',()=>{expect(boundedHeight(50,96,500)).toBe(96);expect(boundedHeight(900,96,500)).toBe(500);expect(boundedHeight(NaN,96,500)).toBe(96);});
it('ignores older mode events',()=>{expect(canApplySurface(4,3)).toBe(false);expect(canApplySurface(4,4)).toBe(true);expect(canApplySurface(4,5)).toBe(true);});
import {ResizeQueue} from '../lib/surface';
it('serializes native resizing and keeps the latest queued height',async()=>{
 let release!:()=>void;const gate=new Promise<void>(r=>release=r);let physicalHeight=96;const applied:number[]=[];
 const q=new ResizeQueue(async(height)=>{if(height===100)await gate;physicalHeight=height;applied.push(height);},()=>{});
 q.request(100,1);q.request(200,1);q.request(300,1);release();await new Promise(r=>setTimeout(r,0));
 expect(physicalHeight).toBe(300);expect(applied).toEqual([100,300]);
});
