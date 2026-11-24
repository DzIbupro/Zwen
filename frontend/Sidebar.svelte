<script lang="ts">
 import BrandLogo from './BrandLogo.svelte';import Icon from './Icon.svelte';
 import {t} from './lib/i18n.svelte';
 import type {Summary} from './lib/history';
 import {
  createProject,
  deleteProject,
  detachFromProject,
  getActiveProjectId,
  getDisplayTitle,
  getProjectConversations,
  getProjects,
  renameProject,
  renameRecord,
 } from './lib/history-org.svelte';

 export type Destination='chat'|'history'|'knowledge'|'graph'|'settings';

 let {
  collapsed,
  active,
  disabled=false,
  canNew=true,
  activeRequestId=null,
  recentSummaries=[],
  onToggle,
  onNavigate,
  onNew,
  onNewInProject,
  onOpenConversation,
  onDeleteConversation,
 }:{
  collapsed:boolean;
  active:Destination;
  disabled?:boolean;
  canNew?:boolean;
  activeRequestId?:string|null;
  recentSummaries?:Summary[];
  onToggle:()=>void;
  onNavigate:(d:Destination)=>void;
  onNew:()=>void;
  onNewInProject?:(projectId:string)=>void;
  onOpenConversation?:(requestId:string)=>void;
  onDeleteConversation?:(requestId:string)=>void;
 }=$props();

 const items=$derived([
  {id:'chat' as const,label:t('nav.chat'),icon:'message' as const},
  {id:'history' as const,label:t('nav.history'),icon:'history' as const},
  {id:'knowledge' as const,label:t('nav.knowledge'),icon:'book' as const},
  {id:'graph' as const,label:t('nav.graph'),icon:'network' as const},
 ]);

 const projects=$derived(getProjects());
 const activeProjectId=$derived(getActiveProjectId());
 let addingProject=$state(false);
 let draftProjectName=$state('');
 let collapsedProjects=$state<Record<string,boolean>>({});

 type MenuTarget={kind:'project';id:string}|{kind:'chat';id:string;conversationId:string;title:string}|null;
 let openMenu=$state<MenuTarget>(null);
 let renamingTarget=$state<{kind:'project'|'chat';id:string}|null>(null);
 let renameDraft=$state('');
 let sidebarEl=$state<HTMLElement>();

 function closeMenuOutside(e:PointerEvent){
  if(!openMenu)return;
  if(e.target instanceof Element&&e.target.closest('.sidebar-ctx-menu,.sidebar-more-btn'))return;
  openMenu=null;
 }

 function submitProject(e:Event){
  e.preventDefault();
  const created=createProject(draftProjectName);
  if(created){
   draftProjectName='';
   addingProject=false;
   onNewInProject?.(created.id);
  }
 }

 function submitRename(e:Event){
  e.preventDefault();
  if(!renamingTarget)return;
  if(renamingTarget.kind==='project'){
   renameProject(renamingTarget.id,renameDraft);
  }else{
   renameRecord(renamingTarget.id,renameDraft);
  }
  renamingTarget=null;
  renameDraft='';
 }

 function toggleProjectGroup(projectId:string){
  collapsedProjects={...collapsedProjects,[projectId]:!collapsedProjects[projectId]};
 }

 function formatRelativeAge(tsSec:number):string{
  if(!tsSec)return '';
  const diffSec=Math.max(0,Math.floor(Date.now()/1000)-tsSec);
  if(diffSec<60)return '1m';
  const mins=Math.floor(diffSec/60);
  if(mins<60)return `${mins}m`;
  const hours=Math.floor(mins/60);
  if(hours<24)return `${hours}h`;
  const days=Math.floor(hours/24);
  if(days<365)return `${days}d`;
  return `${Math.floor(days/365)}y`;
 }
</script>

<svelte:window onpointerdown={closeMenuOutside}/>
<aside bind:this={sidebarEl} class="sidebar" class:collapsed data-sidebar data-collapsed={collapsed}>
 <div class="sidebar-brand"><BrandLogo/>{#if !collapsed}<strong>Zwen</strong>{/if}</div>
 <button class="sidebar-toggle" aria-label={collapsed?t('nav.expandSidebar'):t('nav.collapseSidebar')} aria-expanded={!collapsed} title={collapsed?t('nav.expandSidebar'):t('nav.collapseSidebar')} onclick={onToggle} {disabled}><Icon name={collapsed?'expand':'collapse'}/>{#if !collapsed}<span>{t('nav.collapseSidebar')}</span>{/if}</button>
 <nav aria-label={t('nav.main')}>
  <button class="new-chat" data-new-conversation aria-label={t('nav.newChat')} title={collapsed?t('nav.newChat'):undefined} disabled={!canNew} onclick={onNew}><Icon name="plus"/><span class:sr-only={collapsed}>{t('nav.newChat')}</span></button>
  {#each items as item}<button class:active={active===item.id} aria-current={active===item.id?'page':undefined} aria-label={item.label} title={collapsed?item.label:undefined} data-nav={item.id} onclick={()=>onNavigate(item.id)}><Icon name={item.icon}/><span class:sr-only={collapsed}>{item.label}</span></button>{/each}

  {#if !collapsed && (onNewInProject || onOpenConversation)}
   <div class="sidebar-projects" data-sidebar-projects>
    <div class="sidebar-projects-head">
     <span>{t('sidebar.projects')}</span>
     <button type="button" class="sidebar-plus-btn" data-sidebar-new-project title={t('sidebar.newProject')} aria-label={t('sidebar.newProject')} onclick={()=>{addingProject=!addingProject;draftProjectName='';}}><Icon name="plus" size={16}/></button>
    </div>

    {#if addingProject}
     <form class="sidebar-project-create" data-sidebar-project-form onsubmit={submitProject}>
      <input type="text" maxlength="80" placeholder={t('history.projectNamePlaceholder')} bind:value={draftProjectName} onkeydown={e=>{if(e.key==='Escape'){addingProject=false;draftProjectName='';}}}/>
      <button type="submit" disabled={!draftProjectName.trim()}>{t('history.create')}</button>
     </form>
    {/if}

    {#each projects as project (project.id)}
     {@const chats=getProjectConversations(project.id,recentSummaries)}
     {@const isCollapsed=!!collapsedProjects[project.id]}
     <div class="sidebar-project-group" data-sidebar-project-group={project.id}>
      {#if renamingTarget?.kind==='project'&&renamingTarget.id===project.id}
       <form class="sidebar-project-create" onsubmit={submitRename}>
        <input type="text" maxlength="80" bind:value={renameDraft} onkeydown={e=>{if(e.key==='Escape'){renamingTarget=null;}}}/>
        <button type="submit" disabled={!renameDraft.trim()}>{t('history.save')}</button>
       </form>
      {:else}
       <div class="sidebar-project-row" class:current-project={active==='chat'&&activeProjectId===project.id&&!activeRequestId}>
        <button type="button" class="project-folder-btn" data-sidebar-project={project.id} aria-expanded={!isCollapsed} title={project.name} onclick={()=>toggleProjectGroup(project.id)}>
         <svg class="folder-icon" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true"><path d="M1.75 4.25A1.25 1.25 0 0 1 3 3h3.1l1.25 1.4H13A1.25 1.25 0 0 1 14.25 5.65v6.1A1.25 1.25 0 0 1 13 13H3a1.25 1.25 0 0 1-1.25-1.25v-7.5Z" fill="none" stroke="currentColor" stroke-width="1.35" stroke-linejoin="round"/></svg>
         <span class="project-nav-label">{project.name}</span>
        </button>
        {#if onNewInProject}
         <button type="button" class="sidebar-plus-btn project-add-chat" data-sidebar-project-new={project.id} title={t('history.newChatInProject')} aria-label={`${t('history.newChatInProject')}: ${project.name}`} disabled={!canNew} onclick={()=>onNewInProject(project.id)}><Icon name="plus" size={15}/></button>
        {/if}
        <button type="button" class="sidebar-more-btn" data-sidebar-project-menu={project.id} aria-label={project.name} onclick={()=>{openMenu=openMenu?.kind==='project'&&openMenu.id===project.id?null:{kind:'project',id:project.id};}}><Icon name="more" size={14}/></button>
       </div>
       {#if openMenu?.kind==='project'&&openMenu.id===project.id}
        <div class="sidebar-ctx-menu" role="menu">
         <button type="button" role="menuitem" onclick={()=>{openMenu=null;renamingTarget={kind:'project',id:project.id};renameDraft=project.name;}}>{t('history.rename')}</button>
         <button type="button" role="menuitem" class="danger-item" onclick={()=>{openMenu=null;deleteProject(project.id);}}>{t('history.deleteProject')}</button>
        </div>
       {/if}
      {/if}

      {#if !isCollapsed && chats.length>0}
       <div class="sidebar-project-chats">
        {#each chats as item (item.requestId)}
         {@const label=getDisplayTitle(item.requestId,item.title)}
         {#if renamingTarget?.kind==='chat'&&renamingTarget.id===item.requestId}
          <form class="sidebar-project-create" onsubmit={submitRename}>
           <input type="text" maxlength="120" bind:value={renameDraft} onkeydown={e=>{if(e.key==='Escape'){renamingTarget=null;}}}/>
           <button type="submit" disabled={!renameDraft.trim()}>{t('history.save')}</button>
          </form>
         {:else}
          <div class="sidebar-chat-row">
           <button type="button" class="sidebar-chat-item" class:active={active==='chat'&&activeRequestId===item.requestId} data-sidebar-chat={item.requestId} title={label} onclick={()=>onOpenConversation?.(item.requestId)}>
            <span class="chat-item-title">{label}</span>
            {#if item.createdAt}<span class="chat-item-age">{formatRelativeAge(item.createdAt)}</span>{/if}
           </button>
           <button type="button" class="sidebar-more-btn chat-more-btn" data-sidebar-chat-menu={item.requestId} aria-label={label} onclick={()=>{openMenu=openMenu?.kind==='chat'&&openMenu.id===item.requestId?null:{kind:'chat',id:item.requestId,conversationId:item.conversationId,title:label};}}><Icon name="more" size={13}/></button>
          </div>
          {#if openMenu?.kind==='chat'&&openMenu.id===item.requestId}
           <div class="sidebar-ctx-menu" role="menu">
            <button type="button" role="menuitem" onclick={()=>{openMenu=null;renamingTarget={kind:'chat',id:item.requestId};renameDraft=label;}}>{t('history.rename')}</button>
            <button type="button" role="menuitem" onclick={()=>{openMenu=null;detachFromProject([{requestId:item.requestId,conversationId:item.conversationId}]);}}>{t('history.detachProject')}</button>
            {#if onDeleteConversation}
             <button type="button" role="menuitem" class="danger-item" disabled={!canNew} onclick={()=>{openMenu=null;onDeleteConversation(item.requestId);}}>{t('history.delete')}</button>
            {/if}
           </div>
          {/if}
         {/if}
        {/each}
       </div>
      {/if}
     </div>
    {/each}
   </div>
  {/if}

  <button class="settings-link" class:active={active==='settings'} aria-current={active==='settings'?'page':undefined} aria-label={t('nav.settings')} title={collapsed?t('nav.settings'):undefined} data-nav="settings" onclick={()=>onNavigate('settings')}><Icon name="settings"/><span class:sr-only={collapsed}>{t('nav.settings')}</span></button>
 </nav>
</aside>
