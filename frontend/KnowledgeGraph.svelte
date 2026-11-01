<script lang="ts">
 import {buildGraph,expandTopic} from './lib/graph';
 import type {KnowledgeController,KnowledgeState} from './lib/knowledge';
 import type {GraphNode} from './lib/graph';
 let {controller,info,onOpenNote,onOpenTopic}:{controller:KnowledgeController;info:KnowledgeState;onOpenNote:(id:string)=>void;onOpenTopic:(id:string)=>void}=$props();
 let filter=$state('');
 let zoom=$state(1);
 let panX=$state(0);
 let panY=$state(0);
 let dragging=$state<{x:number;y:number;px:number;py:number}|null>(null);
 let expanded=$state<Record<string,boolean>>({});
 const notesOf=(topicId:string)=>{
  const tv=info.topicView&&info.topicView.id===topicId?info.topicView:null;
  return tv?tv.notes.map(n=>({id:n.meta.id,title:n.meta.title})):[];
 };
 const graph=$derived.by(()=>{
  let g=buildGraph(info.topics,info.relations,filter);
  for(const id of Object.keys(expanded))if(expanded[id]){
   const tv=info.topicView&&info.topicView.id===id?info.topicView:null;
   if(tv)g=expandTopic(g,id,tv.notes.map(n=>({id:n.meta.id,title:n.meta.title})));
  }
  return g;
 });
 const nodeIndex=$derived(new Map(graph.nodes.map(n=>[n.id,n])));
 function toggleExpand(id:string){expanded={...expanded,[id]:!expanded[id]};}
 function onWheel(e:WheelEvent){zoom=Math.min(3,Math.max(0.4,zoom*(e.deltaY>0?0.9:1.1)));}
 function onDown(e:MouseEvent){dragging={x:e.clientX,y:e.clientY,px:panX,py:panY};}
 function onMove(e:MouseEvent){if(!dragging)return;panX=dragging.px+(e.clientX-dragging.x);panY=dragging.py+(e.clientY-dragging.y);}
 function onUp(){dragging=null;}
 function activateNode(node:GraphNode){
  if(node.id.startsWith('note:')){onOpenNote(node.id.split(':')[2]);return;}
  toggleExpand(node.id);onOpenTopic(node.id);
 }
</script>
<section class="graphview" data-graph aria-label="知识网络">
 <div class="graph-bar">
  <input data-graph-search type="search" placeholder="筛选主题…" value={filter} oninput={e=>filter=e.currentTarget.value}/>
  <span class="graph-hint">{graph.nodes.length} 节点 · {graph.edges.length} 关系 · 滚轮缩放 / 拖动平移</span>
  <button onclick={()=>{zoom=1;panX=0;panY=0;}}>复位视图</button>
 </div>
 <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
 <div class="graph-stage" data-graph-stage
   onwheel={onWheel} onmousedown={onDown} onmousemove={onMove} onmouseup={onUp} onmouseleave={onUp}
   role="application" aria-label="主题关系网络（滚轮缩放，拖动平移）" tabindex="-1">
  <svg viewBox="0 0 800 600" width="800" height="600"
    style={`transform:translate(${panX}px,${panY}px) scale(${zoom});transform-origin:center`}>
   {#each graph.edges as edge (edge.id)}
    {#if nodeIndex.get(edge.from)&&nodeIndex.get(edge.to)}
     <line x1={nodeIndex.get(edge.from)!.x} y1={nodeIndex.get(edge.from)!.y}
       x2={nodeIndex.get(edge.to)!.x} y2={nodeIndex.get(edge.to)!.y}
       class="edge" data-edge={edge.id}><title>{edge.reason}</title></line>
    {/if}
   {/each}
   {#each graph.nodes as node (node.id)}
    <g class="node" data-graph-node={node.id} transform={`translate(${node.x},${node.y})`}
      role="button" tabindex="0"
      onkeydown={e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();activateNode(node);}}}
      onclick={()=>activateNode(node)}>
     <circle r={node.id.startsWith('note:')?10:14+Math.min(node.count,10)*2} class:locked={node.locked} class:note={node.id.startsWith('note:')}/>
     <text y={node.id.startsWith('note:')?-16:24}>{node.name}</text>
    </g>
   {/each}
  </svg>
  {#if graph.nodes.length===0}
   <p class="graph-empty">还没有主题。先在知识库中整理出主题，这里会长成你的知识网络。</p>
  {/if}
 </div>
</section>
<style>
 .graphview{max-width:900px;margin:22px auto;padding:0 18px 28px}
 .graph-bar{display:flex;gap:10px;align-items:center;margin-bottom:12px}
 .graph-bar input{flex:1;border:1px solid #d7dece;border-radius:9px;padding:10px 12px;background:#fffef9;font:inherit;font-size:13px;color:#333d36}
 .graph-bar button{border:1px solid #d5ddcb;background:transparent;color:#55634b;border-radius:8px;padding:9px 13px;font:inherit;font-size:12px;cursor:pointer}
 .graph-hint{font-size:11px;color:#8b967e;white-space:nowrap}
 .graph-stage{border:1px solid #dce2d2;border-radius:13px;background:linear-gradient(180deg,#fbfcf6,#f2f4ea);overflow:hidden;cursor:grab;min-height:420px}
 .graph-stage:active{cursor:grabbing}
 svg{display:block}
 .edge{stroke:#a9ba97;stroke-width:1.6}
 .node circle{fill:#e7eddf;stroke:#417f65;stroke-width:2;cursor:pointer}
 .node circle.locked{stroke-dasharray:4 3}
 .node circle.note{fill:#fffef9;stroke:#8fa983;stroke-width:1.5}
 .node text{font-size:12px;fill:#333d36;text-anchor:middle;pointer-events:none}
 .graph-empty{position:absolute;color:#697663;font-size:13px;padding:40px}
</style>
