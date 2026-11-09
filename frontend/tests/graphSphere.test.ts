import {expect,it} from 'vitest';
import {spherePoint,rotatePoint,projectSphere,dragRotation,isRotationDrag,clampZoom,buildColumnGraph} from '../lib/graphSphere';
import type {TopicView} from '../lib/knowledge';
it('places deterministic equal-radius points across front and rear hemispheres',()=>{
 const points=Array.from({length:30},(_,i)=>spherePoint(i,30));
 expect(points.every(p=>Math.abs(Math.hypot(p.x,p.y,p.z)-1)<1e-10)).toBe(true);
 expect(points.some(p=>p.z<-.5)).toBe(true);expect(points.some(p=>p.z>.5)).toBe(true);
 expect(spherePoint(2,30)).toEqual(points[2]);
});
it('rotates without changing radius and projects with depth and bounded labels',()=>{
 const p=rotatePoint({x:1,y:0,z:0},Math.PI/2,0);expect(p.z).toBeCloseTo(-1);
 const nodes=Array.from({length:30},(_,i)=>({id:String(i),name:'主题',count:1,locked:false,x:0,y:0}));
 const view=projectSphere(nodes,0,.2,1);
 expect(view.filter(n=>n.label)).toHaveLength(6);
 expect(view.every(n=>Number.isFinite(n.x)&&Number.isFinite(n.y))).toBe(true);
 expect(view[0].scale).toBeLessThan(view[29].scale);
 expect(view.map(n=>n.depth)).toEqual([...view.map(n=>n.depth)].sort((a,b)=>a-b));
});
it('limits zoom and tilt and distinguishes a click from a rotation gesture',()=>{
 expect(clampZoom(100)).toBe(2.8);expect(clampZoom(.01)).toBe(.5);
 expect(dragRotation(0,0,10,1000).pitch).toBe(-Math.PI/2);
 expect(isRotationDrag(2,2)).toBe(false);expect(isRotationDrag(6,0)).toBe(true);
});
it('aggregates only real cross-column edges and preserves original evidence',()=>{
 const topic=(id:string,column:string):TopicView=>({topic:{id,column,name:id,locked:false,createdBy:'ai',batchId:null,createdAt:1},noteIds:[],count:1});
 const topics=[topic('a','甲'),topic('b','甲'),topic('c','乙'),topic('d','')];
 const relation=(id:string,fromTopic:string,toTopic:string)=>({id,fromTopic,toTopic,reason:'真实关系',evidence:'note1',addedBy:'ai'});
 const graph=buildColumnGraph(topics,[relation('internal','a','b'),relation('cross','a','c')],'');
 expect(graph.nodes.map(n=>[n.name,n.count])).toEqual([['甲',2],['乙',1],['未分栏',1]]);
 expect(graph.edges).toEqual([{id:'cross',from:'column:甲',to:'column:乙',reason:'真实关系',evidence:'note1',kind:'relation'}]);
 expect(buildColumnGraph(topics,[],'a').nodes.map(n=>n.name)).toEqual(['甲']);
});
it('projects decorative sphere guides with the same rotation and zoom as nodes',async()=>{
 const {sphereGuides,projectPoint}=await import('../lib/graphSphere');
 const guides=sphereGuides(0,0,1);
 expect(guides).toHaveLength(12);expect(guides.some(g=>g.front&&g.path)).toBe(true);expect(guides.some(g=>!g.front&&g.path)).toBe(true);
 expect(guides.every(g=>!g.path.includes('NaN'))).toBe(true);
 expect(sphereGuides(.5,.3,1)).not.toEqual(guides);
 const near=projectPoint({x:1,y:0,z:0},0,0,1),zoomed=projectPoint({x:1,y:0,z:0},0,0,2);
 expect(zoomed.x-400).toBeCloseTo((near.x-400)*2);
});
