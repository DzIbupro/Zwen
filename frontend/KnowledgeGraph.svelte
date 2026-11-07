<script lang="ts">
 import {buildGraph,expandTopic,nodeRadius,nodeLabel,graphViewBox} from './lib/graph';
 import type {KnowledgeController,KnowledgeState} from './lib/knowledge';
 import type {GraphNode,GraphEdge} from './lib/graph';
 let {controller,info,onOpenNote,onOpenTopic}:{controller:KnowledgeController;info:KnowledgeState;onOpenNote:(id:string)=>void;onOpenTopic:(id:string)=>void}=$props();
 let selectedEdge=$state<GraphEdge|null>(null);
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
 function onWheel(e:WheelEvent){zoom=Math.min(5,Math.max(0.4,zoom*(e.deltaY>0?0.9:1.1)));}
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
  <button aria-label="放大知识网络" onclick={()=>zoom=Math.min(5,zoom*1.25)}>放大</button><button aria-label="缩小知识网络" onclick={()=>zoom=Math.max(.4,zoom/1.25)}>缩小</button>
  <button onclick={()=>{zoom=1;panX=0;panY=0;}}>复位视图</button>
 </div>
 <p class="graph-hint">每次最多显示 30 个主题、60 条主题关系；可搜索其他主题，点击主题展开最多 12 篇笔记。</p>
 {#if selectedEdge}
  <div data-relation-detail role="status"><p>关系原因：{selectedEdge.reason}</p><p>证据笔记：{selectedEdge.evidence||'未提供'}</p>{#if selectedEdge.evidence}<button data-relation-evidence onclick={()=>onOpenNote(selectedEdge!.evidence)}>打开证据笔记</button>{/if}<button onclick={()=>selectedEdge=null}>关闭关系详情</button></div>
 {/if}
 <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
 <div class="graph-stage" data-graph-stage
   onwheel={onWheel} onmousedown={onDown} onmousemove={onMove} onmouseup={onUp} onmouseleave={onUp}
   role="application" aria-label="主题关系网络（滚轮缩放，拖动平移）" tabindex="-1">
  <svg viewBox={graphViewBox(graph.nodes)} width="800" height="600"
    style={`transform:translate(${panX}px,${panY}px) scale(${zoom});transform-origin:center`}>
   {#each graph.edges as edge (edge.id)}
    {#if nodeIndex.get(edge.from)&&nodeIndex.get(edge.to)}
     <line x1={nodeIndex.get(edge.from)!.x} y1={nodeIndex.get(edge.from)!.y}
       x2={nodeIndex.get(edge.to)!.x} y2={nodeIndex.get(edge.to)!.y}
       class="edge" data-edge={edge.id} role="button" tabindex="0" aria-label={`查看关系：${edge.reason}`}
       onclick={e=>{e.stopPropagation();selectedEdge=edge;}} onkeydown={e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();selectedEdge=edge;}}}><title>{edge.reason} · 证据：{edge.evidence}</title></line>
    {/if}
   {/each}
   {#each graph.nodes as node (node.id)}
    <g class="node" data-graph-node={node.id} transform={`translate(${node.x},${node.y})`}
      role="button" tabindex="0" aria-label={node.name}
      onkeydown={e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();activateNode(node);}}}
      onclick={()=>activateNode(node)}>
     <rect class="node-hit" x="-92" y={-nodeRadius(node)-6} width="184" height={nodeRadius(node)*2+54}/>
     <circle r={nodeRadius(node)} class:locked={node.locked} class:note={node.id.startsWith('note:')}/>
     <title>{node.name}</title>
     <rect class="node-label-bg" x="-92" y={nodeRadius(node)+6} width="184" height={nodeLabel(node.name).length*18+8} rx="6"/>
     <text>{#each nodeLabel(node.name) as line,i}<tspan x="0" y={nodeRadius(node)+22+i*18}>{line}</tspan>{/each}</text>
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
 .graph-bar input{flex:1;border:1px solid var(--edge);border-radius:12px;padding:10px 12px;background:var(--surface);font:inherit;font-size:13px;color:var(--text)}
 .graph-bar button{border:1px solid var(--edge);background:transparent;color:var(--muted);border-radius:12px;padding:9px 13px;font:inherit;font-size:13px;cursor:pointer}
 .graph-hint{font-size:13px;color:var(--muted);white-space:nowrap}
 .graph-stage{border:1px solid var(--edge);border-radius:13px;background:var(--bg);overflow:hidden;cursor:grab;min-height:420px}
 .graph-stage:active{cursor:grabbing}
 svg{display:block;width:100%;height:600px}
 .edge{stroke:var(--edge);stroke-width:5;cursor:pointer;pointer-events:stroke}.edge:focus{stroke:var(--accent);outline:none}
 .node circle{fill:var(--surface-soft);stroke:var(--accent);stroke-width:2;cursor:pointer}
 .node circle.locked{stroke-dasharray:4 3}
 .node circle.note{fill:var(--surface);stroke:var(--edge);stroke-width:1.5}
 .node-hit{fill:transparent;pointer-events:all;cursor:pointer}.node-label-bg{fill:var(--bg);opacity:.96;pointer-events:none}.node text{font-size:13px;fill:var(--text);text-anchor:middle;pointer-events:none}
 .graph-empty{position:absolute;color:var(--muted);font-size:13px;padding:40px}

 button{transition:box-shadow .16s ease,background-color .16s ease;border-radius:12px;cursor:pointer;box-shadow:var(--raised-shadow)}
 button:hover:not(:disabled){box-shadow:var(--hover-shadow)}
 button:active:not(:disabled){box-shadow:var(--pressed-shadow)}
 button:focus-visible,input:focus-visible{outline:2px solid var(--focus-ring);outline-offset:3px}
 button:disabled{opacity:.5;cursor:default}
 @media(prefers-reduced-motion:reduce){button{transition:none}}
 .graphview{font-size:16px;line-height:1.65;color:var(--text)}.graph-bar{flex-wrap:wrap}.graph-bar input{min-width:120px;font-size:16px;box-shadow:var(--inset-shadow)}.graph-bar button,button{font-size:14px;border:1px solid var(--edge);background:var(--surface);color:var(--text);padding:9px 13px}.graph-stage{box-shadow:var(--inset-shadow)}.node text{font-size:14px}.edge:focus-visible{stroke:var(--accent);stroke-width:7}.node circle:focus-visible{outline:2px solid var(--focus-ring);outline-offset:4px}[data-relation-detail]{margin-bottom:16px}[data-relation-detail] p{margin:8px 0;overflow-wrap:anywhere}@media(max-width:520px){.graphview{padding-inline:0}.graph-stage{min-height:340px}svg{height:440px}.graph-hint{white-space:normal}}
</style>
