/// Local knowledge-network view: topics are nodes (size = member count),
/// relations are edges, notes are reachable per topic on demand. Pure SVG —
/// no graph dependency; layout is a deterministic circle with jitter-free ids.
export interface GraphNode {id:string;name:string;count:number;locked:boolean;x:number;y:number}
export interface GraphEdge {id:string;from:string;to:string;reason:string}
export interface GraphData {nodes:GraphNode[];edges:GraphEdge[]}

import type {TopicView,RelationView} from './knowledge';

export function buildGraph(topics:TopicView[],relations:RelationView[],filter:string):GraphData{
 const query=filter.trim();
 const visible=topics.filter(t=>!query||t.topic.name.includes(query));
 const ids=new Set(visible.map(t=>t.topic.id));
 const n=visible.length;
 const cx=400,cy=300,radius=Math.min(260,120+n*28);
 const nodes:GraphNode[]=visible.map((t,i)=>{
  const angle=(2*Math.PI*i)/Math.max(n,1)-Math.PI/2;
  return{id:t.topic.id,name:t.topic.name,count:t.count,locked:t.topic.locked,
   x:Math.round(cx+radius*Math.cos(angle)),y:Math.round(cy+radius*Math.sin(angle))};
 });
 const nodeIndex=new Map(nodes.map(node=>[node.id,node]));
 const edges:GraphEdge[]=relations
  .filter(r=>ids.has(r.fromTopic)&&ids.has(r.toTopic))
  .map(r=>({id:r.id,from:r.fromTopic,to:r.toTopic,reason:r.reason}));
 return{nodes,edges};
}

/// Expands a topic into its member notes (secondary nodes attached around it).
export function expandTopic(graph:GraphData,topicId:string,notes:{id:string;title:string}[]):GraphData{
 const host=graph.nodes.find(node=>node.id===topicId);
 if(!host||notes.length===0)return graph;
 const nodes=graph.nodes.filter(node=>!node.id.startsWith(`note:${topicId}:`));
 const base=nodes.findIndex(node=>node.id===topicId);
 const added=notes.slice(0,12).map((note,i)=>{
  const angle=(2*Math.PI*i)/Math.min(notes.length,12)+Math.PI/8;
  return{id:`note:${topicId}:${note.id}`,name:note.title,count:0,locked:false,
   x:Math.round(host.x+90*Math.cos(angle)),y:Math.round(host.y+90*Math.sin(angle))};
 });
 return{nodes:[...nodes,...added],edges:graph.edges};
}
