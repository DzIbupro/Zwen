import { invoke } from '@tauri-apps/api/core';
import type { Draft, DraftPort } from './draft';
export const isDesktop = '__TAURI_INTERNALS__' in window;
import {readBrowserDraft,DRAFT_KEY} from './browser-draft';
async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try { return await invoke<T>(name, args); }
  catch (error) {
    const message = typeof error === 'object' && error && 'message' in error ? String(error.message) : '本地操作未成功，当前内容已保留';
    throw Error(message);
  }
}
function browserDraft(): Draft {return readBrowserDraft(localStorage);}
export const draftPort: DraftPort = isDesktop ? {
  load: ()=>command<Draft>('load_draft'),
  save: (revision, text)=>command<Draft>('save_draft', { expectedRevision: revision, text }),
} : {
  load: async()=>browserDraft(),
  save: async(revision, text)=>{
    if (new TextEncoder().encode(text).length > 256*1024) throw Error('草稿超过 256 KiB，请拆分内容后保存');
    if (browserDraft().revision !== revision) throw Error('草稿已在别处更新；当前输入已保留');
    const d={text,revision:revision+1};localStorage.setItem(DRAFT_KEY,JSON.stringify(d));return d;
  },
};
export const hideWindow = ()=>command<void>('hide_window');
export const quitApp = ()=>command<void>('quit_app');

import type { SurfaceInfo } from './surface';
let browserSurface:SurfaceInfo={scaleFactor:1,mode:new URLSearchParams(location.search).get('surface')==='capsule'?'capsule':'full',generation:0,bounds:{width:Math.min(480,Math.max(1,innerWidth-32)),initialHeight:96,maxHeight:Math.max(96,Math.floor(innerHeight*.55))}};
export const getSurface=():Promise<SurfaceInfo>=>isDesktop?command('get_surface'):Promise.resolve(browserSurface);
export const setSurface=(mode:'full'|'capsule'):Promise<SurfaceInfo>=>{
  if(isDesktop)return command('set_surface',{mode});
  browserSurface={...browserSurface,mode,generation:browserSurface.generation+1};return Promise.resolve(browserSurface);
};
export const startCapsuleDrag=():Promise<void>=>isDesktop?command('plugin:window|start_dragging'):Promise.resolve();
export const resizeCapsule=(height:number,generation:number):Promise<void>=>isDesktop?command('resize_capsule',{height,generation}):Promise.resolve();

import type {ProviderPort,ProviderView} from './provider';
export const providerPort:ProviderPort=isDesktop?{
 load:()=>command<ProviderView>('load_provider'),
 save:(revision,config)=>command<ProviderView>('save_provider',{expectedRevision:revision,config}),
 saveSecret:(value,expectedRevision)=>command<void>('save_model_secret',{secret:value,expectedRevision}),
 clearSecret:()=>command<void>('clear_model_secret'),
}:{load:async()=>({config:null,revision:0,keyAvailable:false,keySource:null}),save:async()=>{throw Error('浏览器仅预览设置，请在桌面应用中保存');},saveSecret:async()=>{throw Error('浏览器仅用于预览；请在桌面应用中保存密钥');},clearSecret:async()=>{throw Error('浏览器仅用于预览；请在桌面应用中操作');}};

import {listen} from '@tauri-apps/api/event';
import type {ChatPort,ChatReply,Overview} from './chat';
import type {ReadingOptions,PaperMetadata} from './papers';
export const chatPort:ChatPort=isDesktop?{
 startPaper:(requestId,paperId,options)=>command<ChatReply>('paper_start',{requestId,paperId,options}),
 listen:cb=>listen<ChatReply>('chat-update',e=>cb(e.payload)),
 load:()=>command<Overview>('get_chat'),startSelection:(requestId,selection,actionId,expectedRevision)=>command<ChatReply>('start_selection_chat',{requestId,selection,actionId,expectedRevision}),start:(requestId,prompt,conversationId)=>command<ChatReply>('start_chat',{requestId,prompt,conversationId:conversationId??null}),
 stop:requestId=>command<ChatReply|null>('stop_chat',{requestId}),settle:()=>command<ChatReply|null>('settle_chat'),
}:{listen:async()=>()=>{},load:async()=>({reply:null,activeRequestId:null}),start:async()=>{throw Error('浏览器仅用于预览，请在桌面应用中发送');},stop:async()=>null,settle:async()=>null};

import type {HistoryPort,Summary} from './history';
export const historyPort:HistoryPort=isDesktop?{list:page=>command<Summary[]>('get_chat_history',{page}),get:requestId=>command<ChatReply|null>('get_chat_turn',{requestId}),remove:requestId=>command<void>('delete_chat_turn',{requestId}),removeMany:requestIds=>command<number>('delete_chat_history',{scope:{kind:'selected',requestIds}}),clear:()=>command<number>('delete_chat_history',{scope:{kind:'all'}})}:{list:async()=>[],get:async()=>null};

import type {KnowledgePort,KnowledgeState} from './knowledge';
import type {SaveNoteInput,Note,NoteMeta,TopicView,OrganizeSummaryView,SourceMetaInput,RelationView} from './knowledge';
export const knowledgePort:KnowledgePort=isDesktop?{
 rounds:{prepare:noteIds=>command('organize_prepare',{noteIds}),status:()=>command('organize_round_status'),start:roundId=>command('organize_round_start',{roundId}),stop:roundId=>command('organize_round_stop',{roundId}),undo:roundId=>command('organize_round_undo',{roundId})},
 list:query=>command<NoteMeta[]>('notes_list',{query}),
 read:id=>command<Note>('notes_read',{id}),
 save:(input:SaveNoteInput)=>command<NoteMeta>('notes_save',{input}),
 remove:id=>command<void>('notes_delete',{id}),
 update:(id:string,title:string,body:string,source:SourceMetaInput)=>command<NoteMeta>('notes_update',{id,title,body,sourceTitle:source.sourceTitle??null,sourceUrl:source.sourceUrl??null,sourceLoc:source.sourceLoc??null}),
 updateSource:(id:string,source:SourceMetaInput)=>command<void>('notes_update_source',{id,sourceTitle:source.sourceTitle??null,sourceUrl:source.sourceUrl??null,sourceLoc:source.sourceLoc??null}),
 exportNote:id=>command<string>('note_export',{id}),
 exportAll:()=>command<string>('notes_export_all'),
 rebuild:()=>command<number>('notes_rebuild'),
 topics:()=>command<TopicView[]>('organize_topics'),
 topicNotes:topicId=>command<Note[]>('organize_topic_notes',{topicId}),
 run:(noteIds,automatic=false)=>command<OrganizeSummaryView>('organize_run',{noteIds,automatic}),
 undo:batchId=>command<void>('organize_undo',{batchId}),
 renameTopic:(topicId,name)=>command<void>('organize_rename',{topicId,name}),
 setTopicColumn:(topicId,column)=>command<void>('organize_set_column',{topicId,column}),
 mergeTopics:(source,target)=>command<void>('organize_merge',{source,target}),
 splitNote:(noteId,from,toName,column)=>command<string>('organize_split',{noteId,from,toName,column}),
 setLocked:(topicId,locked)=>command<void>('organize_set_locked',{topicId,locked}),
 deleteTopic:topicId=>command<void>('organize_delete_topic',{topicId}),
 listRelations:()=>command<RelationView[]>('organize_list_relations'),
 addRelation:(fromTopic,toTopic,reason,evidence)=>command<string>('organize_add_relation',{fromTopic,toTopic,reason,evidence}),
 removeRelation:relationId=>command<void>('organize_remove_relation',{relationId}),
 setAuto:on=>command<void>('organize_set_auto',{on}),
 getAuto:()=>command<boolean>('organize_get_auto'),
}:{list:async()=>[],read:async()=>{throw Error('浏览器仅用于预览；请在桌面应用中使用知识库');},save:async()=>{throw Error('浏览器仅用于预览；请在桌面应用中收藏');},remove:async()=>{},update:async()=>{throw Error('浏览器仅用于预览；请在桌面应用中编辑');},updateSource:async()=>{throw Error('浏览器仅用于预览');},exportNote:async()=>{throw Error('浏览器仅用于预览');},exportAll:async()=>{throw Error('浏览器仅用于预览');},topics:async()=>[],topicNotes:async()=>[],run:async()=>{throw Error('浏览器仅用于预览；请在桌面应用中整理');},undo:async()=>{},renameTopic:async()=>{},setTopicColumn:async()=>{},mergeTopics:async()=>{},splitNote:async()=>{throw Error('浏览器仅用于预览');},setLocked:async()=>{},deleteTopic:async()=>{},listRelations:async()=>[],addRelation:async()=>{throw Error('浏览器仅用于预览');},removeRelation:async()=>{},setAuto:async()=>{},getAuto:async()=>false,rebuild:async()=>{throw Error('浏览器仅用于预览');}};

import type {SelectionPort,SelectionItem} from './selection';
export const openExternal=(url:string)=>command<void>('open_external',{url});
export const invokeRaw=invoke;
export const selectionPort:SelectionPort=isDesktop?{freeze:id=>command<SelectionItem>('freeze_selection',{id}),claim:()=>command<{reading:boolean;item:SelectionItem|null}>('claim_selection'),acknowledge:id=>command<void>('acknowledge_selection',{id})}:{claim:async()=>({reading:false,item:null}),acknowledge:async()=>{}};

import type {AttachmentPort,AttachmentView} from './attachments';
export const attachmentPort:AttachmentPort=isDesktop?{
 list:()=>command<AttachmentView>('get_attachments'),
 capture:replaceId=>command<void>('capture_screenshot',{replaceId:replaceId??null}),
 cancel:()=>command<void>('cancel_screenshot'),
 remove:id=>command<void>('remove_attachment',{id}),
 listen:cb=>listen('attachments-changed',()=>cb()),
}:{list:async()=>({items:[],capture:{state:'idle',requestId:null,replaceId:null},notice:null,max:3}),capture:async()=>{throw Error('浏览器仅用于预览，请在桌面应用中截图');},cancel:async()=>{},remove:async()=>{},listen:async()=>()=>{}};

import type {QuickActionPort} from './quick-actions';
import type {ActionOutputPort} from './action-output';
export const quickActionPort:QuickActionPort={
 load:()=>command('load_quick_actions'),
 save:(expectedRevision,actions,outputDirectory)=>command('save_quick_actions',{expectedRevision,actions,outputDirectory}),
};
export const actionOutputPort:ActionOutputPort={save:({requestId,selection,actionId,actionName,outputDirectory})=>command('save_action_document',{requestId,selection,actionId,actionName,outputDirectory})};

import {APPEARANCE_KEY,DEFAULT_APPEARANCE,parseAppearance,type Appearance,type AppearancePort} from './appearance';
export const appearancePort:AppearancePort=isDesktop?{load:()=>command<Appearance>('load_appearance'),save:preferences=>command<Appearance>('save_appearance',{preferences})}:{
 load:async()=>{const raw=localStorage.getItem(APPEARANCE_KEY);if(raw===null)return{...DEFAULT_APPEARANCE};const value=parseAppearance(JSON.parse(raw));if(!value)throw Error('外观偏好格式无效，请重新读取');return value;},
 save:async preferences=>{localStorage.setItem(APPEARANCE_KEY,JSON.stringify(preferences));return preferences;}
};

export const paperPort={
 defaults:()=>command<ReadingOptions>('paper_defaults'),
 legacyPapers:()=>isDesktop?command<PaperMetadata[]>('paper_legacy_papers'):Promise.resolve([]),
 legacyOpen:(paperId:string)=>command<void>('paper_legacy_open',{paperId}),
 sessionInfo:(conversationId:string)=>command<import('./papers').SessionInfo>('paper_session_info',{conversationId}),
 legacy:()=>isDesktop?command<import('./papers').LegacyRun[]>('paper_legacy'):Promise.resolve([]),
 legacyReport:(runId:string)=>command<string>('paper_legacy_report',{runId}),
 list:()=>isDesktop?command<PaperMetadata[]>('paper_list'):Promise.resolve([]),
 choose:()=>command<PaperMetadata|null>('paper_choose'),
 sessions:(paperId:string)=>command<ChatReply[]>('paper_sessions',{paperId}).then(rows=>rows.map(r=>({...r,saved:true}))),
 source:(paperId:string)=>command<string[]>('paper_source',{paperId}),
};
