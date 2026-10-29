<script lang="ts">
  import {onMount,tick} from 'svelte';
  import type {Snapshot} from './lib/draft';
  import type {SurfaceInfo} from './lib/surface';
  import {isDesktop} from './lib/bridge';
  import {boundedHeight,cssToLogicalRatio} from './lib/surface';
  import {renderMarkdown} from './lib/render';
  let {notice,draft,surface,response,running,busy,onEdit,onSave,onHide,onExpand,onDemo,onStop,onResize}: {
    notice:string;draft:Snapshot;surface:SurfaceInfo;response:string;running:boolean;busy:boolean;
    onEdit:(s:string)=>void;onSave:()=>void;onHide:()=>void;onExpand:()=>void;onDemo:()=>void;onStop:()=>void;onResize:(height:number,generation:number)=>void;
  }=$props();
  let root:HTMLDivElement;let answer=$state<HTMLDivElement>();let stick=true;
  let dpr=$state(devicePixelRatio);
  const ratio=$derived(isDesktop?cssToLogicalRatio(dpr,surface.scaleFactor):1);
  const html=$derived(response?renderMarkdown(response):'');
  const loaded=$derived(draft.loaded);
  $effect(()=>{if(loaded)void tick().then(()=>root?.querySelector('textarea')?.focus());});
  $effect(()=>{response;void tick().then(()=>{if(stick&&answer)answer.scrollTop=answer.scrollHeight;});});
  // A remapped native window can reset its size without changing DOM geometry.
  $effect(()=>{
    const generation=surface.generation;let active=true;
    void tick().then(()=>{if(active&&root)onResize(boundedHeight(root.getBoundingClientRect().height*ratio,surface.bounds.initialHeight,surface.bounds.maxHeight),generation);});
    return()=>{active=false;};
  });
  onMount(()=>{
    let timer:ReturnType<typeof setTimeout>|undefined;
    const observer=new ResizeObserver(()=>{if(timer!==undefined)return;timer=setTimeout(()=>{timer=undefined;onResize(boundedHeight(root.getBoundingClientRect().height*ratio,surface.bounds.initialHeight,surface.bounds.maxHeight),surface.generation);},60);});
    observer.observe(root);return()=>{observer.disconnect();clearTimeout(timer);};
  });
</script>
<svelte:window onresize={()=>{dpr=devicePixelRatio;}}/>
<div bind:this={root} data-capsule class="capsule-root" style={`--capsule-max:${surface.bounds.maxHeight/ratio}px;--capsule-width:${surface.bounds.width/ratio}px;min-height:${surface.bounds.initialHeight/ratio}px`}>
  <div class="capsule-compose">
    <div class="capsule-head"><span class="capsule-brand"><i></i> Zwen <small>模型未连接</small></span><div class="capsule-actions"><button data-demo onclick={running?onStop:onDemo}>{running?'停止':'演示'}</button><button aria-label="展开完整窗口" title="转到完整窗口，保留内容" disabled={busy} onclick={onExpand}>↗</button><button aria-label="收起胶囊" onclick={onHide}>×</button></div></div>
    <div class="capsule-input-row"><label class="sr-only" for="capsule-question">问题草稿</label><textarea id="capsule-question" aria-label="问题草稿" rows="1" disabled={!draft.loaded} value={draft.text} oninput={e=>onEdit(e.currentTarget.value)} placeholder="输入问题，或粘贴一段文字……"></textarea><button aria-label="保存草稿" title="保存到本机，不发送模型" disabled={!draft.loaded} onclick={onSave}>✓</button></div>
    {#if notice}<div class="capsule-error" role="status">{notice}</div>{/if}
    {#if draft.error}<div class="capsule-error" role="alert">{draft.error}<button onclick={onSave}>重试</button></div>{/if}
  </div>
  {#if response}
    <div class="capsule-response-label">演示回复 · 非 AI <span>{running?'逐步展示中':'已停止/完成'}</span></div>
    <div bind:this={answer} data-capsule-answer class="capsule-answer prose" onscroll={e=>{const box=e.currentTarget;stick=box.scrollHeight-box.scrollTop-box.clientHeight<32;}}>{@html html}</div>
  {/if}
</div>
