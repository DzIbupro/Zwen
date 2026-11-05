<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import Icon from './Icon.svelte';
  import AppearanceSettings from './AppearanceSettings.svelte';
  import {AppearanceController,initialAppearance,applyAppearance,watchSystemTheme,type AppearanceState} from './lib/appearance';
  import {appearancePort} from './lib/bridge';
  import RetryNotice from './RetryNotice.svelte';
  import ActionOutputNotice from './ActionOutputNotice.svelte';
  import QuickActionSettings from './QuickActionSettings.svelte';
  import {defaultQuickActions,type QuickActionConfig} from './lib/quick-actions';
  import {ActionOutputController,type ActionOutputState} from './lib/action-output';
  import {quickActionPort,actionOutputPort} from './lib/bridge';
  import {QuickSession} from './lib/quick-session';
  import {SendDraftCoordinator} from './lib/send-draft';
  import BrandLogo from './BrandLogo.svelte';
  import WelcomeMark from './WelcomeMark.svelte';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { DraftController, type Snapshot } from './lib/draft';
  import { draftPort, isDesktop, hideWindow, quitApp } from './lib/bridge';
  import { renderMarkdown } from './lib/render';
  import Capsule from './Capsule.svelte';
  import {getSurface,setSurface,resizeCapsule} from './lib/bridge';
  import {canApplySurface,ensureSurface,ResizeQueue,type SurfaceInfo} from './lib/surface';
  import {ChatController,type ChatState} from './lib/chat';
  import {StartupSession} from './lib/startup';
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
  const initialUi=initialAppearance((window as Window & {__ZWEN_APPEARANCE__?:unknown}).__ZWEN_APPEARANCE__,{getItem:key=>localStorage.getItem(key)});
  let appearanceState=$state<AppearanceState>({...initialUi,loaded:false,status:'loading',error:null});
  const appearance=new AppearanceController(appearancePort,s=>{appearanceState=s;applyAppearance(s);},initialUi);
  function navigate(destination:'chat'|'history'|'knowledge'|'graph'|'settings'){
    if(closing)return;
    if(destination==='chat'){view=chatState.reply||chatState.running||chatState.error?'reply':'ask';}
    else{view=destination;if(destination==='knowledge')void knowledge.refresh();if(destination==='graph'){void knowledge.refresh();void knowledge.loadTopics();void knowledge.loadRelations();}}
  }
  let quickConfig=$state<QuickActionConfig>({revision:0,actions:defaultQuickActions(),outputDirectory:''});
  let actionBusy=$state(false);
  let focusRequest=$state(0);
  const quickSession=new QuickSession();
  let quickConfigLoaded=$state(false);
  let actionOutputState=$state<ActionOutputState|null>(null);
  const actionOutput=new ActionOutputController(actionOutputPort,s=>actionOutputState=s);
  let notice=$state('');
  let bookmarkSaving=$state<string|null>(null);let bookmarkSaved=$state<string|null>(null);let bookmarkTimer:ReturnType<typeof setTimeout>|undefined;
  let view=$state<'ask'|'reply'|'settings'|'history'|'knowledge'|'graph'>('ask');
  let settings=$state<SettingsState>({config:null,revision:0,keyAvailable:false,keySource:null,fields:{baseUrl:'',model:'',keyEnv:'ZWEN_API_KEY',imageModel:''},status:'loading',loaded:false,error:null,secretSaving:false,secretNotice:null});
  const provider=new ProviderController(providerPort,s=>settings=s);
  let surface=$state<SurfaceInfo>({mode:'full',scaleFactor:1,generation:0,bounds:{width:480,initialHeight:96,maxHeight:594}});
  let switching=$state(false);let closing=$state(false);
  const startupSession=new StartupSession((window as Window & {__ZWEN_RUN_ID__?:unknown}).__ZWEN_RUN_ID__,{getItem:key=>sessionStorage.getItem(key),setItem:(key,value)=>sessionStorage.setItem(key,value)});
  let chatState=$state<ChatState>({answer:'',running:false,stopping:false,loaded:false,error:null,reply:null});
  const chat=new ChatController(chatPort,s=>{const appeared=!chatState.reply&&s.reply;chatState=s;if(s.loaded)startupSession.markLoaded(s.reply!==null);settlePendingSend();actionOutput.observe(s.reply);if(appeared&&view==='ask'){view='reply';}},{freshStart:startupSession.fresh});
  const response=$derived(chatState.answer);
  const running=$derived(chatState.running);
  const replyStatus=$derived(chatState.reply?.retry?'等待重试':chatState.reply?({streaming:'生成中',complete:'已完成',stopped:'已停止',failed:'生成失败',interrupted:'上次中断',limited:'已达限制',filtered:'服务限制'}[chatState.reply.state]):chatState.running?'等待回复':'');
  const replyLabel='模型回复';
  const combinedNotice=$derived(chatState.error||notice);

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
    view='reply';const sent=draft.text;const sentIds=attachmentState.items.map(item=>item.id);
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
      actionBusy=true;try{await selection.applyPending(expected?.item);if(!appDisposed&&!selectionState.pending){chat.newConversation();view='ask';focusRequest++;await switchSurface('capsule');}}finally{actionBusy=false;}return;
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
      view='reply';
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
  function newConversation(){if(closing)return;if(chat.newConversation()){quickSession.release();view='ask';notice='已准备新对话，当前草稿保留';}}
  function continueHistory(reply:ChatReply){if(closing)return;if(chat.adopt(reply)){quickSession.release();view='reply';notice='继续此会话将附带近期问答';}}
  function stopReply(){void chat.stop().catch(()=>{});}
  const canRestoreQuestion=$derived(!chatState.running&&!chatState.stopping&&!!chatState.reply?.saved&&['failed','stopped','interrupted','limited'].includes(chatState.reply.state));
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
    if(!reply||!reply.answer||chatState.running||bookmarkSaving)return;
    bookmarkSaving=reply.requestId;
    const title=reply.prompt.replace(/\s+/g,' ').trim().slice(0,60);
    const body=`## 问题\n\n${reply.prompt}\n\n## 回答\n\n${reply.answer}`;
    try{
      const saved=await knowledge.save({title,body,source:'chat',model:reply.model,attachmentIds:attachmentState.items.map(item=>item.id),requestId:reply.requestId});
      if(saved){clearTimeout(bookmarkTimer);bookmarkSaved=reply.requestId;bookmarkTimer=setTimeout(()=>{bookmarkSaved=null;},2500);notice='已收藏到知识库';void knowledge.runAutoFor(saved.id);}
      else notice=knowledge.state.notice||'收藏未完成，请重试';
    }catch{notice='收藏未完成，请重试';}finally{bookmarkSaving=null;}
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
  const statusText = $derived({loading:'正在读取草稿',saved:'已保存到本机',dirty:'等待保存',saving:'正在保存',error:'保存未完成'}[draft.status]);
  const renderedReply=$derived(renderMarkdown(response));
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
      await appearance.flush();
      if(endQuick){
        let recoveryDraft='';
        await quickSession.close({
          settle:async()=>{await chat.settle();if(chatState.reply&&!chatState.reply.saved)throw Error('回答尚未保存，请重试保存后再关闭');if(actionOutputState?.stage==='saving')throw Error('文档正在保存，请稍后关闭');},
          revokeSend:()=>sendDraft.revoke(),
          discardSelection:id=>selection.discardOwned(id),
          clearMaterials:async()=>{const ids=attachmentState.items.map(item=>item.id);for(const id of ids)await attachmentPort.remove(id);await attachmentController.load();},
          clearDraft:async()=>{const text=controller.snapshot.text;recoveryDraft=text;controller.edit('');try{await controller.flush();}catch(error){controller.edit(text);throw error;}},
          restoreDraft:async()=>{controller.edit(recoveryDraft);await controller.flush();},
          resetView:()=>{if(!chat.newConversation())throw Error('对话尚未结束，请稍后重试');actionOutput.dismiss();view='ask';notice='';cleared=null;}
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
    const unwatchTheme=watchSystemTheme(()=>appearanceState);void appearance.start();void provider.load();void chat.load();
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
    return ()=>{unwatchTheme();clearTimeout(bookmarkTimer);disposed=true;appDisposed=true;selectionCommands.dispose();appearance.dispose();for(const fn of unlisteners)fn();controller.dispose();selection.dispose();attachmentController.dispose();knowledge.dispose();chat.dispose();actionOutput.dispose();resizeQueue.clear();};
  });
</script>

<svelte:window onkeydown={keydown}/>
{#if surface.mode==='capsule'}
<Capsule retry={chatState.reply?.retry??null} {canRestoreQuestion} onRestoreQuestion={()=>void restoreQuestion()} {focusRequest} quickActions={quickConfig.actions} {actionBusy} onSelectionAction={id=>void selectAction(id)} onQuickSettings={()=>void openQuickSettings()} {actionOutputState} onActionOutputRetry={()=>void actionOutput.retry()} onActionOutputDismiss={()=>actionOutput.dismiss()} {selectionState} onSelectionRetry={()=>void selection.applyPending()} onSelectionDiscard={()=>void selection.discard()} onSelectionReload={()=>void selection.refresh()} attachments={attachmentState} onAttachmentCapture={()=>captureScreenshot(null)} onAttachmentCancel={()=>void attachmentController.cancel()} onAttachmentRemove={id=>void attachmentController.remove(id)} onAttachmentRetake={id=>captureScreenshot(id)} notice={selectionState.pending?.mode==='candidate'?notice:combinedNotice} {draft} {surface} {response} {running} {replyLabel} {replyStatus} modelStatus={settings.config?(settings.keyAvailable?'已配置模型':'需配置密钥'):'模型未配置'} canSend={!selectionState.busy&&!attachmentState.pending&&(!attachmentState.items.length||!!imageModelOf(settings.config))&&isDesktop&&chatState.loaded&&settings.status!=='loading'&&!chatState.running&&!chatState.stopping&&(!chatState.reply||chatState.reply.saved)} unsaved={!!chatState.reply&&!chatState.reply.saved&&!chatState.running&&chatState.reply.state!=='streaming'} busy={switching||closing} onEdit={s=>{if(!closing)controller.edit(s);}} onSave={save} onHide={()=>leave()} onExpand={()=>switchSurface('full')} onStop={stopReply} onSend={()=>void send()} onRetrySave={()=>void retryReplySave()} onResize={resize}/>
{:else}
<div class="shell" class:sidebar-collapsed={appearanceState.sidebarCollapsed}>
  <Sidebar collapsed={appearanceState.sidebarCollapsed} active={view==='history'?'history':view==='knowledge'?'knowledge':view==='graph'?'graph':view==='settings'?'settings':'chat'} disabled={!appearanceState.loaded} canNew={!closing&&!chatState.running&&!chatState.stopping} onToggle={()=>void appearance.update({sidebarCollapsed:!appearanceState.sidebarCollapsed})} onNavigate={navigate} onNew={newConversation}/>
  <div class="main-panel">
  <header class="titlebar">
    <span class="context-title">{view==='settings'?'设置':view==='history'?'问答历史':view==='knowledge'?'知识库':view==='graph'?'知识网络':'当前对话'}</span>
    <div class="window-actions"><button class="capsule-launch" title="打开快捷胶囊" disabled={switching} onclick={()=>switchSurface('capsule')}><Icon name="external" size={18}/><span>快捷胶囊</span></button><button class="icon-button" aria-label="收起面板" title="保存并收起" onclick={()=>leave()}><Icon name="minus"/></button><button class="quit-button icon-button" data-quit-app aria-label="保存并退出" title={closing?'正在退出…':'保存并退出'} disabled={closing} onclick={()=>leave(true)}><Icon name="close"/></button></div>
  </header>
  {#if !isDesktop}<div class="preview-banner">浏览器界面预览 · 不会请求模型</div>{/if}
  {#if appearanceState.error&&view!=='settings'}<div class="appearance-save-error" role="alert">{appearanceState.error}<button onclick={()=>view='settings'}>外观设置</button></div>{/if}
  <div class="workspace">
    <main>
      {#if view==='history'}
        <HistoryPanel port={historyPort} canContinue={chatState.loaded&&!chatState.running&&!chatState.stopping&&(!chatState.reply||chatState.reply.saved)} onContinue={continueHistory} focusRequestId={traceRequestId} onFocusConsumed={()=>traceRequestId=null}/>
      {:else if view==='graph'}
        <KnowledgeGraph controller={knowledge} info={knowledgeState} onOpenNote={id=>void knowledge.open(id).then(()=>{view='knowledge';})} onOpenTopic={id=>{view='knowledge';void knowledge.openTopic(id);}}/>
      {:else if view==='knowledge'}
        <KnowledgePanel controller={knowledge} info={knowledgeState} onTrace={traceToHistory} onSaveEdit={saveNoteEdit} modelLabel={settings.config?.model??'未配置模型'}/>
      {:else if view==='settings'}
        <AppearanceSettings state={appearanceState} onTheme={theme=>void appearance.update({theme})} onReload={()=>void appearance.start()}/>
        <ProviderSettings settings={settings} desktop={isDesktop} onEdit={(name,value)=>provider.edit(name,value)} onSave={()=>void provider.save().then(()=>knowledge.refresh())} onReload={()=>void provider.load()} onSaveSecret={value=>provider.saveSecret(value)} onClearSecret={()=>provider.clearSecret()} candidateEnabled={candidateEnabled} onCandidateToggle={onCandidateToggle}/>
        {#if isDesktop}<QuickActionSettings port={quickActionPort} onChanged={config=>{quickConfig=config;quickConfigLoaded=true;}}/>{/if}
      {:else if view==='ask'}
        <div class="empty-conversation"><div class="welcome-scene"><WelcomeMark/><p>今天想问 Zwen 什么问题？</p></div></div>
      {:else}
        <section class="answer">{#if view==='reply'&&chatState.reply}<div class="user-question"><p>{chatState.reply.prompt}</p></div>{/if}<div class="answer-label"><BrandLogo compact/><strong>{chatState.reply?.model||'模型回复'}</strong><span>{replyLabel} · {replyStatus}</span></div><div class="prose" role="presentation" onclick={e=>proseLinkClick(e,openExternal)} onkeydown={e=>proseLinkClick(e as unknown as MouseEvent,openExternal)}>{@html renderedReply}</div>{#if view==='reply'&&running}<button class="text-button" data-stop onclick={stopReply}>停止生成</button>{/if}{#if view==='reply'&&chatState.reply&&chatState.reply.saved&&chatState.reply.answer&&!chatState.running}<button class="text-button" data-knowledge-save disabled={bookmarkSaving!==null} aria-live="polite" onclick={()=>void saveToKnowledge()}>{bookmarkSaving===chatState.reply.requestId?'正在收藏…':bookmarkSaved===chatState.reply.requestId?'已收藏 ✓':'收藏到知识库'}</button>{/if}{#if view==='reply'&&!response&&chatState.running}<p>正在等待模型回复…</p>{/if}</section>
      {/if}
    </main>
    <ActionOutputNotice state={actionOutputState} onRetry={()=>void actionOutput.retry()} onDismiss={()=>actionOutput.dismiss()}/>
        <SelectionNotice actions={quickConfig.actions} {actionBusy} onAction={id=>void selectAction(id)} onSettings={()=>void openQuickSettings()} state={selectionState} onRetry={()=>void selection.applyPending()} onDiscard={()=>void selection.discard()} onReload={()=>void selection.refresh()}/>
    {#if chatState.running&&chatState.reply?.retry}<RetryNotice retry={chatState.reply.retry}/>{/if}
    {#if canRestoreQuestion&&(view==='reply'||view==='ask')}<button class="text-button" data-restore-question onclick={()=>void restoreQuestion()}>恢复问题，重新提问</button>{/if}
    {#if chatState.error}<div class="error" role="alert"><strong>{chatState.error}</strong>{#if !chatState.loaded}<button onclick={()=>void chat.load()}>重新读取对话</button>{/if}{#if chatState.reply&&!chatState.reply.saved&&!chatState.running}<button onclick={()=>void retryReplySave()}>重试保存回答</button>{/if}</div>{/if}
    {#if draft.error}<div class="error" role="alert"><strong>{draft.error}</strong><span>原有文件不会被自动替换。</span><button onclick={()=>draft.loaded ? save() : controller.start().then(()=>selection.refresh()).catch(()=>{})}>{draft.loaded ? '重试保存' : '重新读取'}</button></div>{/if}
    {#if view!=='settings'&&view!=='history'}<section class="composer" aria-label="问题草稿">
      <Attachments state={attachmentState} onCancel={()=>void attachmentController.cancel()} onRemove={id=>void attachmentController.remove(id)} onRetake={id=>captureScreenshot(id)}/>
      <label class="sr-only" for="question">问题草稿</label><textarea bind:this={input} id="question" disabled={!draft.loaded||closing} value={draft.text} oninput={e=>controller.edit(e.currentTarget.value)} placeholder="输入问题，或粘贴内容…" spellcheck="false" rows="2"></textarea>
      <div class="composer-tools"><div><button class="tool-button" data-capture aria-label="附加截图" title="附加截图" disabled={!isDesktop||!draft.loaded||attachmentState.pending||attachmentState.items.length>=attachmentState.max} onclick={()=>captureScreenshot(null)}><Icon name="image"/></button><button class="tool-button" aria-label="保存草稿" title="保存草稿 · Ctrl S" disabled={!draft.loaded} onclick={save}><Icon name="save" size={18}/></button>{#if draft.text}<button class="text-button" onclick={()=>{cleared=draft.text;controller.edit('');}}>清空</button>{:else if cleared!==null}<button class="text-button" onclick={()=>{controller.edit(cleared!);cleared=null;}}>撤销清空</button>{/if}</div><button class="primary-button" data-send title="Enter 发送 · Shift+Enter 换行" disabled={selectionState.busy||attachmentState.pending||(attachmentState.items.length>0&&!imageModelOf(settings.config))||!isDesktop||!draft.loaded||!chatState.loaded||settings.status==='loading'||chatState.running||chatState.stopping||(!!chatState.reply&&!chatState.reply.saved)||!draft.text.trim()} onclick={()=>void send()}><span class="sr-only">{chatState.stopping?'正在保存…':'发送'}</span><Icon name="send"/><kbd>Enter ↵</kbd></button></div>
    </section>
    {/if}
    <footer><span class:error-status={draft.status==='error'}><i class:saved={draft.status==='saved'}></i>{statusText}</span><span role="status">{notice}</span></footer>
  </div>
  </div>
</div>

{/if}
