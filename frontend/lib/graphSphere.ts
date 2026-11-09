import type {GraphData, GraphNode} from './graph';
import type {TopicView, RelationView} from './knowledge';

export interface SpherePoint {x:number;y:number;z:number}
export interface ProjectedNode extends GraphNode {depth:number;scale:number;opacity:number;label:boolean}
export const INITIAL_YAW=-0.35;
export const INITIAL_PITCH=0.18;
export function clampZoom(value:number){return Math.max(.5,Math.min(2.8,value));}
export function dragRotation(yaw:number,pitch:number,dx:number,dy:number){
 return {yaw:yaw+dx*.008,pitch:Math.max(-Math.PI/2,Math.min(Math.PI/2,pitch-dy*.008))};
}
export function isRotationDrag(dx:number,dy:number){return Math.hypot(dx,dy)>5;}
/** Equal-area Fibonacci placement spreads nodes over a sphere, including its rear. */
export function spherePoint(index:number,count:number):SpherePoint{
 if(count===1)return{x:0,y:0,z:1};
 const y=1-2*(index+.5)/Math.max(1,count),r=Math.sqrt(1-y*y),angle=index*Math.PI*(3-Math.sqrt(5));
 return{x:Math.cos(angle)*r,y,z:Math.sin(angle)*r};
}
export function rotatePoint(point:SpherePoint,yaw:number,pitch:number):SpherePoint{
 const x=point.x*Math.cos(yaw)+point.z*Math.sin(yaw),z=-point.x*Math.sin(yaw)+point.z*Math.cos(yaw);
 return{x,y:point.y*Math.cos(pitch)-z*Math.sin(pitch),z:point.y*Math.sin(pitch)+z*Math.cos(pitch)};
}
export function projectPoint(point:SpherePoint,yaw:number,pitch:number,zoom:number){
 const p=rotatePoint(point,yaw,pitch),scale=900/(900-p.z*205);
 return{x:400+p.x*205*scale*zoom,y:300+p.y*205*scale*zoom,depth:p.z,scale:scale*zoom};
}
/** Decorative latitude/longitude grid; these paths carry no knowledge relations. */
export function sphereGuides(yaw:number,pitch:number,zoom:number):{path:string;front:boolean}[]{
 const circles:((angle:number)=>SpherePoint)[]=[];
 for(const longitude of [0,Math.PI/3,2*Math.PI/3])circles.push(angle=>({x:Math.cos(angle)*Math.cos(longitude),y:Math.sin(angle),z:Math.cos(angle)*Math.sin(longitude)}));
 for(const latitude of [-Math.PI/6,0,Math.PI/6])circles.push(angle=>({x:Math.cos(latitude)*Math.cos(angle),y:Math.sin(latitude),z:Math.cos(latitude)*Math.sin(angle)}));
 return circles.flatMap(circle=>{
  const paths=['',''];
  for(let i=0;i<72;i++){
   const a=projectPoint(circle(i*Math.PI/36),yaw,pitch,zoom),b=projectPoint(circle((i+1)*Math.PI/36),yaw,pitch,zoom);
   const front=(a.depth+b.depth)/2>=0?1:0;
   paths[front]+=`M${a.x.toFixed(2)},${a.y.toFixed(2)}L${b.x.toFixed(2)},${b.y.toFixed(2)}`;
  }
  return paths.map((path,i)=>({path,front:i===1}));
 });
}
export function projectSphere(nodes:GraphNode[],yaw:number,pitch:number,zoom:number):ProjectedNode[]{
 const projected=nodes.map((node,i)=>{
  const p=projectPoint(spherePoint(i,nodes.length),yaw,pitch,zoom);
  return{...node,...p,opacity:.28+(p.depth+1)*.36,label:false};
 });
 const labeled=new Set([...projected].sort((a,b)=>b.depth-a.depth).slice(0,Math.min(6,projected.length)).map(n=>n.id));
 return projected.map(n=>({...n,label:labeled.has(n.id)})).sort((a,b)=>a.depth-b.depth);
}
export function columnName(topic:TopicView){return topic.topic.column.trim()||'未分栏';}
export function columnNodeId(column:string){return `column:${column}`;}
/** Columns summarize existing topic assignments; edges retain the original evidence. */
export function buildColumnGraph(topics:TopicView[],relations:RelationView[],filter:string):GraphData{
 const query=filter.trim().toLocaleLowerCase();
 const visible=topics.filter(t=>!query||`${columnName(t)} ${t.topic.name}`.toLocaleLowerCase().includes(query));
 const groups=new Map<string,TopicView[]>();
 for(const topic of visible){const name=columnName(topic);groups.set(name,[...(groups.get(name)||[]),topic]);}
 const columns=[...groups].slice(0,30),ids=new Map<string,string>();
 const nodes=columns.map(([name,members])=>{
  for(const member of members)ids.set(member.topic.id,columnNodeId(name));
  return{id:columnNodeId(name),name,count:members.length,locked:false,x:0,y:0};
 });
 const edges=relations.filter(r=>ids.has(r.fromTopic)&&ids.has(r.toTopic)&&ids.get(r.fromTopic)!==ids.get(r.toTopic)).slice(0,60)
  .map(r=>({id:r.id,from:ids.get(r.fromTopic)!,to:ids.get(r.toTopic)!,reason:r.reason,evidence:r.evidence,kind:'relation' as const}));
 return{nodes,edges};
}
