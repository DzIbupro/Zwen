<script lang="ts">
 import {onMount,untrack} from 'svelte';
 import BrandLogo from './BrandLogo.svelte';
 import Icon from './Icon.svelte';
 import {copyTextToClipboard,handleProseClick,renderMarkdown} from './lib/render';import {openExternal} from './lib/bridge';
 import {HistoryController,type HistoryPort,type HistoryState,type Summary} from './lib/history';import type {ChatReply} from './lib/chat';
 import {getLocale,t} from './lib/i18n.svelte';
 import {
  archiveRecords,
  assignToProject,
  cleanupDeletedRecords,
  createProject,
  deleteProject,
  detachFromProject,
  getDisplayTitle,
  getProjectById,
  getProjectIdFor,
  getProjects,
  isArchived,
  isPinned,
  pinRank,
  renameProject,
  renameRecord,
  setPinned,
  togglePin,
 } from './lib/history-org.svelte';

 let {port,canContinue,canDelete=true,onContinue,onDeleted,onBulkDeleted,onClearedAll,onNewInProject,currentRequestId=null,focusRequestId,onFocusConsumed}:{port:HistoryPort;canContinue:boolean;canDelete?:boolean;onDeleted?:(id:string)=>void;onBulkDeleted?:(ids:string[])=>void;onClearedAll?:()=>void;onNewInProject?:(projectId:string)=>void;currentRequestId?:string|null;onContinue:(reply:ChatReply)=>void;focusRequestId?:string|null;onFocusConsumed?:()=>void}=$props();

 let info=$state<HistoryState>({items:[],selected:null,page:0,hasMore:false,loading:false,deleting:false,checked:{},notice:null,error:null});
 let localNotice=$state<string|null>(null);
 let clearingReply:string|null=null;
 let query=$state('');
 let editMode=$state(false);
 let projectFilter=$state<'all'|'unassigned'|string>('all');
 let creatingProject=$state(false);
 let newProjectName=$state('');
 let renamingProject=$state(false);
 let projectRenameDraft=$state('');
 let renamingDetail=$state(false);
 let detailTitleDraft=$state('');
 let bulkProjectMenuOpen=$state(false);
 let bulkProjectMenuWrap=$state<HTMLDivElement>();
 let detailProjectMenuOpen=$state(false);
 let detailProjectMenuWrap=$state<HTMLDivElement>();
 let detailCopied=$state(false);
 let detailCopyTimer:ReturnType<typeof setTimeout>|undefined;
 let pendingAssignTarget=$state<'bulk'|'detail'|null>(null);

 const history=new HistoryController(
  untrack(()=>port),
  s=>info=s,
  id=>{cleanupDeletedRecords([id]);onDeleted?.(id);},
  ids=>{
   if(ids===null){
    cleanupDeletedRecords(null);
    onClearedAll?.();
    onBulkDeleted?.(clearingReply?[clearingReply]:[]);
   }else{
    cleanupDeletedRecords(ids);
    onBulkDeleted?.(ids);
   }
  }
 );

 type DeleteRequest={kind:'single';id:string}|{kind:'selected';ids:string[]}|{kind:'all'};
 let deletion=$state<DeleteRequest|null>(null);

 const projects=$derived(getProjects());
 const activeFilterProject=$derived(projectFilter!=='all'&&projectFilter!=='unassigned'?getProjectById(projectFilter):null);

 $effect(()=>{
  if(projectFilter!=='all'&&projectFilter!=='unassigned'&&!projects.some(p=>p.id===projectFilter)){
   projectFilter='all';
   renamingProject=false;
  }
 });

 // Non-archived items in current page, filtered by project and search, with pinned items floated to top
 const nonArchivedItems=$derived(info.items.filter(r=>!isArchived(r.requestId)));

 const visibleItems=$derived.by(()=>{
  const q=query.trim().toLowerCase();
  const filtered=nonArchivedItems.filter(r=>{
   const pid=getProjectIdFor(r.requestId,r.conversationId);
   if(projectFilter==='unassigned'&&pid!==null)return false;
   if(projectFilter!=='all'&&projectFilter!=='unassigned'&&pid!==projectFilter)return false;
   if(!q)return true;
   const title=getDisplayTitle(r.requestId,r.title).toLowerCase();
   return title.includes(q)||r.title.toLowerCase().includes(q)||r.model.toLowerCase().includes(q);
  });
  return [...filtered].sort((a,b)=>{
   const pa=pinRank(a.requestId),pb=pinRank(b.requestId);
   if(pa!==-1&&pb!==-1)return pa-pb;
   if(pa!==-1)return -1;
   if(pb!==-1)return 1;
   return 0;
  });
 });

 const checkedIds=$derived(Object.keys(info.checked).filter(id=>!isArchived(id)));
 const checkedCount=$derived(checkedIds.length);
 const checkedItems=$derived(info.items.filter(r=>checkedIds.includes(r.requestId)));
 const allCheckedPinned=$derived(checkedCount>0&&checkedIds.every(id=>isPinned(id)));
 const anyCheckedInProject=$derived(checkedItems.some(r=>getProjectIdFor(r.requestId,r.conversationId)!==null));
 const selectableVisible=$derived(visibleItems.filter(r=>r.state!=='streaming'));
 const pageChecked=$derived(selectableVisible.length>0&&selectableVisible.every(r=>info.checked[r.requestId]));

 const selectedTitle=$derived(info.selected?getDisplayTitle(info.selected.requestId,info.selected.prompt):'');
 const selectedProjectId=$derived(info.selected?getProjectIdFor(info.selected.requestId,info.selected.conversationId):null);
 const selectedPinned=$derived(info.selected?isPinned(info.selected.requestId):false);
 const answer=$derived(info.selected?renderMarkdown(info.selected.answer):'');
 const labels=$derived<Record<ChatReply['state'],string>>({streaming:t('reply.streaming'),complete:t('reply.complete'),stopped:t('reply.stopped'),failed:t('reply.failed'),interrupted:t('reply.interrupted'),limited:t('reply.limited'),filtered:t('reply.filtered')});
 const shortDate=(ts:number)=>new Date(ts*1000).toLocaleString(getLocale(),{month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit'});

 function flashNotice(msg:string){
  localNotice=msg;
 }

 function open(id:string){
  deletion=null;
  renamingDetail=false;
  detailProjectMenuOpen=false;
  detailCopied=false;
  void history.open(id);
 }

 function copyDetailAnswer(){
  if(!info.selected?.answer)return;
  void copyTextToClipboard(info.selected.answer).then(ok=>{
   if(!ok)return;
   clearTimeout(detailCopyTimer);
   detailCopied=true;
   detailCopyTimer=setTimeout(()=>{detailCopied=false;},1600);
  });
 }

 function load(page:number){
  deletion=null;
  void history.load(page);
 }

 function toggleEditMode(){
  editMode=!editMode;
  deletion=null;
  bulkProjectMenuOpen=false;
  if(!editMode){
   history.clearChecked();
  }
 }
 function closeBulkMenuOnOutside(e:PointerEvent){
  if(bulkProjectMenuOpen&&bulkProjectMenuWrap&&e.target instanceof Node&&!bulkProjectMenuWrap.contains(e.target)){
   bulkProjectMenuOpen=false;
  }
  if(detailProjectMenuOpen&&detailProjectMenuWrap&&e.target instanceof Node&&!detailProjectMenuWrap.contains(e.target)){
   detailProjectMenuOpen=false;
  }
 }

 function toggleVisiblePage(){
  if(info.loading||info.deleting||!selectableVisible.length)return;
  const allSelected=pageChecked;
  for(const item of selectableVisible){
   const isChecked=!!info.checked[item.requestId];
   if(allSelected&&isChecked)history.toggleChecked(item.requestId);
   else if(!allSelected&&!isChecked)history.toggleChecked(item.requestId);
  }
 }

 function handleRowClick(item:Summary){
  if(editMode){
   if(item.state!=='streaming')history.toggleChecked(item.requestId);
  }else{
   open(item.requestId);
  }
 }

 function confirmDelete(){
  const target=deletion;deletion=null;if(!target||!canDelete)return;
  if(target.kind==='all'){clearingReply=currentRequestId;void history.clearAll();}
  else if(target.kind==='selected')void history.removeMany(target.ids);
  else if(info.selected?.requestId===target.id)void history.removeSelected();
 }

 function bulkPinToggle(){
  if(!checkedCount)return;
  const nextPin=!allCheckedPinned;
  setPinned(checkedIds,nextPin);
  history.clearChecked();
 }

 function bulkAssignProject(projectId:string){
  if(projectId==='__new__'){
   pendingAssignTarget='bulk';
   creatingProject=true;
   newProjectName='';
   return;
  }
  if(!projectId||!checkedCount)return;
  const proj=getProjectById(projectId);
  if(!proj)return;
  const targets=checkedIds.map(id=>{
   const found=info.items.find(r=>r.requestId===id);
   return {
    requestId:id,
    conversationId:found?.conversationId,
    title:found?.title,
    model:found?.model,
    createdAt:found?.createdAt,
   };
  });
  assignToProject(targets,projectId);
  history.clearChecked();
  flashNotice(t('history.assignedNotice',{name:proj.name}));
 }

 function bulkDetachProject(){
  if(!checkedCount)return;
  const targets=checkedIds.map(id=>{
   const found=info.items.find(r=>r.requestId===id);
   return {requestId:id,conversationId:found?.conversationId};
  });
  detachFromProject(targets);
  history.clearChecked();
  flashNotice(t('history.detachedNotice'));
 }

 function bulkArchive(){
  if(!checkedCount)return;
  const targets=info.items.filter(r=>checkedIds.includes(r.requestId));
  if(!targets.length)return;
  const count=archiveRecords(targets);
  if(info.selected&&checkedIds.includes(info.selected.requestId)){
   info={...info,selected:null};
  }
  history.clearChecked();
  flashNotice(t('history.archivedBatchNotice',{count}));
 }

 function archiveSelectedDetail(){
  const sel=info.selected;
  if(!sel||sel.state==='streaming')return;
  archiveRecords([{requestId:sel.requestId,conversationId:sel.conversationId,title:sel.prompt,model:sel.model,createdAt:sel.createdAt}]);
  info={...info,selected:null};
  renamingDetail=false;
  flashNotice(t('history.archivedNotice'));
 }

 function changeDetailProject(projectId:string){
  const sel=info.selected;
  if(!sel)return;
  if(projectId==='__new__'){
   pendingAssignTarget='detail';
   creatingProject=true;
   newProjectName='';
   return;
  }
  if(!projectId){
   detachFromProject([{requestId:sel.requestId,conversationId:sel.conversationId}]);
   flashNotice(t('history.detachedNotice'));
  }else{
   const proj=getProjectById(projectId);
   assignToProject([{
    requestId:sel.requestId,
    conversationId:sel.conversationId,
    title:sel.prompt,
    model:sel.model,
    createdAt:sel.createdAt,
   }],projectId);
   if(proj)flashNotice(t('history.assignedNotice',{name:proj.name}));
  }
 }

 function startRenameDetail(){
  if(!info.selected)return;
  detailTitleDraft=selectedTitle;
  renamingDetail=true;
 }

 function saveRenameDetail(e?:Event){
  e?.preventDefault();
  if(!info.selected)return;
  renameRecord(info.selected.requestId,detailTitleDraft);
  renamingDetail=false;
 }

 function submitNewProject(e:Event){
  e.preventDefault();
  const created=createProject(newProjectName);
  if(created){
   const assignMode=pendingAssignTarget;
   pendingAssignTarget=null;
   newProjectName='';
   creatingProject=false;
   if(assignMode==='bulk'&&checkedCount>0){
    bulkAssignProject(created.id);
   }else if(assignMode==='detail'&&info.selected){
    changeDetailProject(created.id);
   }else{
    projectFilter=created.id;
   }
  }
 }

 function submitRenameProject(e:Event){
  e.preventDefault();
  if(activeFilterProject&&renameProject(activeFilterProject.id,projectRenameDraft)){
   renamingProject=false;
   projectRenameDraft='';
  }
 }

 function handleDeleteProject(){
  if(!activeFilterProject)return;
  deleteProject(activeFilterProject.id);
  projectFilter='all';
  renamingProject=false;
 }

 $effect(()=>{
  if(!localNotice)return;
  const timer=setTimeout(()=>{localNotice=null;},3500);
  return()=>clearTimeout(timer);
 });

 onMount(()=>{void history.load(0);return()=>history.dispose();});
 $effect(()=>{const id=focusRequestId;if(!id)return;deletion=null;renamingDetail=false;onFocusConsumed?.();untrack(()=>void history.open(id));});
</script>

<svelte:window onpointerdown={closeBulkMenuOnOutside}/>
<section class="history" aria-label={t('history.title')}>
 {#if localNotice||info.notice}<div role="status" class="banner-notice">{localNotice||info.notice}</div>{/if}
 {#if info.error}<div class="banner-error" role="alert">{info.error}</div>{/if}
 {#if deletion}
  <div class="delete-confirm" role="alertdialog" aria-labelledby="history-delete-title">
   <div class="delete-copy">
    <h2 id="history-delete-title">{deletion.kind==='all'?t('history.deleteAllTitle'):deletion.kind==='selected'?t('history.deleteSelectedTitle',{count:deletion.ids.length}):t('history.deleteSingleTitle')}</h2>
    <p>{deletion.kind==='all'?t('history.deleteAllDesc'):t('history.deleteOneDesc')}</p>
   </div>
   <div class="delete-actions">
    <button disabled={info.deleting} onclick={()=>deletion=null}>{t('history.cancel')}</button>
    <button class="danger-solid" data-history-confirm-delete disabled={!canDelete||info.deleting} onclick={confirmDelete}>{t('history.confirmDelete')}</button>
   </div>
  </div>
 {/if}

 <div class="history-split">
  <aside class="history-sidebar" aria-label={t('history.title')}>
   <div class="sidebar-top">
    <div class="sidebar-head">
     <h1>{t('history.title')}</h1>
     <div class="head-ops">
      <button class="ghost-btn" title={t('history.refreshTitle')} disabled={info.loading||info.deleting} onclick={()=>load(0)}>{t('history.refresh')}</button>
      <button class="ghost-btn" class:edit-active={editMode} data-history-edit aria-pressed={editMode} disabled={info.loading||info.deleting} onclick={toggleEditMode}>{editMode?t('history.done'):t('history.edit')}</button>
     </div>
    </div>

    <div class="project-tabs" role="tablist" aria-label={t('history.projects')}>
     <button type="button" role="tab" class="proj-chip" class:active={projectFilter==='all'} aria-selected={projectFilter==='all'} data-project-filter="all" onclick={()=>{projectFilter='all';renamingProject=false;}}>{t('history.allChats')}</button>
     <button type="button" role="tab" class="proj-chip" class:active={projectFilter==='unassigned'} aria-selected={projectFilter==='unassigned'} data-project-filter="unassigned" onclick={()=>{projectFilter='unassigned';renamingProject=false;}}>{t('history.unassigned')}</button>
     {#each projects as proj (proj.id)}
      <button type="button" role="tab" class="proj-chip" class:active={projectFilter===proj.id} aria-selected={projectFilter===proj.id} data-project-filter={proj.id} onclick={()=>{projectFilter=proj.id;renamingProject=false;}}>{proj.name}</button>
     {/each}
     <button type="button" class="proj-chip add-chip" data-new-project title={t('history.newProject')} onclick={()=>{creatingProject=!creatingProject;newProjectName='';}}>+ {t('history.newProject')}</button>
    </div>

    {#if creatingProject}
     <form class="project-form" data-project-create-form onsubmit={submitNewProject}>
      <input type="text" maxlength="80" placeholder={t('history.projectNamePlaceholder')} bind:value={newProjectName} onkeydown={e=>{if(e.key==='Escape'){creatingProject=false;newProjectName='';}}}/>
      <button type="submit" class="mini-btn primary-mini" disabled={!newProjectName.trim()}>{t('history.create')}</button>
      <button type="button" class="mini-btn" onclick={()=>{creatingProject=false;newProjectName='';}}>{t('history.cancel')}</button>
     </form>
    {/if}

    {#if activeFilterProject}
     <div class="project-banner" data-active-project-bar>
      {#if renamingProject}
       <form class="project-form" onsubmit={submitRenameProject}>
        <input type="text" maxlength="80" bind:value={projectRenameDraft} onkeydown={e=>{if(e.key==='Escape'){renamingProject=false;}}}/>
        <button type="submit" class="mini-btn primary-mini" disabled={!projectRenameDraft.trim()}>{t('history.save')}</button>
        <button type="button" class="mini-btn" onclick={()=>renamingProject=false}>{t('history.cancel')}</button>
       </form>
      {:else}
       <div class="project-banner-actions">
        {#if onNewInProject}
         <button type="button" class="mini-btn primary-mini" data-project-new-chat onclick={()=>onNewInProject(activeFilterProject.id)}>+ {t('history.newChatInProject')}</button>
        {/if}
        <button type="button" class="ghost-btn compact" onclick={()=>{projectRenameDraft=activeFilterProject.name;renamingProject=true;}}>{t('history.rename')}</button>
        <button type="button" class="ghost-btn compact danger-text" data-project-delete onclick={handleDeleteProject}>{t('history.deleteProject')}</button>
       </div>
      {/if}
     </div>
    {/if}

    <input class="list-search" type="search" aria-label={t('history.searchPlaceholder')} placeholder={t('history.searchPlaceholder')} bind:value={query}/>

    {#if editMode}
     <div class="edit-toolbar" data-history-edit-bar>
      <div class="edit-row">
       <button type="button" class="mini-btn" data-history-check-page disabled={!canDelete||info.loading||info.deleting||!selectableVisible.length} onclick={toggleVisiblePage}>{pageChecked?t('history.unselectPage'):t('history.selectPage')}</button>
       <span class="selected-count">{t('history.selectedCount',{count:checkedCount})}</span>
       <button type="button" class="ghost-btn compact danger-text ml-auto" data-history-clear title={t('history.clearTitle')} disabled={!canDelete||!port.clear||info.loading||info.deleting||(!info.items.length&&info.page===0&&!info.hasMore)} onclick={()=>deletion={kind:'all'}}>{t('history.clear')}</button>
      </div>
      <div class="edit-row actions-row">
       <button type="button" class="mini-btn" data-history-bulk-pin disabled={!checkedCount||info.loading||info.deleting} onclick={bulkPinToggle}>{allCheckedPinned?t('history.unpin'):t('history.pin')}</button>
       <div class="bulk-project-wrap" bind:this={bulkProjectMenuWrap}>
        <button type="button" class="mini-btn" data-history-bulk-project aria-haspopup="menu" aria-expanded={bulkProjectMenuOpen} disabled={!checkedCount||info.loading||info.deleting} onclick={()=>{bulkProjectMenuOpen=!bulkProjectMenuOpen;}}>{t('history.moveToProject').replace(/[…\.]+$/,'')} ▾</button>
        {#if bulkProjectMenuOpen}
         <div class="bulk-project-menu" role="menu" aria-label={t('history.moveToProject')}>
          {#each projects as proj (proj.id)}
           <button type="button" role="menuitem" data-bulk-project-item={proj.id} onclick={()=>{bulkProjectMenuOpen=false;bulkAssignProject(proj.id);}}>{proj.name}</button>
          {/each}
          <button type="button" role="menuitem" class="new-proj-item" data-bulk-project-new onclick={()=>{bulkProjectMenuOpen=false;bulkAssignProject('__new__');}}>+ {t('history.newProject')}</button>
         </div>
        {/if}
       </div>
       <button type="button" class="mini-btn" data-history-bulk-detach disabled={!anyCheckedInProject||info.loading||info.deleting} onclick={bulkDetachProject}>{t('history.detachProject')}</button>
       <button type="button" class="mini-btn" data-history-bulk-archive disabled={!checkedCount||info.loading||info.deleting} onclick={bulkArchive}>{t('history.archive')}</button>
       <button type="button" class="mini-btn danger-pill" data-history-bulk disabled={!checkedCount||!canDelete||!port.removeMany||info.loading||info.deleting} onclick={()=>deletion={kind:'selected',ids:[...checkedIds]}}>{checkedCount?t('history.deleteCount',{count:checkedCount}):t('history.delete')}</button>
      </div>
     </div>
    {/if}
   </div>

   <div class="records" aria-label={t('history.title')}>
    {#each visibleItems as item (item.requestId)}
     {@const titleText=getDisplayTitle(item.requestId,item.title)}
     {@const pinned=isPinned(item.requestId)}
     {@const itemProject=getProjectById(getProjectIdFor(item.requestId,item.conversationId))}
     <div class="record-row" class:selected={!editMode&&info.selected?.requestId===item.requestId} class:checked={editMode&&!!info.checked[item.requestId]} class:pinned>
      {#if editMode}
       <input type="checkbox" class="row-check" data-history-check={item.requestId} aria-label={titleText} checked={!!info.checked[item.requestId]} disabled={!canDelete||info.loading||info.deleting||item.state==='streaming'} onchange={()=>history.toggleChecked(item.requestId)}/>
      {/if}
      <button class="row-card" class:selected={!editMode&&info.selected?.requestId===item.requestId} disabled={info.loading||info.deleting} onclick={()=>handleRowClick(item)}>
       <div class="row-title-line">
        <strong title={titleText}>{titleText}</strong>
        {#if pinned}<span class="pin-badge" data-pin-badge>{t('history.pinned')}</span>{/if}
       </div>
       <div class="row-meta">
        <span class="state-dot" data-state={item.state} aria-hidden="true"></span>
        <span>{labels[item.state]}</span>
        {#if itemProject&&projectFilter==='all'}
         <span class="dot-sep">·</span>
         <span class="proj-badge">{itemProject.name}</span>
        {/if}
        <span class="dot-sep">·</span>
        <span class="model-tag">{item.model}</span>
        {#if item.createdAt}<span class="row-time">{shortDate(item.createdAt)}</span>{/if}
       </div>
      </button>
     </div>
    {/each}
    {#if !visibleItems.length&&!info.loading}
     <div class="list-empty">{query.trim()?t('history.emptySearch'):activeFilterProject?t('history.emptyProject'):t('history.empty')}</div>
    {/if}
   </div>

   <div class="pagination">
    <button class="mini-btn" disabled={info.page===0||info.loading||info.deleting} onclick={()=>load(info.page-1)}>{t('history.prevPage')}</button>
    <span class="page-num">{t('history.pageNum',{page:info.page+1})}</span>
    <button class="mini-btn" disabled={!info.hasMore||info.loading||info.deleting} onclick={()=>load(info.page+1)}>{t('history.nextPage')}</button>
   </div>
  </aside>

  <div class="record-detail">
   {#if info.selected}
    <header class="detail-header">
     <div class="detail-top-row">
      {#if renamingDetail}
       <form class="rename-form" data-rename-form onsubmit={saveRenameDetail}>
        <input type="text" maxlength="120" placeholder={t('history.renamePlaceholder')} bind:value={detailTitleDraft} onkeydown={e=>{if(e.key==='Escape')renamingDetail=false;}}/>
        <button type="submit" class="mini-btn primary-mini">{t('history.save')}</button>
        <button type="button" class="mini-btn" onclick={()=>renamingDetail=false}>{t('history.cancel')}</button>
       </form>
      {:else}
       <div class="detail-title-wrap">
        <h2 class="detail-title" title={selectedTitle} ondblclick={startRenameDetail}>{selectedTitle}</h2>
        <button type="button" class="ghost-btn compact" data-history-rename onclick={startRenameDetail}>{t('history.rename')}</button>
       </div>
      {/if}
     </div>

     <div class="detail-bottom-row">
      <div class="detail-meta">
       <span class="status-pill" data-state={info.selected.state}>{labels[info.selected.state]}</span>
       <span class="detail-time">{new Date(info.selected.createdAt*1000).toLocaleString(getLocale())}</span>
      </div>
      <div class="detail-actions record-meta">
       <button class="primary-continue" disabled={!canContinue||info.deleting||info.selected.state==='streaming'} onclick={()=>onContinue(info.selected!)}>{t('history.continue')}</button>
       <div class="detail-project-wrap" bind:this={detailProjectMenuWrap}>
        <button type="button" class="mini-btn" data-detail-project aria-haspopup="menu" aria-expanded={detailProjectMenuOpen} onclick={()=>{detailProjectMenuOpen=!detailProjectMenuOpen;}}>{(selectedProjectId?getProjectById(selectedProjectId)?.name:null)??t('history.moveToProject').replace(/[…\.]+$/,'')} ▾</button>
        {#if detailProjectMenuOpen}
         <div class="bulk-project-menu detail-project-menu" role="menu" aria-label={t('chat.projectLabel')}>
          {#each projects as proj (proj.id)}
           <button type="button" role="menuitem" class:active-item={selectedProjectId===proj.id} data-detail-project-item={proj.id} onclick={()=>{detailProjectMenuOpen=false;changeDetailProject(proj.id);}}>{proj.name}</button>
          {/each}
          {#if selectedProjectId}
           <button type="button" role="menuitem" data-detail-project-detach onclick={()=>{detailProjectMenuOpen=false;changeDetailProject('');}}>{t('history.detachProject')}</button>
          {/if}
          <button type="button" role="menuitem" class="new-proj-item" data-detail-project-new onclick={()=>{detailProjectMenuOpen=false;changeDetailProject('__new__');}}>+ {t('history.newProject')}</button>
         </div>
        {/if}
       </div>
       <button type="button" class="ghost-btn compact" data-detail-pin onclick={()=>togglePin(info.selected!.requestId)}>{selectedPinned?t('history.unpin'):t('history.pin')}</button>
       <button type="button" class="ghost-btn compact" data-detail-archive disabled={info.selected.state==='streaming'} onclick={archiveSelectedDetail}>{t('history.archive')}</button>
       <button class="ghost-btn compact danger-text" data-history-delete disabled={!canDelete||!port.remove||info.loading||info.deleting||!info.selected.saved||info.selected.state==='streaming'} onclick={()=>deletion={kind:'single',id:info.selected!.requestId}}>{t('history.delete')}</button>
      </div>
     </div>
    </header>

    <div class="detail-scroll">
     <article class="detail-thread">
      <div class="thread-question">
       <p class="original-question">{info.selected.prompt}</p>
      </div>
      <div class="thread-answer">
       <div class="answer-head">
        <BrandLogo compact/>
        <strong>{info.selected.model||t('reply.label')}</strong>
        {#if info.selected.answer}
         <button type="button" class="ghost-btn compact ml-auto" data-detail-copy onclick={copyDetailAnswer}>{detailCopied?t('reply.copied'):t('reply.copy')}</button>
        {/if}
       </div>
       <div class="prose" role="presentation" onclick={e=>handleProseClick(e,openExternal)} onkeydown={e=>handleProseClick(e as unknown as MouseEvent,openExternal)}>{@html answer}</div>
       {#if info.selected.error}<p class="banner-error">{info.selected.error}</p>{/if}
      </div>
     </article>
    </div>
   {:else}
    <div class="empty-detail">
     <div class="empty-icon"><Icon name="history" size={24}/></div>
     <p>{info.loading?t('history.loadingDetail'):t('history.selectPrompt')}</p>
    </div>
   {/if}
  </div>
 </div>
</section>

<style>
 .history{height:100%;min-height:0;display:flex;flex-direction:column;color:var(--text);font-size:1rem}
 .banner-notice{flex-shrink:0;margin:0 0 10px;padding:8px 12px;border-radius:var(--radius-control);background:var(--surface-soft);border:1px solid var(--edge);font-size:.9rem;color:var(--muted)}
 .banner-error{flex-shrink:0;margin:0 0 10px;padding:8px 12px;border-radius:var(--radius-control);background:var(--danger-soft);border:1px solid color-mix(in srgb,var(--danger) 28%,transparent);font-size:.9rem;color:var(--danger)}
 .delete-confirm{flex-shrink:0;display:flex;align-items:center;justify-content:space-between;gap:14px;flex-wrap:wrap;padding:12px 14px;margin-bottom:12px;border:1px solid color-mix(in srgb,var(--danger) 35%,var(--edge));border-radius:var(--radius-card);background:var(--danger-soft)}
 .delete-copy h2{margin:0 0 4px;font-size:.98rem;font-weight:600;color:var(--text)}
 .delete-copy p{margin:0;font-size:.86rem;color:var(--muted)}
 .delete-actions{display:flex;gap:8px;align-items:center}
 .danger-solid{background:var(--danger);color:#ffffff;border-color:var(--danger);font-weight:500}
 @media(hover:hover) and (pointer:fine){.danger-solid:hover:not(:disabled){background:var(--danger);opacity:.9;color:#ffffff}}

 .history-split{flex:1;min-height:0;display:grid;grid-template-columns:minmax(290px,348px) minmax(0,1fr);border:1px solid var(--edge);border-radius:var(--radius-card);background:var(--surface);overflow:hidden}
 .history-sidebar{display:flex;flex-direction:column;min-height:0;min-width:0;background:var(--sidebar-bg);border-right:1px solid var(--edge)}
 .sidebar-top{flex-shrink:0;padding:12px 12px 10px;border-bottom:1px solid var(--edge);display:flex;flex-direction:column;gap:8px}
 .sidebar-head{display:flex;align-items:center;justify-content:space-between;gap:8px}
 .sidebar-head h1{margin:0;font-size:1.04rem;font-weight:600;letter-spacing:-0.01em}
 .head-ops{display:flex;align-items:center;gap:4px}
 .edit-active{background:var(--accent-soft);color:var(--text);font-weight:600}

 .project-tabs{display:flex;align-items:center;gap:5px;flex-wrap:wrap}
 .proj-chip{padding:3px 9px;font-size:.82rem;font-weight:500;border-radius:999px;background:var(--surface);border:1px solid var(--edge);color:var(--muted);max-width:140px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
 .proj-chip.active{background:var(--accent);color:var(--on-accent);border-color:var(--accent);font-weight:600}
 .proj-chip.add-chip{background:transparent;border-style:dashed;color:var(--muted)}
 @media(hover:hover) and (pointer:fine){.proj-chip:hover:not(.active){background:var(--surface-hover);color:var(--text);border-color:var(--edge-strong)}}
 .project-form{display:flex;align-items:center;gap:5px}
 .project-form input{min-width:0;flex:1;padding:5px 8px;font-size:.86rem;border:1px solid var(--edge-strong);border-radius:7px;background:var(--surface);color:var(--text)}
 .project-banner{padding:6px 8px;border-radius:var(--radius-control);background:var(--surface);border:1px solid var(--edge)}
 .project-banner-actions{display:flex;align-items:center;gap:4px;flex-wrap:wrap}

 .list-search{width:100%;padding:6px 10px;border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface);color:var(--text);font:inherit;font-size:.88rem}
 .list-search:focus-visible{outline:2px solid var(--focus-ring);outline-offset:1px}

 .edit-toolbar{display:flex;flex-direction:column;gap:6px;padding:8px;border-radius:var(--radius-control);background:var(--surface);border:1px solid var(--edge)}
 .edit-row{display:flex;align-items:center;gap:5px;flex-wrap:wrap}
 .selected-count{font-size:.82rem;color:var(--muted);font-variant-numeric:tabular-nums}
 .ml-auto{margin-left:auto}
 .bulk-project-wrap{position:relative;display:inline-flex}
 .bulk-project-menu{position:absolute;left:0;top:calc(100% + 5px);min-width:140px;max-width:210px;padding:4px;border-radius:9px;background:var(--surface);border:1px solid var(--edge-strong);box-shadow:var(--hover-shadow);display:flex;flex-direction:column;gap:2px;z-index:45;animation:menuPop 140ms var(--ease-out) both}
 .bulk-project-menu button{border:0;background:transparent;color:var(--text);font-size:.86rem;font-weight:500;padding:6px 10px;border-radius:6px;text-align:left;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
 .bulk-project-menu button.active-item{background:var(--accent-soft);font-weight:600}
 .bulk-project-menu button.new-proj-item{color:var(--muted);border-top:1px solid var(--edge);border-radius:0 0 6px 6px;margin-top:2px;padding-top:7px}
 @media(hover:hover) and (pointer:fine){.bulk-project-menu button:hover{background:var(--surface-hover);color:var(--text)}}
 .detail-project-wrap{position:relative;display:inline-flex}
 .detail-project-menu{left:auto;right:0}

 .records{flex:1;min-height:0;overflow-y:auto;padding:8px;display:flex;flex-direction:column;gap:4px;scrollbar-width:thin}
 .record-row{position:relative;display:flex;align-items:flex-start;gap:6px;padding:2px 6px;border-radius:var(--radius-control);border:1px solid transparent;transition:background-color 140ms ease,border-color 140ms ease}
 @media(hover:hover) and (pointer:fine){.record-row:hover{background:var(--surface-hover)}}
 .record-row.selected{background:var(--surface);border-color:var(--edge-strong);box-shadow:var(--raised-shadow)}
 .record-row.checked{background:var(--accent-soft);border-color:color-mix(in srgb,var(--accent) 30%,transparent)}
 .row-check{margin:11px 0 0;width:15px;height:15px;flex-shrink:0;accent-color:var(--accent);cursor:pointer}
 .row-card{flex:1;min-width:0;border:0;background:transparent;box-shadow:none;padding:8px 4px;text-align:left;display:flex;flex-direction:column;gap:5px;border-radius:6px}
 @media(hover:hover) and (pointer:fine){.row-card:hover:not(:disabled){background:transparent;border-color:transparent}}
 .row-title-line{display:flex;align-items:flex-start;justify-content:space-between;gap:6px;min-width:0}
 .row-card strong{font-size:.93rem;font-weight:500;line-height:1.45;color:var(--text);display:-webkit-box;-webkit-line-clamp:2;line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;overflow-wrap:anywhere;flex:1;min-width:0}
 .record-row.selected .row-card strong{font-weight:600}
 .pin-badge{flex-shrink:0;font-size:.74rem;font-weight:600;padding:1px 6px;border-radius:999px;background:var(--accent-soft);color:var(--text);margin-top:1px}
 .row-meta{display:flex;align-items:center;gap:5px;min-width:0;font-size:.8rem;color:var(--muted);font-variant-numeric:tabular-nums}
 .state-dot{width:6px;height:6px;border-radius:50%;background:var(--muted);flex-shrink:0}
 .state-dot[data-state="complete"]{background:#22c55e}
 .state-dot[data-state="failed"],.state-dot[data-state="interrupted"]{background:var(--danger)}
 .state-dot[data-state="streaming"]{background:var(--accent)}
 .dot-sep{opacity:.5}
 .proj-badge{max-width:82px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;padding:0 5px;border-radius:4px;background:var(--surface-soft);border:1px solid var(--edge);color:var(--text);font-size:.76rem;flex-shrink:0}
 .model-tag{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;min-width:0;flex:1}
 .row-time{flex-shrink:0;opacity:.8}
 .list-empty{padding:36px 12px;text-align:center;font-size:.9rem;color:var(--muted)}

 .pagination{flex-shrink:0;display:flex;align-items:center;justify-content:space-between;gap:8px;padding:8px 10px;border-top:1px solid var(--edge);background:var(--sidebar-bg);font-size:.84rem;color:var(--muted);font-variant-numeric:tabular-nums}
 .page-num{font-weight:500}

 .record-detail{display:flex;flex-direction:column;min-height:0;min-width:0;background:var(--bg)}
 .detail-header{flex-shrink:0;display:flex;flex-direction:column;gap:8px;padding:12px 18px;border-bottom:1px solid var(--edge);background:var(--surface)}
 .detail-top-row{display:flex;align-items:center;gap:8px;min-width:0}
 .detail-title-wrap{display:flex;align-items:center;gap:8px;min-width:0;width:100%}
 .detail-title{margin:0;font-size:1.05rem;font-weight:600;color:var(--text);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;min-width:0;flex:1;cursor:text}
 .rename-form{display:flex;align-items:center;gap:6px;width:100%}
 .rename-form input{flex:1;min-width:0;padding:5px 9px;font-size:.95rem;border:1px solid var(--focus-ring);border-radius:7px;background:var(--surface-soft);color:var(--text)}
 .detail-bottom-row{display:flex;align-items:center;justify-content:space-between;gap:10px;flex-wrap:wrap}
 .detail-meta{display:flex;align-items:center;gap:8px;min-width:0;flex-wrap:wrap;font-size:.84rem;color:var(--muted);font-variant-numeric:tabular-nums}
 .status-pill{display:inline-flex;align-items:center;padding:2px 8px;border-radius:999px;font-size:.8rem;font-weight:600;background:var(--surface-soft);color:var(--text);border:1px solid var(--edge)}
 .status-pill[data-state="failed"],.status-pill[data-state="interrupted"]{background:var(--danger-soft);color:var(--danger);border-color:transparent}
 .detail-actions{display:flex;align-items:center;gap:6px;flex-wrap:wrap}
 .primary-continue{background:var(--accent);color:var(--on-accent);border-color:var(--accent);font-size:.86rem;font-weight:500;padding:5px 11px}
 @media(hover:hover) and (pointer:fine){.primary-continue:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}

 .detail-scroll{flex:1;min-height:0;overflow-y:auto;padding:20px 24px 28px;scrollbar-width:thin}
 .detail-thread{max-width:var(--reading-width);margin:0 auto;display:flex;flex-direction:column;gap:22px}
 .thread-question{display:flex;justify-content:flex-end}
 .original-question{max-width:84%;margin:0;padding:11px 15px;border-radius:var(--radius-card);background:var(--surface-soft);border:1px solid var(--edge);white-space:pre-wrap;overflow-wrap:anywhere;text-wrap:pretty;font-size:var(--prose-size);line-height:1.65;color:var(--text)}
 .thread-answer{display:flex;flex-direction:column;gap:10px;min-width:0}
 .answer-head{display:flex;align-items:center;gap:8px;font-size:13px;color:var(--text)}
 .answer-head strong{font-weight:600}

 .empty-detail{flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:10px;padding:32px;color:var(--muted);text-align:center}
 .empty-icon{width:44px;height:44px;border-radius:12px;background:var(--surface-soft);border:1px solid var(--edge);display:grid;place-items:center;color:var(--muted)}
 .empty-detail p{margin:0;font-size:13.5px}

 .ghost-btn{border:0;background:transparent;color:var(--muted);padding:5px 8px;font-size:12.5px;font-weight:500;border-radius:6px}
 .ghost-btn.compact{padding:3px 7px;font-size:12px}
 @media(hover:hover) and (pointer:fine){.ghost-btn:hover:not(:disabled){background:var(--surface-hover);color:var(--text)}}
 .danger-text{color:var(--muted)}
 @media(hover:hover) and (pointer:fine){.danger-text:hover:not(:disabled){color:var(--danger);background:var(--danger-soft)}}
 .mini-btn{padding:4px 9px;font-size:12px;font-weight:500;border-radius:6px;background:var(--surface);border:1px solid var(--edge);color:var(--text)}
 .primary-mini{background:var(--accent);color:var(--on-accent);border-color:var(--accent)}
 @media(hover:hover) and (pointer:fine){.primary-mini:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}
 .danger-pill{color:var(--danger);background:var(--danger-soft);border-color:color-mix(in srgb,var(--danger) 30%,transparent)}

 @media(max-width:760px){
  .history-split{grid-template-columns:1fr;grid-template-rows:minmax(240px,45%) minmax(0,1fr)}
  .history-sidebar{border-right:0;border-bottom:1px solid var(--edge)}
  .detail-scroll{padding:16px}
 }
</style>
