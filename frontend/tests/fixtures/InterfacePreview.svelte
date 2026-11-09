<script lang="ts">
 import {onMount,tick} from 'svelte';
 import WelcomeMark from '../../WelcomeMark.svelte';
 import KnowledgePanel from '../../KnowledgePanel.svelte';
 import KnowledgeGraph from '../../KnowledgeGraph.svelte';
 import SettingsHub from '../../SettingsHub.svelte';
 import ProviderSettings from '../../ProviderSettings.svelte';
 import AppearanceSettings from '../../AppearanceSettings.svelte';
 import SelectionSettings from '../../SelectionSettings.svelte';
 import QuickActionSettings from '../../QuickActionSettings.svelte';
 import PaperComposer from '../../PaperComposer.svelte';
 import BrandLogo from '../../BrandLogo.svelte';
 import Icon from '../../Icon.svelte';
 import {KnowledgeController,type KnowledgePort,type KnowledgeState,type Note,type TopicView,type RelationView} from '../../lib/knowledge';
 import {PaperDraftController,type PaperDraftState} from '../../lib/paper-draft';
 import {defaultReadingOptions} from '../../lib/papers';
 import {defaultQuickActions,type QuickActionConfig} from '../../lib/quick-actions';
 import type {SettingsState} from '../../lib/provider';
 import type {AppearanceState} from '../../lib/appearance';
 import type {SettingsCategory} from '../../SettingsHub.svelte';
 const params=new URLSearchParams(location.search);
 type Screen='home'|'knowledge'|'graph'|'settings'|'paper';
 const screens:Screen[]=['home','knowledge','graph','settings','paper'];
 let screen=$state<Screen>(screens.includes(params.get('screen') as Screen)?params.get('screen') as Screen:'home');
 const labels={home:'新对话',knowledge:'知识库',graph:'知识网络',settings:'设置',paper:'论文阅读'};
 const columns=['声音与感知','模型与方法','实验与评估','工具与写作'];
 const names=['说话人表征','语音合成','音频伪造检测','声学特征','跨域泛化','自监督学习','注意力机制','数据增强','实验复现','评价指标','对比学习','多模态理解','数据集设计','错误分析','可解释性','文献阅读','时间管理','研究笔记','科研写作','图表设计'];
 const notes:Note[]=Array.from({length:60},(_,i)=>({meta:{id:`preview-note-${i}`,title:`${names[i%names.length]}：${['核心概念与阅读记录','实验设计及适用条件','结果核对与后续问题'][Math.floor(i/20)]}`,createdAt:1794200400-i*86400,source:'阅读收藏',model:'本地演示',attachments:[],request:null},body:`## ${names[i%names.length]}\n\n这是一篇用于界面验证的合成笔记。记录核心问题、方法与证据，保留原始文献的核对入口。\n\n### 阅读重点\n\n- 研究问题与方法假设\n- 实验设置与适用条件\n- 还需要进一步验证的问题\n\n`+'结合任务背景比较不同方法的设计取舍，并记录证据边界。\n\n'.repeat(20)}));
 const topics:TopicView[]=Array.from({length:40},(_,i)=>({topic:{id:`preview-topic-${i}`,name:`${names[i%20]}${i>=20?' · 延伸阅读':''}`,column:columns[Math.floor(i/10)],createdBy:i%4?'ai':'manual',batchId:null,createdAt:1794200400,locked:i%11===0},noteIds:[notes[i%60].meta.id,notes[(i+19)%60].meta.id,notes[(i+37)%60].meta.id],count:3}));
 const relations:RelationView[]=Array.from({length:52},(_,i)=>({id:`preview-relation-${i}`,fromTopic:topics[i%40].topic.id,toTopic:topics[(i+1+(i%4)*7)%40].topic.id,reason:'共同讨论表示学习与实验评估，阅读证据可互相参照。',evidence:notes[i%60].meta.id,addedBy:'manual'}));
 const noop=async()=>{};
 const port:KnowledgePort={
  list:async query=>notes.filter(n=>n.meta.title.includes(query)||n.body.includes(query)).map(n=>n.meta),
  read:async id=>notes.find(n=>n.meta.id===id)??notes[0],
  save:async input=>({...notes[0].meta,title:input.title}),remove:noop,
  update:async(id,title,body,source)=>{const note=notes.find(n=>n.meta.id===id)??notes[0];note.meta={...note.meta,title,...source};note.body=body;return note.meta;},updateSource:noop,
  exportNote:async()=>'/synthetic-preview/note.md',exportAll:async()=>'/synthetic-preview/notes',rebuild:async()=>notes.length,
  topics:async()=>topics,topicNotes:async id=>(topics.find(t=>t.topic.id===id)?.noteIds??[]).map(id=>notes.find(n=>n.meta.id===id)!),
  run:async()=>({batchId:'preview',topics:0,assignments:0}),undo:noop,renameTopic:noop,setTopicColumn:noop,mergeTopics:noop,splitNote:async()=>'',setLocked:noop,deleteTopic:noop,
  listRelations:async()=>relations,addRelation:async()=>'',removeRelation:noop,setAuto:noop,getAuto:async()=>false,
 };
 let knowledgeState=$state<KnowledgeState>({notes:[],query:'',current:null,busy:false,loaded:false,notice:'',topics:[],organizeMode:false,selected:{},confirming:false,organizeBusy:false,lastBatch:null,lastSummary:null,topicView:null,relations:[],auto:false});
 const knowledge=new KnowledgeController(port,s=>knowledgeState=s);
 const paper={id:'synthetic-paper',name:'说话人相关音频伪造检测：方法与实验综述.pdf',pageCount:12,readablePages:12};
 const paperPort={defaults:async()=>defaultReadingOptions(),list:async()=>[paper],choose:async()=>paper,open:noop,source:async()=>['本页为合成原文，仅用于验证页面布局。']};
 let paperState=$state<PaperDraftState>({enabled:false,restoring:false,recoveryAvailable:false,loaded:true,importing:false,paper:null,options:defaultReadingOptions(),notice:''});
 const paperController=new PaperDraftController(paperPort,s=>paperState=s);
 let settings=$state<SettingsState>({config:null,revision:0,keyAvailable:true,keySource:'keyring',fields:{baseUrl:'https://example.test/v1',model:'研究助手',keyEnv:'ZWEN_API_KEY',imageModel:''},status:'saved',loaded:true,error:null,secretSaving:false,secretNotice:null});
 let appearance=$state<AppearanceState>({theme:params.get('theme')==='dark'?'dark':'light',sidebarCollapsed:false,loaded:true,status:'saved',error:null});
 let category=$state<SettingsCategory>((['provider','appearance','selection','quick'].includes(params.get('category')??'')?params.get('category'):'provider') as SettingsCategory);
 let enabled=$state(true);
 let quickConfig:QuickActionConfig={revision:1,actions:defaultQuickActions(),outputDirectory:'/synthetic-preview/documents'};
 const quickPort={load:async()=>quickConfig,save:async(revision:number,actions:QuickActionConfig['actions'],outputDirectory:string)=>quickConfig={revision:revision+1,actions,outputDirectory}};
 let ready=$state(false);
 let collapsed=$state(window.innerWidth<700);
 const nav=[{screen:'home',icon:'plus'},{screen:'knowledge',icon:'book'},{screen:'graph',icon:'network'},{screen:'settings',icon:'settings'}] as const;
 onMount(()=>{void (async()=>{await knowledge.refresh();await knowledge.loadTopics();await knowledge.loadRelations();await paperController.prepare();if(screen==='paper'){paperController.attach(paper);paperController.edit({background:'说话人相关的音频伪造检测，关注跨域泛化、可迁移方法与实验设计。'});}await tick();if(screen==='graph'&&params.get('level')==='topics'){document.querySelector<HTMLButtonElement>('.graph-level button:last-child')?.click();await tick();}ready=true;})();return()=>{knowledge.dispose();paperController.dispose();};});
</script>
<div class="shell" class:sidebar-collapsed={collapsed} data-preview-ready={ready?'true':'false'}>
 <aside class="sidebar" class:collapsed><div class="sidebar-brand"><BrandLogo/>{#if !collapsed}<strong>Zwen</strong>{/if}</div><button class="sidebar-toggle" aria-label="切换侧栏" onclick={()=>collapsed=!collapsed}><Icon name={collapsed?'expand':'collapse'}/>{#if !collapsed}收起侧栏{/if}</button><nav aria-label="主导航">{#each nav as item}<button class:active={screen===item.screen} class:new-chat={item.screen==='home'} class:settings-link={item.screen==='settings'} aria-label={labels[item.screen]} onclick={()=>screen=item.screen}><Icon name={item.icon}/>{#if !collapsed}<span>{labels[item.screen]}</span>{/if}</button>{/each}</nav></aside>
 <div class="main-panel"><div class="titlebar"><span class="context-title">{labels[screen]}</span><div class="window-actions"><button class="capsule-launch" onclick={()=>{screen='paper';paperController.attach(paper);}}>论文阅读</button><button class="icon-button" aria-label="切换明暗" onclick={()=>{appearance.theme=appearance.theme==='dark'?'light':'dark';document.documentElement.dataset.theme=appearance.theme;}}><Icon name={appearance.theme==='dark'?'moon':'sun'}/></button></div></div>
 <div class="workspace"><main>
 {#if screen==='home'||screen==='paper'}<div class="empty-conversation"><div class="welcome-scene"><WelcomeMark/><p>今天想问 Zwen 什么问题？</p></div></div>
 {:else if screen==='knowledge'}<KnowledgePanel controller={knowledge} info={knowledgeState} onTrace={()=>{}} onSaveEdit={(id,title,body,source)=>knowledge.saveEdit(id,title,body,source)} modelLabel="演示模型"/>
 {:else if screen==='graph'}<KnowledgeGraph controller={knowledge} info={knowledgeState} onOpenNote={id=>{void knowledge.open(id);screen='knowledge';}} onOpenTopic={id=>void knowledge.openTopic(id)}/>
 {:else}<SettingsHub bind:category>
 {#snippet providerPanel()}<ProviderSettings {settings} desktop={true} onEdit={(name,value)=>settings.fields[name]=value} onSave={()=>settings.status='saved'} onReload={()=>{}} onSaveSecret={noop} onClearSecret={noop}/>{/snippet}
 {#snippet appearancePanel()}<AppearanceSettings state={appearance} onTheme={theme=>{appearance.theme=theme;document.documentElement.dataset.theme=theme==='dark'?'dark':'light';}} onReload={()=>{}}/>{/snippet}
 {#snippet selectionPanel()}<SelectionSettings desktop={true} {enabled} onToggle={value=>enabled=value}/>{/snippet}
 {#snippet quickPanel()}<QuickActionSettings port={quickPort} onChanged={()=>{}}/>{/snippet}
 </SettingsHub>{/if}
 </main>
 {#if screen!=='settings'}<section class="composer" aria-label="问题草稿"><PaperComposer controller={paperController} state={paperState} port={paperPort} destination="https://example.test/v1 · 演示模型" paperModel="演示模型" onExit={()=>paperController.close()}/><textarea aria-label="问题草稿" placeholder={screen==='paper'?'可直接发送开始精读，或补充你想了解的问题…':'输入问题，或粘贴内容…'} rows="2"></textarea><div class="composer-tools"><div><button class="tool-button" aria-label="附加截图"><Icon name="image"/></button><button class="tool-button" aria-label="保存草稿"><Icon name="save" size={18}/></button></div><button class="primary-button" aria-label="发送（仅视觉演示）"><Icon name="send"/><kbd>Enter ↵</kbd></button></div></section>{/if}
 <footer><span><i></i>已保存</span><span>合成预览 · 不连接后端或模型</span></footer></div></div>
</div>
