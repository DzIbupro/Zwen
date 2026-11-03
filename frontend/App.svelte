<script lang="ts">
  import RetryNotice from './RetryNotice.svelte';
  import ActionOutputNotice from './ActionOutputNotice.svelte';
  import QuickActionSettings from './QuickActionSettings.svelte';
  import {defaultQuickActions,type QuickActionConfig} from './lib/quick-actions';
  import {ActionOutputController,type ActionOutputState} from './lib/action-output';
  import {quickActionPort,actionOutputPort} from './lib/bridge';
  import {QuickSession} from './lib/quick-session';
  import {SendDraftCoordinator} from './lib/send-draft';
  import BrandLogo from './BrandLogo.svelte';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { DraftController, type Snapshot } from './lib/draft';
  import { draftPort, isDesktop, hideWindow, quitApp } from './lib/bridge';
  import { renderMarkdown } from './lib/render';
  import Capsule from './Capsule.svelte';
  import {getSurface,setSurface,resizeCapsule} from './lib/bridge';
  import {canApplySurface,ensureSurface,ResizeQueue,type SurfaceInfo} from './lib/surface';
  import {DemoStream} from './lib/demo';
  import {ChatController,type ChatState} from './lib/chat';
  import {chatPort,historyPort} from './lib/bridge';
  import HistoryPanel from './HistoryPanel.svelte';
  import type {ChatReply} from './lib/chat';
  import {shouldSendQuestion} from './lib/keyboard';
  import SelectionNotice from './SelectionNotice.svelte';
  import {SelectionCommandCoordinator,selectionActionBlock,type SelectionCommand} from './lib/selection-command';
  import {SelectionController,type SelectionItem,type SelectionState} from './lib/selection';
  import {selectionPort} from './lib/bridge';
  import ProviderSettings from './ProviderSettings.svelte';
  import {ProviderController,imageModelOf,type SettingsState} from './lib/provider';
  import {providerPort} from './lib/bridge';
  import Attachments from './Attachments.svelte';
  import {AttachmentController,type AttachmentState} from './lib/attachments';
  import {attachmentPort,openExternal,invokeRaw as invoke} from './lib/bridge';
  import {proseLinkClick} from './lib/links';
  import KnowledgePanel from './KnowledgePanel.svelte';
  import KnowledgeGraph from './KnowledgeGraph.svelte';
  import {KnowledgeController,type KnowledgeState} from './lib/knowledge';
  import {knowledgePort} from './lib/bridge';
  let quickConfig=$state<QuickActionConfig>({revision:0,actions:defaultQuickActions(),outputDirectory:''});
  let actionBusy=$state(false);
  let focusRequest=$state(0);
  const quickSession=new QuickSession();
  let quickConfigLoaded=$state(false);
  let actionOutputState=$state<ActionOutputState|null>(null);
  const actionOutput=new ActionOutputController(actionOutputPort,s=>actionOutputState=s);
  let notice=$state('');
  let view=$state<'ask'|'preview'|'reply'|'settings'|'history'|'knowledge'|'graph'>('ask');
  let settings=$state<SettingsState>({config:null,revision:0,keyAvailable:false,keySource:null,fields:{baseUrl:'',model:'',keyEnv:'ZWEN_API_KEY',imageModel:''},status:'loading',loaded:false,error:null,secretSaving:false,secretNotice:null});
  const provider=new ProviderController(providerPort,s=>settings=s);
  let surface=$state<SurfaceInfo>({mode:'full',scaleFactor:1,generation:0,bounds:{width:480,initialHeight:96,maxHeight:594}});
  let responseKind=$state<'demo'|'model'>('model');let demoText=$state('');let demoRunning=$state(false);let switching=$state(false);let closing=$state(false);
  let chatState=$state<ChatState>({answer:'',running:false,stopping:false,loaded:false,error:null,reply:null});
  const chat=new ChatController(chatPort,s=>{const appeared=!chatState.reply&&s.reply;chatState=s;settlePendingSend();actionOutput.observe(s.reply);if(appeared&&view==='ask'){responseKind='model';view='reply';}});
  const response=$derived(responseKind==='demo'?demoText:chatState.answer);
  const running=$derived(responseKind==='demo'?demoRunning:chatState.running);
  const replyStatus=$derived(responseKind==='demo'?(demoRunning?'演示中':'演示结束'):chatState.reply?.retry?'等待重试':chatState.reply?({streaming:'生成中',complete:'已完成',stopped:'已停止',failed:'生成失败',interrupted:'上次中断',limited:'已达限制',filtered:'服务限制'}[chatState.reply.state]):chatState.running?'等待回复':'');
  const replyLabel=$derived(responseKind==='demo'?'演示回复 · 非 AI':'模型回复');
  const combinedNotice=$derived(responseKind==='model'?(chatState.error||notice):notice);
  const demo=new DemoStream((text,active)=>{demoText=text;demoRunning=active;});
  function startDemo(){if(chatState.running||chatState.stopping)return;responseKind='demo';demo.start();}

  let traceRequestId=$state<string|null>(null);
 let candidateEnabled=$state(false);
 void 0;
  // The input clears the moment a send is accepted. When the model starts
  // answering, the screenshots are consumed; a request that ends without a
  // single token (429, network…) puts the question back for a one-press retry.
  function settlePendingSend(){sendDraft.reconcile(chatState);}
  async function send(){
    if(closing||chatState.running||chatState.stopping||actionBusy)return;
    if(actionOutputState&&actionOutputState.stage!=='saved'){notice='请先处理或取消文档自动保存';return;}
    if(selectionState.busy){notice='请等待选区处理完成';return;}
    if(attachmentState.pending){attachmentController.note('请等待截图完成');return;}
    if(attachmentState.items.length>0&&!imageModelOf(settings.config)){attachmentController.note('截图尚未接入模型；请先在模型设置中配置图片模型');return;}
    if(!isDesktop){notice='浏览器仅预览，请在桌面应用中发送';return;}
    if(settings.status==='loading'||!chatState.loaded){notice='正在读取本机配置，请稍后发送';return;}
    if(!settings.config||!settings.keyAvailable){notice='请先配置模型服务与密钥';await switchSurface('full');view='settings';return;}
    demo.stop();responseKind='model';view='reply';const sent=draft.text;const sentIds=attachmentState.items.map(item=>item.id);
    const ticket=sendDraft.begin(sentIds);if(!ticket)return;
    const accepted=await chat.send(sent,()=>controller.flush());
    if(accepted)sendDraft.accept(ticket,chatState);else sendDraft.cancel(ticket);
  }
  async function openQuickSettings(){await switchSurface('full');view='settings';setTimeout(()=>document.querySelector('[data-quick-action-settings]')?.scrollIntoView({block:'start'}),0);}
  async function selectAction(id:string,expected?:{item:SelectionItem;config:QuickActionConfig}){
    if(appDisposed||closing)return;
    if(expected&&(selectionState.pending?.id!==expected.item.id||selectionState.pending.text!==expected.item.text))return;
    if(actionBusy||selectionState.busy||!selectionState.pending?.text)return;
    const blocked=selectionActionBlock({text:draft.text,attachmentCount:attachmentState.items.length,attachmentPending:attachmentState.pending,loaded:draft.loaded&&attachmentState.loaded&&chatState.loaded,unsaved:!!chatState.reply&&!chatState.reply.saved,running:chatState.running||chatState.stopping});
    if(blocked){notice=blocked;return;}
    if(id==='ask'){
      quickSession.begin(selectionState.pending.id);
      actionBusy=true;try{await selection.applyPending(expected?.item);if(!appDisposed&&!selectionState.pending){chat.newConversation();responseKind='model';view='ask';focusRequest++;await switchSurface('capsule');}}finally{actionBusy=false;}return;
    }
    if(!quickConfigLoaded){notice='快捷提示词尚未读入，请在设置中重新读取';return;}
    const actionConfig=expected?.config??quickConfig;
    const action=actionConfig.actions.find(a=>a.id===id&&a.enabled);if(!action){notice='该快捷动作未启用，请在设置中检查';return;}
    if(actionOutputState&&actionOutputState.stage!=='saved'){notice='请先处理上次文档保存，或关闭其提示';return;}
    if(!isDesktop){notice='浏览器仅预览，请在桌面程序中执行划词操作';return;}
    if(!chatState.loaded||!settings.loaded||settings.status==='loading'){notice='正在读取本机模型配置，请稍后重试';return;}
    if(!settings.config||!settings.keyAvailable){notice='请先配置模型服务与密钥；选区已保留';await openQuickSettings();return;}
    actionBusy=true;
    try{
      const item=await selection.freezePending(expected?.item);if(appDisposed||!item?.text)return;
      quickSession.begin(item.id);
      const requestId=crypto.randomUUID();
      if(!actionOutput.track({requestId,selection:item.text,actionId:action.id,actionName:action.name,output:action.output,outputDirectory:actionConfig.outputDirectory}))return;
      demo.stop();responseKind='model';view='reply';
      const ticket=sendDraft.begin([], 'selection');if(!ticket){actionOutput.dismiss();return;}
      const accepted=await chat.sendSelection(requestId,item.text,action.id,actionConfig.revision);
      if(accepted)sendDraft.accept(ticket,chatState);else sendDraft.cancel(ticket);
      if(appDisposed)return;
      if(accepted){await selection.acknowledgeFrozen(item.id);if(!appDisposed)await switchSurface('capsule');}
      else{actionOutput.dismiss();notice=chatState.error||'请求未开始，选区保留，请手动重试';}
    }finally{actionBusy=false;}
  }
  async function onCandidateToggle(on:boolean){
    try{await invoke('candidate_set_enabled',{enabled:on});candidateEnabled=on;}
    catch(error){notice=error instanceof Error?error.message:'自动划词开关未完成';}
  }
  function traceToHistory(requestId:string){traceRequestId=requestId;view='history';}
  async function saveNoteEdit(id:string,title:string,body:string,source:{sourceTitle?:string|null;sourceUrl?:string|null;sourceLoc?:string|null}){return knowledge.saveEdit(id,title,body,source);}
  function newConversation(){if(closing)return;if(chat.newConversation()){quickSession.release();responseKind='model';view='ask';notice='已准备新对话，当前草稿保留';}}
  function continueHistory(reply:ChatReply){if(closing)return;if(chat.adopt(reply)){quickSession.release();responseKind='model';view='reply';notice='继续此会话将附带近期问答';}}
  function stopReply(){if(responseKind==='demo')demo.stop();else void chat.stop().catch(()=>{});}
  const canRestoreQuestion=$derived(responseKind==='model'&&!chatState.running&&!chatState.stopping&&!!chatState.reply?.saved&&['failed','stopped','interrupted','limited'].includes(chatState.reply.state));
  async function restoreQuestion(){
    const reply=chatState.reply;if(closing||!reply||!canRestoreQuestion)return;
    if(draft.text.trim()&&draft.text!==reply.prompt){notice='当前草稿已有新内容，请先处理；旧问题和部分回答仍在历史中';return;}
    if(actionOutputState?.stage==='saving'){notice='文档正在保存，请稍后重提问';return;}
    actionOutput.dismiss();controller.edit(reply.prompt);
    try{await controller.flush();view='ask';focusRequest++;input?.focus();input?.setSelectionRange(reply.prompt.length,reply.prompt.length);notice='问题已恢复，点击发送可重新提问；已有回答保留在历史中';}
    catch{notice='问题保留在草稿，保存未完成，请先重试保存';}
  }
  async function retryReplySave(){try{await chat.settle();}catch(e){notice=e instanceof Error?e.message:'回答保存未完成';}}
  function captureScreenshot(replaceId?:string|null){if(!isDesktop){notice='浏览器仅预览，请在桌面应用中截图';return;}void attachmentController.capture(replaceId??null);}
  async function saveToKnowledge(){
    const reply=chatState.reply;
    if(!reply||!reply.answer||chatState.running)return;
    const title=reply.prompt.replace(/\s+/g,' ').trim().slice(0,60);
    const body=`## 问题\n\n${reply.prompt}\n\n## 回答\n\n${reply.answer}`;
    const saved=await knowledge.save({title,body,source:'chat',model:reply.model,attachmentIds:attachmentState.items.map(item=>item.id),requestId:reply.requestId});
    if(saved){notice='已收藏到知识库';await knowledge.runAutoFor(saved.id);}
  }

  function applySurface(next:SurfaceInfo){if(canApplySurface(surface.generation,next.generation)){surface=next;document.documentElement.dataset.surface=next.mode;}}
  async function switchSurface(mode:'full'|'capsule'){
    if(switching)return;switching=true;
    try{await controller.flush();applySurface(await ensureSurface(mode,{get:getSurface,set:setSurface}));if(mode==='full'&&response)view='reply';}
    catch(error){notice=error instanceof Error?error.message:'窗口切换未完成，内容已保留';}
    finally{switching=false;}
  }
  const resizeQueue=new ResizeQueue(resizeCapsule,()=>{notice='窗口尺寸调整未完成，可展开完整窗口';});
  function resize(height:number,generation:number){resizeQueue.request(height,generation);}

  let draft = $state<Snapshot>({text:'',revision:0,status:'loading',error:null,loaded:false});
  const controller = new DraftController(draftPort, s=>draft=s);
  const sendDraft=new SendDraftCoordinator(controller,ids=>{void attachmentController.removeSent(ids);});
  let appDisposed=false;
  let draftReady:Promise<void>|undefined;
  let selectionState=$state<SelectionState>({busy:false,pending:null,inserted:false,notice:''});
  const selection=new SelectionController(selectionPort,controller,s=>{if(s.pending?.mode==='candidate'&&s.pending.id!==selectionState.pending?.id)notice='';selectionState=s;});
  const selectionCommands=new SelectionCommandCoordinator({
    refresh:async()=>{if(!draft.loaded)await (draftReady??controller.start());if(!appDisposed)await selection.refresh();},
    pending:()=>selectionState.pending,
    freeze:item=>selection.freezePending(item),
    loadConfig:()=>quickActionPort.load(),
    configChanged:config=>{if(!appDisposed){quickConfig=config;quickConfigLoaded=true;}},
    execute:async(command,item,config)=>{
      if(appDisposed)return;
      quickConfig=config;quickConfigLoaded=true;
      await selectAction(command.actionId,{item,config});
    }
  });
  let attachmentState=$state<AttachmentState>({items:[],pending:false,notice:'',loaded:false,max:3});
  const attachmentController=new AttachmentController(attachmentPort,s=>attachmentState=s);
  let knowledgeState=$state<KnowledgeState>({notes:[],query:'',current:null,busy:false,loaded:false,notice:'',topics:[],organizeMode:false,selected:{},confirming:false,organizeBusy:false,lastBatch:null,lastSummary:null,topicView:null,relations:[],auto:false});
  const knowledge=new KnowledgeController(knowledgePort,s=>knowledgeState=s);
  let cleared = $state<string|null>(null);
  let input = $state<HTMLTextAreaElement>();
  const example = `## 把复杂，拆成可以理解的部分\n\n正则化让模型在拟合数据与控制复杂度之间取得平衡。\n\n| 方法 | 思路 | 适用场景 |\n| --- | --- | --- |\n| L1 | 让部分参数变为零 | 特征选择 |\n| L2 | 限制参数的幅度 | 改善泛化 |\n\n行内公式 $E=mc^2$，也可以清楚地展示：\n\n$$\nL(w)=\\frac{1}{n}\\sum_{i=1}^{n}(y_i-w^Tx_i)^2+\\lambda\\lVert w\\rVert_2^2\n$$\n\n> 先理解一个概念，再把它与已有知识连起来。\n\n\`\`\`python\nloss = data_loss + strength * penalty\n\`\`\``;
  const statusText = $derived({loading:'正在读取草稿',saved:'已保存到本机',dirty:'等待保存',saving:'正在保存',error:'保存未完成'}[draft.status]);
  const preview = $derived(view==='reply' ? renderMarkdown(response) : view==='preview' ? renderMarkdown(draft.text.trim() ? draft.text : example) : '');
  function fill(text:string) { controller.insertPrompt(text);input?.focus(); }
  async function save() { try { if(!draft.loaded){await controller.start();await selection.refresh();}else await controller.flush(); notice='草稿已保存'; } catch { /* error is visible in draft state */ } }
  async function leave(quit=false) {
    if (!isDesktop) { notice='这是浏览器预览，草稿保存在此浏览器；桌面收起功能请在应用中使用';return; }
    if(quit&&(actionBusy||actionOutputState&&actionOutputState.stage!=='saved')){notice='请先完成或取消文档自动保存，再退出';return;}
    if(quit&&knowledge.pendingEditorId()){notice='知识库仍有编辑未完成，请先保存编辑或取消；切换页面会保留本次编辑';view='knowledge';void knowledge.open(knowledge.pendingEditorId()!);return;}
    if(quit&&(selectionState.busy||selectionState.pending)){notice='请先处理待导入选区，再退出';return;}
    if(quit&&attachmentState.pending){notice='请先取消等待截图，再退出';return;}
    if(closing)return;
    const endQuick=surface.mode==='capsule'&&quickSession.active;
    if(endQuick&&(actionBusy||selectionState.busy||attachmentState.pending)){notice='请等待选区处理完成，或先取消截图，再关闭胶囊';return;}
    closing=true;
    try {
      if(endQuick){
        let recoveryDraft='';
        await quickSession.close({
          settle:async()=>{demo.stop();await chat.settle();if(chatState.reply&&!chatState.reply.saved)throw Error('回答尚未保存，请重试保存后再关闭');if(actionOutputState?.stage==='saving')throw Error('文档正在保存，请稍后关闭');},
          revokeSend:()=>sendDraft.revoke(),
          discardSelection:id=>selection.discardOwned(id),
          clearMaterials:async()=>{const ids=attachmentState.items.map(item=>item.id);for(const id of ids)await attachmentPort.remove(id);await attachmentController.load();},
          clearDraft:async()=>{const text=controller.snapshot.text;recoveryDraft=text;controller.edit('');try{await controller.flush();}catch(error){controller.edit(text);throw error;}},
          restoreDraft:async()=>{controller.edit(recoveryDraft);await controller.flush();},
          resetView:()=>{if(!chat.newConversation())throw Error('对话尚未结束，请稍后重试');actionOutput.dismiss();responseKind='model';view='ask';notice='';cleared=null;}
        });
      }else if(quit)await chat.settle();
      await controller.leave(quit ? quitApp : hideWindow);
    } catch(e){notice=e instanceof Error?e.message:'退出未完成，当前内容已保留';}finally{closing=false;}
  }
  function keydown(e:KeyboardEvent) {
    if (e.isComposing || e.keyCode===229) return;
    if (shouldSendQuestion(e)){e.preventDefault();void send();}
    if (e.key==='Escape') {e.preventDefault();void leave();}
    if ((e.ctrlKey||e.metaKey) && e.key.toLowerCase()==='s') {e.preventDefault();void save();}
  }
  onMount(()=>{
    let disposed=false;const unlisteners:(()=>void)[]=[];
    document.documentElement.dataset.browserPreview=isDesktop?'false':'true';
    void provider.load();void chat.load();
    if(isDesktop)void quickActionPort.load().then(c=>{if(disposed)return;quickConfig=c;quickConfigLoaded=true;}).catch(error=>{if(!disposed)notice=error instanceof Error?error.message:'快捷提示词读取失败';});
    draftReady=controller.start();
    void draftReady.then(()=>{if(disposed)return;input?.focus();void selection.refresh();}).catch(()=>{});
    void attachmentController.start();
    if(isDesktop)void invoke('candidate_get_enabled').then((value:unknown)=>{if(disposed)return;candidateEnabled=typeof value==='boolean'&&value;}).catch(()=>{});
    void getSurface().then(value=>{if(!disposed)applySurface(value);}).catch(()=>{if(!disposed)notice='窗口模式读取失败';});
    if(isDesktop) void listen<SelectionCommand>('selection-command',e=>{if(!disposed)void selectionCommands.handle(e.payload).catch(error=>{if(!disposed)notice=error instanceof Error?error.message:'选区操作未开始；材料已保留';});}).then(fn=>{if(disposed)fn();else unlisteners.push(fn);}).catch(()=>{if(!disposed)notice='选区命令事件暂不可用，请手动打开面板重试';});
    if(isDesktop) void listen('selection-ready',()=>{if(!disposed)void selection.refresh();}).then(fn=>{if(disposed)fn();else{unlisteners.push(fn);void selection.refresh();}}).catch(()=>{if(!disposed)notice='选区事件暂不可用，可重新读取状态';});
    if(isDesktop) void listen<SurfaceInfo>('surface-changed',e=>{if(disposed)return;applySurface(e.payload);if(e.payload.mode==='full'&&response&&view==='ask')view='reply';}).then(fn=>{if(disposed)fn();else unlisteners.push(fn);});
    if(isDesktop)void listen('request-settings',()=>{if(!disposed)view='settings';}).then(fn=>{if(disposed)fn();else unlisteners.push(fn);});
    if(isDesktop) for(const [event,quit] of [['request-hide',false],['request-quit',true]] as const) {
      void listen(event,()=>{if(!disposed)void leave(quit);}).then(fn=>{if(disposed)fn();else unlisteners.push(fn);}).catch(()=>{if(!disposed)notice='关闭事件暂不可用，请使用面板按钮';});
    }
    return ()=>{disposed=true;appDisposed=true;selectionCommands.dispose();for(const fn of unlisteners)fn();controller.dispose();selection.dispose();attachmentController.dispose();knowledge.dispose();chat.dispose();actionOutput.dispose();demo.stop();resizeQueue.clear();};
  });
</script>

<svelte:window onkeydown={keydown}/>
{#if surface.mode==='capsule'}
<Capsule retry={chatState.reply?.retry??null} {canRestoreQuestion} onRestoreQuestion={()=>void restoreQuestion()} {focusRequest} quickActions={quickConfig.actions} {actionBusy} onSelectionAction={id=>void selectAction(id)} onQuickSettings={()=>void openQuickSettings()} {actionOutputState} onActionOutputRetry={()=>void actionOutput.retry()} onActionOutputDismiss={()=>actionOutput.dismiss()} {selectionState} onSelectionRetry={()=>void selection.applyPending()} onSelectionDiscard={()=>void selection.discard()} onSelectionReload={()=>void selection.refresh()} attachments={attachmentState} onAttachmentCapture={()=>captureScreenshot(null)} onAttachmentCancel={()=>void attachmentController.cancel()} onAttachmentRemove={id=>void attachmentController.remove(id)} onAttachmentRetake={id=>captureScreenshot(id)} notice={selectionState.pending?.mode==='candidate'?notice:combinedNotice} {draft} {surface} {response} {running} {replyLabel} {replyStatus} modelStatus={settings.config?(settings.keyAvailable?'已配置模型':'需配置密钥'):'模型未配置'} canSend={!selectionState.busy&&!attachmentState.pending&&(!attachmentState.items.length||!!imageModelOf(settings.config))&&isDesktop&&chatState.loaded&&settings.status!=='loading'&&!chatState.running&&!chatState.stopping&&(!chatState.reply||chatState.reply.saved)} unsaved={!!chatState.reply&&!chatState.reply.saved&&!chatState.running&&chatState.reply.state!=='streaming'} busy={switching||closing} onEdit={s=>{if(!closing)controller.edit(s);}} onSave={save} onHide={()=>leave()} onExpand={()=>switchSurface('full')} onDemo={startDemo} onStop={stopReply} onSend={()=>void send()} onRetrySave={()=>void retryReplySave()} onResize={resize}/>
{:else}
<div class="shell">
  <header class="titlebar">
    <div class="brand"><BrandLogo/><span>Zwen</span><span class="edition">开发预览</span></div>
    <div class="window-actions"><button class="capsule-launch" disabled={switching} onclick={()=>switchSurface('capsule')}>快捷胶囊</button><span class="keyhint">Esc 收起</span><button class="icon-button" aria-label="收起面板" title="保存草稿并收起" onclick={()=>leave()}>−</button><details><summary aria-label="更多操作">···</summary><div class="menu"><button onclick={()=>leave(true)}>保存并退出</button></div></details></div>
  </header>
  {#if !isDesktop}<div class="preview-banner">浏览器交互预览 · 草稿仅保存在此浏览器；桌面版使用本机数据库</div>{/if}
  <div class="workspace">
    <nav aria-label="面板视图"><div class="tabs"><button class:active={view==='ask'} aria-pressed={view==='ask'} onclick={()=>view='ask'}>提问</button><button data-preview class:active={view==='preview'} aria-pressed={view==='preview'} onclick={()=>view='preview'}>格式预览</button>{#if chatState.reply||chatState.running||chatState.error}<button aria-pressed={view==='reply'&&responseKind==='model'} class:active={view==='reply'&&responseKind==='model'} onclick={()=>{responseKind='model';view='reply';}}>模型回复</button>{/if}{#if demoText}<button aria-pressed={view==='reply'&&responseKind==='demo'} class:active={view==='reply'&&responseKind==='demo'} onclick={()=>{responseKind='demo';view='reply';}}>演示回复</button>{/if}<button class:active={view==='history'} aria-pressed={view==='history'} onclick={()=>view='history'}>历史</button><button class:active={view==='knowledge'} aria-pressed={view==='knowledge'} onclick={()=>{view='knowledge';void knowledge.refresh();}}>知识库</button><button class:active={view==='graph'} aria-pressed={view==='graph'} onclick={()=>{view='graph';void knowledge.refresh();void knowledge.loadTopics();void knowledge.loadRelations();}}>知识网络</button><button disabled={chatState.running||chatState.stopping} onclick={newConversation}>新对话</button><button class:active={view==='settings'} aria-pressed={view==='settings'} onclick={()=>view='settings'}>模型设置</button></div><span class="model-status"><i></i> {settings.config?'模型已配置':'模型未配置'}</span></nav>
    <main>
      {#if view==='history'}
        <HistoryPanel port={historyPort} canContinue={chatState.loaded&&!chatState.running&&!chatState.stopping&&(!chatState.reply||chatState.reply.saved)} onContinue={continueHistory} focusRequestId={traceRequestId} onFocusConsumed={()=>traceRequestId=null}/>
      {:else if view==='graph'}
        <KnowledgeGraph controller={knowledge} info={knowledgeState} onOpenNote={id=>void knowledge.open(id).then(()=>{view='knowledge';})} onOpenTopic={id=>{view='knowledge';void knowledge.openTopic(id);}}/>
      {:else if view==='knowledge'}
        <KnowledgePanel controller={knowledge} info={knowledgeState} onTrace={traceToHistory} onSaveEdit={saveNoteEdit} modelLabel={settings.config?.model??'未配置模型'}/>
      {:else if view==='settings'}
        <ProviderSettings settings={settings} desktop={isDesktop} onEdit={(name,value)=>provider.edit(name,value)} onSave={()=>void provider.save().then(()=>knowledge.refresh())} onReload={()=>void provider.load()} onSaveSecret={value=>provider.saveSecret(value)} onClearSecret={()=>provider.clearSecret()} candidateEnabled={candidateEnabled} onCandidateToggle={onCandidateToggle}/>
        <QuickActionSettings port={quickActionPort} onChanged={config=>{quickConfig=config;quickConfigLoaded=true;}}/>
      {:else if view==='ask'}
        <section class="welcome"><div class="eyebrow">A LITTLE SPACE FOR CURIOSITY</div><h1>从一个问题开始<span>。</span></h1><p>留住阅读时的疑问，慢慢把知识连起来。</p><div class="orbit" aria-hidden="true"><span></span><span></span><span></span><b><svg viewBox="0 0 32 32" aria-hidden="true"><path d="M16 4v24M4 16h24M7.5 7.5l17 17m-17 0 17-17"/></svg></b></div></section>
        <section class="suggestions" aria-label="开始一个草稿"><button disabled={!draft.loaded} onclick={()=>fill('请帮我解释这段内容：\n\n')}><span class="suggestion-icon">Aa</span><strong>读懂一段内容</strong><span>换个角度，把意思讲清楚</span><b>↗</b></button><button disabled={!draft.loaded} onclick={()=>fill('我想理解这个公式：\n\n')}><span class="suggestion-icon">ƒ</span><strong>理解一个公式</strong><span>从符号到它背后的直觉</span><b>↗</b></button><button disabled={!draft.loaded} onclick={()=>fill('帮我梳理这个想法：\n\n')}><span class="suggestion-icon"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3c0 5-4 9-9 9 5 0 9 4 9 9 0-5 4-9 9-9-5 0-9-4-9-9Z"/></svg></span><strong>梳理一个想法</strong><span>先记下来，再找到联系</span><b>↗</b></button></section>
        <div class="connection-note"><span aria-hidden="true">◌</span><p>这里会成为你的问答空间。<br/><span>先连接你的模型。继续提问会附带本会话近期内容；切换服务请新建对话。</span></p></div>
      {:else}
        <section class="answer"><div class="answer-label"><span class="assistant-dot"></span><strong>{view==='reply'&&responseKind==='model'?chatState.reply?.model||'模型回复':'阅读效果'}</strong><span>{view==='reply' ? `${replyLabel} · ${replyStatus}` : draft.text.trim() ? '草稿排版预览 · 非 AI 回答' : '内置示例 · 非 AI 回答'}</span></div>{#if view==='reply'&&responseKind==='model'&&chatState.reply}<details class="question-summary"><summary>本轮问题 · 上下文 {chatState.reply.contextTurns} 轮</summary><p>{chatState.reply.prompt}</p></details>{/if}<div class="prose" role="presentation" onclick={e=>proseLinkClick(e,openExternal)} onkeydown={e=>proseLinkClick(e as unknown as MouseEvent,openExternal)}>{@html preview}</div>{#if view==='reply'&&running}<button class="text-button" data-stop onclick={stopReply}>{responseKind==='demo'?'停止演示':'停止生成'}</button>{/if}{#if view==='reply'&&responseKind==='model'&&chatState.reply&&chatState.reply.saved&&chatState.reply.answer&&!chatState.running}<button class="text-button" data-knowledge-save onclick={()=>void saveToKnowledge()}>收藏到知识库</button>{/if}{#if view==='reply'&&responseKind==='model'&&!response&&chatState.running}<p>正在等待模型回复…</p>{/if}</section>
      {/if}
    </main>
    <ActionOutputNotice state={actionOutputState} onRetry={()=>void actionOutput.retry()} onDismiss={()=>actionOutput.dismiss()}/>
        <SelectionNotice actions={quickConfig.actions} {actionBusy} onAction={id=>void selectAction(id)} onSettings={()=>void openQuickSettings()} state={selectionState} onRetry={()=>void selection.applyPending()} onDiscard={()=>void selection.discard()} onReload={()=>void selection.refresh()}/>
    {#if responseKind==='model'&&chatState.running&&chatState.reply?.retry}<RetryNotice retry={chatState.reply.retry}/>{/if}
    {#if canRestoreQuestion&&(view==='reply'||view==='ask')}<button class="text-button" data-restore-question onclick={()=>void restoreQuestion()}>恢复问题，重新提问</button>{/if}
    {#if responseKind==='model'&&chatState.error}<div class="error" role="alert"><strong>{chatState.error}</strong>{#if !chatState.loaded}<button onclick={()=>void chat.load()}>重新读取对话</button>{/if}{#if chatState.reply&&!chatState.reply.saved&&!chatState.running}<button onclick={()=>void retryReplySave()}>重试保存回答</button>{/if}</div>{/if}
    {#if draft.error}<div class="error" role="alert"><strong>{draft.error}</strong><span>原有文件不会被自动替换。</span><button onclick={()=>draft.loaded ? save() : controller.start().then(()=>selection.refresh()).catch(()=>{})}>{draft.loaded ? '重试保存' : '重新读取'}</button></div>{/if}
    {#if view!=='settings'&&view!=='history'}<section class="composer" aria-label="问题草稿">
      <Attachments state={attachmentState} onCancel={()=>void attachmentController.cancel()} onRemove={id=>void attachmentController.remove(id)} onRetake={id=>captureScreenshot(id)}/>
      <label class="sr-only" for="question">问题草稿</label><textarea bind:this={input} id="question" disabled={!draft.loaded||closing} value={draft.text} oninput={e=>controller.edit(e.currentTarget.value)} placeholder="写下问题，或粘贴一段想理解的内容……" spellcheck="false" rows="4"></textarea>
      <div class="composer-tools"><div><span class="local-label"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m8 13 6-6a3 3 0 0 1 4 4l-8 8a5 5 0 0 1-7-7l9-9"/></svg> 本地草稿</span>{#if draft.text}<button class="text-button" onclick={()=>{cleared=draft.text;controller.edit('');}}>清空</button>{:else if cleared!==null}<button class="text-button" onclick={()=>{controller.edit(cleared!);cleared=null;}}>撤销清空</button>{/if}</div><button class="text-button" data-capture disabled={!isDesktop||!draft.loaded||attachmentState.pending||attachmentState.items.length>=attachmentState.max} onclick={()=>captureScreenshot(null)}>截图</button><button class="text-button" aria-label="保存草稿" title="Ctrl S 保存草稿" disabled={!draft.loaded} onclick={save}>保存</button><button class="primary-button" data-send disabled={selectionState.busy||attachmentState.pending||(attachmentState.items.length>0&&!imageModelOf(settings.config))||!isDesktop||!draft.loaded||!chatState.loaded||settings.status==='loading'||chatState.running||chatState.stopping||(!!chatState.reply&&!chatState.reply.saved)||!draft.text.trim()} onclick={()=>void send()}>{chatState.stopping?'正在保存…':'发送'} <kbd>Ctrl ↵</kbd></button></div>
    </section>
    {/if}
    <footer><span class:error-status={draft.status==='error'}><i class:saved={draft.status==='saved'}></i>{statusText}</span><span>{notice || '不采集当前应用内容 · 不自动上传'}</span></footer>
  </div>
</div>

{/if}
