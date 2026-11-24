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
  import {isDesktop,startCapsuleDrag} from './lib/bridge';
  import {boundedHeight,cssToLogicalRatio} from './lib/surface';
  import {handleProseClick,renderMarkdown} from './lib/render';
  import {openExternal} from './lib/bridge';
  import {t} from './lib/i18n.svelte';
  let {paperLabel=null,hasPaperAttachment=false,retry=null,canRestoreQuestion=false,onRestoreQuestion=()=>{},focusRequest=0,quickActions=[],actionBusy=false,onSelectionAction,onQuickSettings,actionOutputState=null,onActionOutputRetry=()=>{},onActionOutputDismiss=()=>{},selectionState,onSelectionRetry,onSelectionDiscard,onSelectionReload,attachments,onAttachmentCapture,onAttachmentImage,onAttachmentCancel,onAttachmentRemove,onAttachmentRetake,notice,draft,surface,response,running,busy,replyLabel,replyStatus,modelStatus,canSend,unsaved,onSend,onRetrySave,onEdit,onSave,onHide,onExpand,onStop,onResize}: {
    paperLabel?:string|null;hasPaperAttachment?:boolean;retry?:RetryState|null;canRestoreQuestion?:boolean;onRestoreQuestion?:()=>void;focusRequest?:number;quickActions?:QuickAction[];actionBusy?:boolean;onSelectionAction?:(id:string)=>void;onQuickSettings?:()=>void;actionOutputState?:ActionOutputState|null;onActionOutputRetry?:()=>void;onActionOutputDismiss?:()=>void;selectionState:SelectionState;onSelectionRetry:()=>void;onSelectionDiscard:()=>void;onSelectionReload:()=>void;attachments:AttachmentState;onAttachmentCapture:()=>void;onAttachmentImage?:()=>void;onAttachmentCancel:()=>void;onAttachmentRemove:(id:string)=>void;onAttachmentRetake:(id:string)=>void;
    notice:string;draft:Snapshot;surface:SurfaceInfo;response:string;running:boolean;busy:boolean;replyLabel:string;replyStatus:string;modelStatus:string;canSend:boolean;unsaved:boolean;onSend:()=>void;onRetrySave:()=>void;
    onEdit:(s:string)=>void;onSave:()=>void;onHide:()=>void;onExpand:()=>void;onStop:()=>void;onResize:(height:number,generation:number)=>void;
  }=$props();
  let root:HTMLDivElement;let answer=$state<HTMLDivElement>();let stick=true;
  let dragError=$state('');
  let attachOpen=$state(false);let fileInput=$state<HTMLInputElement>();let fileError=$state('');
  async function handleTextFile(event:Event){
    const el=event.currentTarget as HTMLInputElement;const file=el.files?.[0];el.value='';if(!file)return;
    if(file.size>200*1024){fileError='单个文本文件不超过 200KB';return;}
    fileError='';
    try{const text=await file.text();const block=`[文件: ${file.name}]\n\`\`\`\n${text}\n\`\`\``;onEdit(draft.text.trim()?`${draft.text.trimEnd()}\n\n${block}`:block);}
    catch{fileError='文件读取失败，请重试';}
  }
  function dragHeader(event:MouseEvent){
    if(event.button!==0||event.detail>1||!(event.target instanceof Element)||event.target.closest('button,a,input,textarea,select,[contenteditable],[role="button"]'))return;
    event.preventDefault();dragError='';
    void startCapsuleDrag().catch(()=>{dragError='暂时无法移动窗口，请重试';});
  }
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
    <div class="capsule-head" role="toolbar" tabindex="-1" aria-label={t('capsule.dragTitle')} onmousedown={dragHeader}><span class="capsule-brand"><BrandLogo compact/> Zwen <small>{modelStatus}</small></span><div class="capsule-actions">{#if running}<button data-stop title={t('reply.stop')} disabled={busy} onclick={onStop}><Icon name="stop" size={16}/><span class="sr-only">{t('capsule.stop')}</span></button>{/if}<button aria-label={t('capsule.expand')} title={t('capsule.expandTitle')} disabled={busy||actionBusy||selectionState.busy} onclick={onExpand}><Icon name="external" size={18}/></button><button aria-label={t('capsule.hide')} disabled={busy||actionBusy||selectionState.busy||attachments.pending} onclick={onHide}><Icon name="close" size={18}/></button></div></div>
    {#if !selectionOnly}
    {#if paperLabel}<button class="capsule-paper" data-capsule-paper disabled={busy} onclick={onExpand}><Icon name="book" size={14}/>{t('capsule.paperPrefix')} · {paperLabel}</button>{/if}
    <div class="capsule-input-row"><label class="sr-only" for="capsule-question">{t('composer.label')}</label><textarea id="capsule-question" aria-label={t('composer.label')} rows="1" disabled={!draft.loaded||busy} value={draft.text} oninput={e=>onEdit(e.currentTarget.value)} placeholder={hasPaperAttachment?t('capsule.placeholderPaper'):paperLabel?t('capsule.placeholderPaperFollow'):t('capsule.placeholder')}></textarea><button type="button" data-attach-trigger aria-label={t('composer.attach')} title={t('composer.attachTitle')} aria-expanded={attachOpen} disabled={!draft.loaded||busy} onclick={()=>{attachOpen=!attachOpen;}}><Icon name="plus" size={17}/></button><button data-send aria-label={t('capsule.send')} title={t('composer.sendTipEnter')} disabled={!canSend||!draft.loaded||(!draft.text.trim()&&!hasPaperAttachment)||attachments.pending} onclick={onSend}><Icon name="send" size={17}/></button></div>
    {#if attachOpen}
     <div class="capsule-attach-bar" role="menu" aria-label={t('composer.attachOptions')}>
      <button type="button" role="menuitem" data-capture disabled={!!paperLabel||attachments.pending||attachments.items.length>=attachments.max||busy} onclick={()=>{attachOpen=false;onAttachmentCapture();}}><Icon name="image" size={14}/>{t('composer.screenshot')}</button>
      {#if onAttachmentImage}<button type="button" role="menuitem" data-upload-image disabled={!!paperLabel||attachments.pending||attachments.items.length>=attachments.max||busy} onclick={()=>{attachOpen=false;onAttachmentImage();}}><Icon name="image" size={14}/>{t('composer.uploadImage')}</button>{/if}
      <button type="button" role="menuitem" data-upload-file disabled={!draft.loaded||busy} onclick={()=>{attachOpen=false;fileInput?.click();}}><Icon name="paperclip" size={14}/>{t('composer.uploadFile')}</button>
     </div>
    {/if}
    <input bind:this={fileInput} type="file" class="sr-only" tabindex="-1" accept=".txt,.md,.markdown,.json,.csv,.ts,.js,.py,.rs,.c,.cpp,.h,.java,.go,.html,.css,.xml,.yaml,.yml,.toml,.log,.tex" onchange={e=>void handleTextFile(e)}/>
    {#if fileError}<div class="capsule-error" role="status">{fileError}</div>{/if}
    <Attachments state={attachments} onCancel={onAttachmentCancel} onRemove={onAttachmentRemove} onRetake={onAttachmentRetake} disabled={busy}/>
    <ActionOutputNotice state={actionOutputState} onRetry={onActionOutputRetry} onDismiss={onActionOutputDismiss}/>
    {/if}
    <SelectionNotice actions={quickActions} {actionBusy} onAction={onSelectionAction} onSettings={onQuickSettings} state={selectionState} onRetry={onSelectionRetry} onDiscard={onSelectionDiscard} onReload={onSelectionReload}/>
    {#if dragError}<div class="capsule-error" role="status">{dragError}</div>{/if}
    {#if notice}<div class="capsule-error" role="status">{notice}{#if unsaved}<button onclick={onRetrySave}>{t('draft.retrySave')}</button>{/if}</div>{/if}
    {#if canRestoreQuestion}<button data-restore-question class="capsule-restore" disabled={busy} onclick={onRestoreQuestion}>{t('reply.restore')}</button>{/if}
    {#if draft.error&&!selectionOnly}<div class="capsule-error" role="alert">{draft.error}<button onclick={onSave}>{t('capsule.retry')}</button></div>{/if}
  </div>
  {#if (response||running)&&(!selectionOnly||(actionBusy&&running))}
    <div class="capsule-response-label">{replyLabel} <span class="stream-status">{#if running}<span class="stream-dot" aria-hidden="true"></span>{/if}{replyStatus}</span></div>
    {#if response}
    <div bind:this={answer} data-capsule-answer class="capsule-answer prose" role="presentation" onclick={e=>handleProseClick(e,openExternal)} onkeydown={e=>handleProseClick(e as unknown as MouseEvent,openExternal)} onscroll={e=>{const box=e.currentTarget;stick=box.scrollHeight-box.scrollTop-box.clientHeight<32;}}>{@html html}{#if running}<div class="stream-tail" data-streaming-indicator aria-hidden="true"><span></span><span></span><span></span></div>{/if}</div>
    {:else}<div data-capsule-waiting class="capsule-waiting">{#if retry}<RetryNotice {retry}/>{:else}<div class="thinking-indicator" role="status"><span class="thinking-spinner" aria-hidden="true"></span><span>{t('reply.waitingModel').replace(/[…\.]+$/,'')}</span><span class="thinking-dots" aria-hidden="true"><i></i><i></i><i></i></span></div>{/if}</div>{/if}
  {/if}
</div>

<style>
 .capsule-restore{align-self:flex-start;font:inherit;font-size:13px;margin:4px 16px 8px;padding:4px 8px;border:1px solid var(--edge);border-radius:6px;background:var(--surface);color:var(--text);}
 .capsule-waiting{padding:12px 18px 16px;font-size:14px;line-height:1.6;color:var(--muted);flex-shrink:0;}
.capsule-paper{display:flex;align-items:center;gap:6px;margin:4px 14px;padding:5px 9px;border:1px solid var(--edge);border-radius:10px;background:var(--surface-soft);color:var(--muted);font-size:12px;max-width:calc(100% - 28px);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
</style>
