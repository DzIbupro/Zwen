import {it,expect} from 'vitest';
import {buildGraph,expandTopic} from '../lib/graph';
import type {TopicView,RelationView} from '../lib/knowledge';
function topic(id:string,name:string,count:number,locked=false):TopicView{
 return{topic:{id,name,column:'',createdBy:'ai',batchId:null,createdAt:1,locked},noteIds:[],count};
}
const topics=[topic('t1','正则化',2),topic('t2','泛化',1,true),topic('t3','无关',0)];
const relations:RelationView[]=[{id:'r1',fromTopic:'t1',toTopic:'t2',reason:'同源',evidence:'n1',addedBy:'ai'}];
it('builds nodes on a circle and keeps only in-scope edges',()=>{
 const graph=buildGraph(topics,relations,'');
 expect(graph.nodes).toHaveLength(3);
 expect(graph.edges).toHaveLength(1);
 expect(graph.edges[0].from).toBe('t1');
 expect(graph.nodes.every(n=>Number.isFinite(n.x)&&Number.isFinite(n.y))).toBe(true);
});
it('filter narrows nodes and drops their edges',()=>{
 const graph=buildGraph(topics,relations,'正则');
 expect(graph.nodes.map(n=>n.name)).toEqual(['正则化']);
 expect(graph.edges).toHaveLength(0);
});
it('expand attaches bounded secondary note nodes near the host',()=>{
 let graph=buildGraph(topics,relations,'');
 const notes=[...Array(20).keys()].map(i=>({id:`n${i}`,title:`笔记${i}`}));
 graph=expandTopic(graph,'t1',notes);
 const attached=graph.nodes.filter(n=>n.id.startsWith('note:t1:'));
 expect(attached).toHaveLength(12);
 expect(graph.nodes.find(n=>n.id==='t1')).toBeDefined();
});
it('expand is idempotent per topic',()=>{
 let graph=buildGraph(topics,relations,'');
 graph=expandTopic(graph,'t1',[{id:'n1',title:'笔记'}]);
 graph=expandTopic(graph,'t1',[{id:'n1',title:'笔记'}]);
 expect(graph.nodes.filter(n=>n.id.startsWith('note:t1:'))).toHaveLength(1);
});

it('bounds initial topics and edges and retains relation evidence',()=>{
 const many=Array.from({length:100},(_,i)=>topic(`t${i}`,`主题${i}`,1));
 const rels=Array.from({length:200},(_,i)=>({...relations[0],id:`r${i}`,fromTopic:'t0',toTopic:'t1'}));
 const graph=buildGraph(many,rels,'');
 expect(graph.nodes.length).toBeLessThanOrEqual(30);expect(graph.edges.length).toBeLessThanOrEqual(60);
 expect(graph.edges[0].evidence).toBe('n1');
});
it('expanded notes have idempotent links to their parent',()=>{
 let graph=expandTopic(buildGraph(topics,relations,''),'t1',[{id:'n1',title:'笔记'}]);
 graph=expandTopic(graph,'t1',[{id:'n1',title:'笔记'}]);
 expect(graph.edges.filter(e=>e.from==='t1'&&e.to==='note:t1:n1')).toHaveLength(1);
});
