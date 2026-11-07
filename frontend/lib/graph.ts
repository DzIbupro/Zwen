/// Local knowledge-network view: topics are nodes (size = member count),
/// relations are edges, notes are reachable per topic on demand. Pure SVG —
/// no graph dependency; layout is a deterministic circle with jitter-free ids.
export interface GraphNode {id:string;name:string;count:number;locked:boolean;x:number;y:number}
export interface GraphEdge {id:string;from:string;to:string;reason:string;evidence:string;kind?:'relation'|'membership'}
export interface GraphData {nodes:GraphNode[];edges:GraphEdge[]}

import type {TopicView,RelationView} from './knowledge';

export function buildGraph(topics:TopicView[],relations:RelationView[],filter:string):GraphData{
 const query=filter.trim();
 const visible=topics.filter(t=>!query||t.topic.name.includes(query)).slice(0,30);
 const ids=new Set(visible.map(t=>t.topic.id));
 const n=visible.length;
 const cx=400,cy=300,radius=Math.max(160,n*224/(2*Math.PI));
 const nodes:GraphNode[]=visible.map((t,i)=>{
  const angle=(2*Math.PI*i)/Math.max(n,1)-Math.PI/2;
  return{id:t.topic.id,name:t.topic.name,count:t.count,locked:t.topic.locked,
   x:Math.round(cx+radius*Math.cos(angle)),y:Math.round(cy+radius*Math.sin(angle))};
 });
 const edges:GraphEdge[]=relations
  .filter(r=>ids.has(r.fromTopic)&&ids.has(r.toTopic))
  .slice(0,60)
  .map(r=>({id:r.id,from:r.fromTopic,to:r.toTopic,reason:r.reason,evidence:r.evidence,kind:'relation'}));
 return{nodes:separateNodes(nodes),edges};
}

/// Expands a topic into its member notes (secondary nodes attached around it).
export function expandTopic(graph:GraphData,topicId:string,notes:{id:string;title:string}[]):GraphData{
 const host=graph.nodes.find(node=>node.id===topicId);
 if(!host||notes.length===0)return graph;
 const nodes=graph.nodes.filter(node=>!node.id.startsWith(`note:${topicId}:`));
 const added=notes.slice(0,12).map((note,i)=>{
  const angle=(2*Math.PI*i)/Math.min(notes.length,12)+Math.PI/8;
  return{id:`note:${topicId}:${note.id}`,name:note.title,count:0,locked:false,
   x:Math.round(host.x+Math.max(140,Math.min(notes.length,12)*224/(2*Math.PI))*Math.cos(angle)),y:Math.round(host.y+Math.max(140,Math.min(notes.length,12)*224/(2*Math.PI))*Math.sin(angle))};
 });
 const edges=graph.edges.filter(edge=>!edge.to.startsWith(`note:${topicId}:`));
 return{nodes:separateNodes([...nodes,...added]),edges:[...edges,...added.map(node=>({id:`member:${node.id}`,from:topicId,to:node.id,reason:'主题收录笔记',evidence:notes.find(note=>node.id===`note:${topicId}:${note.id}`)!.id,kind:'membership' as const}))]};
}

export function nodeRadius(node:GraphNode){return node.id.startsWith('note:')?10:14+Math.min(node.count,10)*2;}
export function nodeLabel(name:string):string[]{const chars=Array.from(name);return [chars.slice(0,12).join(''),...(chars.length>12?[chars.slice(12,23).join('')+(chars.length>24?'…':chars[23]??'')]:[])];}
function overlaps(a:GraphNode,b:GraphNode){return Math.abs(a.x-b.x)<184&&a.y-nodeRadius(a)-6<b.y+nodeRadius(b)+48&&b.y-nodeRadius(b)-6<a.y+nodeRadius(a)+48;}
/// Reserve the whole circle + two-line label, not just a node's center.
/// Deterministic nearest free grid slot keeps expansion bounded and jitter-free.
function separateNodes(input:GraphNode[]):GraphNode[]{
 const placed:GraphNode[]=[];
 for(const original of input){let node={...original};let found=!placed.some(other=>overlaps(node,other));
  for(let ring=1;!found&&ring<=input.length+1;ring++){
   for(let dy=-ring;dy<=ring&&!found;dy++)for(let dx=-ring;dx<=ring&&!found;dx++){
    if(Math.max(Math.abs(dx),Math.abs(dy))!==ring)continue;
    const candidate={...original,x:original.x+dx*200,y:original.y+dy*144};
    if(!placed.some(other=>overlaps(candidate,other))){node=candidate;found=true;}
   }
  }
  placed.push(node);
 }
 return placed;
}
export function graphViewBox(nodes:GraphNode[]):string{
 if(!nodes.length)return '0 0 800 600';
 const left=Math.min(...nodes.map(n=>n.x-100)),right=Math.max(...nodes.map(n=>n.x+100));
 const top=Math.min(...nodes.map(n=>n.y-nodeRadius(n)-24)),bottom=Math.max(...nodes.map(n=>n.y+nodeRadius(n)+64));
 const width=Math.max(400,right-left),height=Math.max(300,bottom-top);
 return `${(left+right-width)/2} ${(top+bottom-height)/2} ${width} ${height}`;
}
