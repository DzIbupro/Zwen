<script lang="ts">
 import {onMount} from 'svelte';
 import {buildGraph,expandTopic,nodeRadius} from './lib/graph';
 import {buildColumnGraph,columnName,columnNodeId,projectSphere,sphereGuides,clampZoom,dragRotation,isRotationDrag,INITIAL_YAW,INITIAL_PITCH} from './lib/graphSphere';
 import type {KnowledgeController,KnowledgeState} from './lib/knowledge';
 import type {GraphNode,GraphEdge} from './lib/graph';
 let {controller,info,onOpenNote,onOpenTopic}:{controller:KnowledgeController;info:KnowledgeState;onOpenNote:(id:string)=>void;onOpenTopic:(id:string)=>void}=$props();
 let stageElement:HTMLDivElement;
 let fitScale=$state(1);
 onMount(()=>{
  const update=(width:number,height:number)=>{if(width>0&&height>0)fitScale=Math.min(width/800,height/600);};
  const rect=stageElement.getBoundingClientRect();update(rect.width,rect.height);
  if(typeof ResizeObserver==='undefined')return;
  const observer=new ResizeObserver(entries=>{for(const entry of entries)update(entry.contentRect.width,entry.contentRect.height);});
  observer.observe(stageElement);return()=>observer.disconnect();
 });
 let selectedEdge=$state<GraphEdge|null>(null);
 let selectedNode=$state<string|null>(null);
 let activeNode=$state<string|null>(null);
 let filter=$state('');
 let level=$state<'columns'|'topics'>('columns');
 let column=$state<string|null>(null);
 const hasColumns=$derived(info.topics.some(t=>t.topic.column.trim()));
 const displayLevel=$derived(level==='columns'&&!hasColumns?'topics':level);
 let zoom=$state(1);
 let yaw=$state(INITIAL_YAW);
 let pitch=$state(INITIAL_PITCH);
 let dragging=$state<{id:number;x:number;y:number;yaw:number;pitch:number;moved:boolean}|null>(null);
 let suppressClick=false;
 let expanded=$state<Record<string,boolean>>({});
 const graph=$derived.by(()=>{
  if(displayLevel==='columns')return buildColumnGraph(info.topics,info.relations,filter);
  let g=buildGraph(column===null?info.topics:info.topics.filter(t=>columnName(t)===column),info.relations,filter);
  for(const id of Object.keys(expanded))if(expanded[id]){
   const tv=info.topicView&&info.topicView.id===id?info.topicView:null;
   if(tv)g=expandTopic(g,id,tv.notes.map(n=>({id:n.meta.id,title:n.meta.title})));
  }
  return g;
 });
 const guides=$derived(sphereGuides(yaw,pitch,zoom));
 const projected=$derived(projectSphere(graph.nodes,yaw,pitch,zoom));
 const nodeIndex=$derived(new Map(projected.map(n=>[n.id,n])));
 function resetView(){zoom=1;yaw=INITIAL_YAW;pitch=INITIAL_PITCH;}
 function changeLevel(next:'columns'|'topics'){level=next;column=null;selectedEdge=null;selectedNode=null;resetView();}
 function onWheel(e:WheelEvent){e.preventDefault();zoom=clampZoom(zoom*Math.exp(-e.deltaY*.0015));}
 function onDown(e:PointerEvent){
  if(!e.isPrimary||e.button!==0)return;
  suppressClick=false;
  dragging={id:e.pointerId,x:e.clientX,y:e.clientY,yaw,pitch,moved:false};
 }
 function onMove(e:PointerEvent){
  if(!dragging||dragging.id!==e.pointerId)return;
  const dx=e.clientX-dragging.x,dy=e.clientY-dragging.y;
  if(!dragging.moved&&!isRotationDrag(dx,dy))return;
  dragging.moved=true;suppressClick=true;
  const target=e.currentTarget as HTMLElement;
  if(!target.hasPointerCapture(e.pointerId))target.setPointerCapture(e.pointerId);
  const rotation=dragRotation(dragging.yaw,dragging.pitch,dx,dy);yaw=rotation.yaw;pitch=rotation.pitch;
 }
 function onUp(e:PointerEvent){
  if(dragging?.id!==e.pointerId)return;
  suppressClick=dragging.moved||e.type==='pointercancel';dragging=null;
  const target=e.currentTarget as HTMLElement;
  if(target.hasPointerCapture(e.pointerId))target.releasePointerCapture(e.pointerId);
 }
 function allowClick(e:MouseEvent){return e.detail===0||!suppressClick;}
 function activateNode(node:GraphNode){
  selectedNode=node.id;selectedEdge=null;
  if(displayLevel==='columns'){
   column=info.topics.map(columnName).find(name=>columnNodeId(name)===node.id)??null;
   level='topics';filter='';resetView();return;
  }
  if(node.id.startsWith('note:')){onOpenNote(node.id.split(':').slice(2).join(':'));return;}
  expanded={...expanded,[node.id]:!expanded[node.id]};onOpenTopic(node.id);
 }
 const shortLabel=(name:string)=>Array.from(name).slice(0,9).join('')+(Array.from(name).length>9?'…':'');
</script>
<section class="graphview" data-graph aria-label="知识网络">
 <div class="graph-bar">
  <div class="graph-level" aria-label="网络层级"><button disabled={!hasColumns} title={hasColumns?'按栏目查看':'主题尚未分栏'} aria-pressed={displayLevel==='columns'} onclick={()=>changeLevel('columns')}>栏目</button><button aria-pressed={displayLevel==='topics'} onclick={()=>changeLevel('topics')}>主题</button></div>
  <input data-graph-search type="search" placeholder={displayLevel==='columns'?'筛选栏目或主题…':'筛选主题…'} value={filter} oninput={e=>{filter=e.currentTarget.value;selectedEdge=null;}}/>
  <button aria-label="放大知识网络" onclick={()=>zoom=clampZoom(zoom*1.2)}>放大</button><button aria-label="缩小知识网络" onclick={()=>zoom=clampZoom(zoom/1.2)}>缩小</button>
  <button onclick={resetView}>复位视图</button>
 </div>
 <div class="graph-summary"><span>{column??(displayLevel==='columns'?'栏目概览':'全部主题')} · {graph.nodes.length} 节点 · {graph.edges.length} 关系</span>{#if column}<button onclick={()=>changeLevel('columns')}>返回栏目</button>{/if}</div>
 <p class="graph-hint">拖动旋转 · 滚轮缩放 · 悬停或聚焦查看名称 · {displayLevel==='columns'?'点击栏目查看主题':'点击主题展开笔记'}</p>
 <p class="graph-limit">每次最多显示 30 个{displayLevel==='columns'?'栏目':'主题'}、60 条真实关系；主题最多展开 12 篇笔记，可搜索查找更多。</p>
 {#if selectedEdge}
  <div data-relation-detail role="status"><p>关系原因：{selectedEdge.reason}</p><p>证据笔记：{selectedEdge.evidence||'未提供'}</p>{#if selectedEdge.evidence}<button data-relation-evidence onclick={()=>onOpenNote(selectedEdge!.evidence)}>打开证据笔记</button>{/if}<button onclick={()=>selectedEdge=null}>关闭关系详情</button></div>
 {/if}
 <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
 <div class="graph-stage" data-graph-stage bind:this={stageElement}
   onwheel={onWheel} onpointerdown={onDown} onpointermove={onMove} onpointerup={onUp} onpointercancel={onUp}
   onlostpointercapture={()=>dragging=null}
   role="application" aria-label="知识球形网络（滚轮缩放，拖动旋转；Tab 选择节点和关系，回车打开）" tabindex="-1">
  <svg viewBox="0 0 800 600" width="800" height="600">
   {#if graph.nodes.length}
    <g class="sphere-guides" aria-hidden="true">
     <defs><radialGradient id="knowledge-sphere-shade"><stop offset="0" stop-color="currentColor" stop-opacity=".012"/><stop offset=".82" stop-color="currentColor" stop-opacity=".025"/><stop offset="1" stop-color="currentColor" stop-opacity=".055"/></radialGradient></defs>
     <circle cx="400" cy="300" r={210.55*zoom} fill="url(#knowledge-sphere-shade)"/>
     {#each guides as guide}<path d={guide.path} class:front={guide.front}/>{/each}
    </g>
   {/if}
   {#each graph.edges as edge (edge.id)}
    {@const from=nodeIndex.get(edge.from)}{@const to=nodeIndex.get(edge.to)}
    {#if from&&to}
     <line x1={from.x} y1={from.y} x2={to.x} y2={to.y}
       style={`--depth-opacity:${Math.max(.16,(from.opacity+to.opacity)/2*.55)}`}
       class="edge" class:selected={selectedEdge?.id===edge.id} data-edge={edge.id} role="button" tabindex="0" aria-label={`查看关系：${edge.reason}`}
       onclick={e=>{if(allowClick(e))selectedEdge=edge;}} onkeydown={e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();selectedEdge=edge;}}}><title>{edge.reason} · 证据：{edge.evidence}</title></line>
    {/if}
   {/each}
   {#each projected as node (node.id)}
    {@const full=activeNode===node.id||selectedNode===node.id}
    <g class="node" class:highlight={full} data-graph-node={node.id} transform={`translate(${node.x},${node.y})`}
      style={`--depth-opacity:${node.opacity}`} role="button" tabindex="0" aria-label={`${node.name}${displayLevel==='columns'?`，${node.count} 个主题`:''}`}
      onmouseenter={()=>activeNode=node.id} onmouseleave={()=>{if(activeNode===node.id)activeNode=null;}}
      onfocus={()=>activeNode=node.id} onblur={()=>{if(activeNode===node.id)activeNode=null;}}
      onkeydown={e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();activateNode(node);}}}
      onclick={e=>{if(allowClick(e))activateNode(node);}}>
     <circle r={Math.max(7,nodeRadius(node)*node.scale*.72)} class:locked={node.locked} class:note={node.id.startsWith('note:')}/>
     <title>{node.name}</title>
     {#if node.label||full}<text class:full style={`font-size:${(full?14:12)/fitScale}px`} x="0" y={Math.max(7,nodeRadius(node)*node.scale*.72)+19}>{full?node.name:shortLabel(node.name)}</text>{/if}
    </g>
   {/each}
  </svg>
  {#if graph.nodes.length===0}<p class="graph-empty">{filter?'没有匹配的内容。试试其他关键词。':'还没有主题。先在知识库中整理出主题，这里会长成你的知识网络。'}</p>{/if}
 </div>
</section>
<style>
 .graphview{display:flex;flex-direction:column;width:100%;height:100%;min-height:0;box-sizing:border-box;max-width:1000px;margin:0 auto;padding:0 18px 8px;font-size:16px;line-height:1.65;color:var(--text)}
 .graph-bar{flex-shrink:0;display:flex;flex-wrap:wrap;gap:10px;align-items:center;margin-bottom:12px}
 .graph-bar input{flex:1;min-width:120px;border:1px solid var(--edge);border-radius:12px;padding:10px 12px;background:var(--surface);font:inherit;font-size:16px;color:var(--text);box-shadow:var(--inset-shadow)}
 button{font:inherit;font-size:14px;border:1px solid var(--edge);background:var(--surface);color:var(--text);padding:9px 13px;border-radius:12px;cursor:pointer;box-shadow:var(--raised-shadow);transition:box-shadow .16s ease,background-color .16s ease}
 button:hover:not(:disabled){box-shadow:var(--hover-shadow)}button:active:not(:disabled){box-shadow:var(--pressed-shadow)}
 button:focus-visible,input:focus-visible{outline:2px solid var(--focus-ring);outline-offset:3px}
 .graph-level{display:flex;gap:4px}.graph-level button[aria-pressed=true]{color:var(--accent);border-color:var(--accent)}
 .graph-summary{flex-shrink:0;display:flex;align-items:center;gap:14px;flex-wrap:wrap;color:var(--muted);font-size:14px}
 .graph-summary button{padding:4px 10px;font-size:13px}
 .graph-hint,.graph-limit{flex-shrink:0;font-size:13px;color:var(--muted);margin:8px 0}.graph-limit{font-size:12px;opacity:.8}
 .graph-stage{flex:1;position:relative;border:1px solid var(--edge);border-radius:18px;background:var(--bg);overflow:hidden;cursor:grab;min-height:200px;touch-action:none;box-shadow:var(--inset-shadow)}
 .graph-stage:active{cursor:grabbing}svg{position:absolute;inset:0;display:block;width:100%;height:100%}
 .sphere-guides{color:var(--muted);pointer-events:none}.sphere-guides path{fill:none;stroke:currentColor;stroke-width:.65;stroke-opacity:.07}.sphere-guides path.front{stroke-opacity:.17}
 .edge{stroke:var(--muted);stroke-width:1.4;stroke-opacity:var(--depth-opacity);cursor:pointer;pointer-events:stroke}
 .edge:hover,.edge:focus-visible,.edge.selected{stroke:var(--accent);stroke-opacity:.9;stroke-width:3;outline:none}
 .node{opacity:var(--depth-opacity);cursor:pointer;outline:none}.node:hover,.node:focus-visible,.node.highlight{opacity:1}
 .node circle{fill:var(--surface-soft);stroke:var(--accent);stroke-width:1.5;cursor:pointer}
 .node circle.locked{stroke-dasharray:4 3}.node circle.note{fill:var(--surface);stroke:var(--muted);stroke-width:1.2}
 .node:focus-visible circle,.node.highlight circle{stroke-width:3;stroke:var(--accent)}
 .node text{fill:var(--text);text-anchor:middle;pointer-events:none}.node text.full{font-weight:600}
 .graph-empty{position:absolute;inset:30% 10% auto;text-align:center;color:var(--muted);font-size:14px;pointer-events:none}
 [data-relation-detail]{flex-shrink:0;max-height:150px;overflow:auto;margin-bottom:12px}[data-relation-detail] p{margin:8px 0;overflow-wrap:anywhere}
 @media(prefers-reduced-motion:reduce){button{transition:none}}
 @media(max-width:520px){.graphview{padding-inline:0}.graph-stage{min-height:180px}}
</style>
