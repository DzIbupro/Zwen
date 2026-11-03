<script lang="ts">
  import Icon from './Icon.svelte';
  import RetryNotice from './RetryNotice.svelte';
  import type {RetryState} from './lib/chat';
  import ActionOutputNotice from './ActionOutputNotice.svelte';
  import type {ActionOutputState} from './lib/action-output';
  import type {QuickAction} from './lib/quick-actions';
  import BrandLogo from './BrandLogo.svelte';
  import SelectionNotice from './SelectionNotice.svelte';
  import Attachments from './Attachments.svelte';
  import type {SelectionState} from './lib/selection';
  import type {AttachmentState} from './lib/attachments';
  import {onMount,tick} from 'svelte';
  import type {Snapshot} from './lib/draft';
  import type {SurfaceInfo} from './lib/surface';
  import {isDesktop} from './lib/bridge';
  import {boundedHeight,cssToLogicalRatio} from './lib/surface';
  import {renderMarkdown} from './lib/render';
  let {retry=null,canRestoreQuestion=false,onRestoreQuestion=()=>{},focusRequest=0,quickActions=[],actionBusy=false,onSelectionAction,onQuickSettings,actionOutputState=null,onActionOutputRetry=()=>{},onActionOutputDismiss=()=>{},selectionState,onSelectionRetry,onSelectionDiscard,onSelectionReload,attachments,onAttachmentCapture,onAttachmentCancel,onAttachmentRemove,onAttachmentRetake,notice,draft,surface,response,running,busy,replyLabel,replyStatus,modelStatus,canSend,unsaved,onSend,onRetrySave,onEdit,onSave,onHide,onExpand,onDemo,onStop,onResize}: {
    retry?:RetryState|null;canRestoreQuestion?:boolean;onRestoreQuestion?:()=>void;focusRequest?:number;quickActions?:QuickAction[];actionBusy?:boolean;onSelectionAction?:(id:string)=>void;onQuickSettings?:()=>void;actionOutputState?:ActionOutputState|null;onActionOutputRetry?:()=>void;onActionOutputDismiss?:()=>void;selectionState:SelectionState;onSelectionRetry:()=>void;onSelectionDiscard:()=>void;onSelectionReload:()=>void;attachments:AttachmentState;onAttachmentCapture:()=>void;onAttachmentCancel:()=>void;onAttachmentRemove:(id:string)=>void;onAttachmentRetake:(id:string)=>void;
    notice:string;draft:Snapshot;surface:SurfaceInfo;response:string;running:boolean;busy:boolean;replyLabel:string;replyStatus:string;modelStatus:string;canSend:boolean;unsaved:boolean;onSend:()=>void;onRetrySave:()=>void;
    onEdit:(s:string)=>void;onSave:()=>void;onHide:()=>void;onExpand:()=>void;onDemo:()=>void;onStop:()=>void;onResize:(height:number,generation:number)=>void;
  }=$props();
  let root:HTMLDivElement;let answer=$state<HTMLDivElement>();let stick=true;
  let dpr=$state(devicePixelRatio);
  const ratio=$derived(isDesktop?cssToLogicalRatio(dpr,surface.scaleFactor):1);
  const selectionOnly=$derived(selectionState.pending?.mode==='candidate');
  const html=$derived(response?renderMarkdown(response):'');
  const loaded=$derived(draft.loaded);
  $effect(()=>{
    focusRequest;const ready=loaded&&!selectionOnly&&!busy;let active=true;
    if(ready)void tick().then(()=>{if(!active)return;const input=root?.querySelector('textarea');if(!input)return;input.focus();input.setSelectionRange(input.value.length,input.value.length);input.scrollTop=input.scrollHeight;input.scrollLeft=input.scrollWidth;});
    return()=>{active=false;};
  });
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
    <div class="capsule-head"><span class="capsule-brand"><BrandLogo compact/> Zwen <small>{modelStatus}</small></span><div class="capsule-actions">{#if !selectionOnly||running}{#if running}<button data-stop title="停止生成" disabled={busy} onclick={onStop}><Icon name="stop" size={16}/><span class="sr-only">停止</span></button>{:else}<details class="capsule-more"><summary aria-label="胶囊更多操作"><Icon name="more" size={16}/></summary><div class="menu"><button data-demo onclick={onDemo}>排版演示（非 AI）</button></div></details>{/if}{/if}<button aria-label="展开完整窗口" title="转到完整窗口，保留内容" disabled={busy||actionBusy||selectionState.busy} onclick={onExpand}><Icon name="external" size={18}/></button><button aria-label="收起胶囊" disabled={busy||actionBusy||selectionState.busy||attachments.pending} onclick={onHide}><Icon name="close" size={18}/></button></div></div>
    {#if !selectionOnly}
    <div class="capsule-input-row"><label class="sr-only" for="capsule-question">问题草稿</label><textarea id="capsule-question" aria-label="问题草稿" rows="1" disabled={!draft.loaded||busy} value={draft.text} oninput={e=>onEdit(e.currentTarget.value)} placeholder="输入问题，或粘贴一段文字……"></textarea><button data-capture aria-label="附加截图" title="附加截图；不会自动发送" disabled={!draft.loaded||attachments.pending||attachments.items.length>=attachments.max||busy} onclick={onAttachmentCapture}><Icon name="image" size={18}/></button><button aria-label="保存草稿" title="保存到本机，不发送模型" disabled={!draft.loaded} onclick={onSave}><Icon name="save" size={17}/></button><button data-send aria-label="发送问题" title="发送给已配置模型" disabled={!canSend||!draft.loaded||!draft.text.trim()||attachments.pending} onclick={onSend}><Icon name="send" size={18}/></button></div>
    <Attachments state={attachments} onCancel={onAttachmentCancel} onRemove={onAttachmentRemove} onRetake={onAttachmentRetake} disabled={busy}/>
    <ActionOutputNotice state={actionOutputState} onRetry={onActionOutputRetry} onDismiss={onActionOutputDismiss}/>
    {/if}
    <SelectionNotice actions={quickActions} {actionBusy} onAction={onSelectionAction} onSettings={onQuickSettings} state={selectionState} onRetry={onSelectionRetry} onDiscard={onSelectionDiscard} onReload={onSelectionReload}/>
    {#if notice}<div class="capsule-error" role="status">{notice}{#if unsaved}<button onclick={onRetrySave}>重试保存</button>{/if}</div>{/if}
    {#if canRestoreQuestion}<button data-restore-question class="capsule-restore" disabled={busy} onclick={onRestoreQuestion}>恢复问题，重新提问</button>{/if}
    {#if draft.error&&!selectionOnly}<div class="capsule-error" role="alert">{draft.error}<button onclick={onSave}>重试</button></div>{/if}
  </div>
  {#if (response||running)&&(!selectionOnly||(actionBusy&&running))}
    <div class="capsule-response-label">{replyLabel} <span>{replyStatus}</span></div>
    {#if response}
    <div bind:this={answer} data-capsule-answer class="capsule-answer prose" onscroll={e=>{const box=e.currentTarget;stick=box.scrollHeight-box.scrollTop-box.clientHeight<32;}}>{@html html}</div>
    {:else}<div data-capsule-waiting class="capsule-waiting">{#if retry}<RetryNotice {retry}/>{:else}<span role="status">{replyLabel.includes('演示')?'正在准备演示…':'正在等待模型回复…'}</span>{/if}</div>{/if}
  {/if}
</div>

<style>
 .capsule-restore{align-self:flex-start;font:inherit;font-size:13px;margin:4px 16px 8px;padding:4px 8px;border:1px solid var(--edge);border-radius:6px;background:var(--surface);color:var(--text);}
 .capsule-waiting{padding:12px 18px 16px;font-size:14px;line-height:1.6;color:var(--muted);flex-shrink:0;}
</style>
