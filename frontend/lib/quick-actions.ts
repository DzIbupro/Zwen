import type {Locale} from './i18n.svelte';

export type QuickAction={id:string;name:string;prompt:string;output:'display'|'document';enabled:boolean};
export type QuickActionConfig={revision:number;actions:QuickAction[];outputDirectory:string};
export type QuickActionPort={load():Promise<QuickActionConfig>;save(expectedRevision:number,actions:QuickAction[],outputDirectory:string):Promise<QuickActionConfig>};
export type ActionDocument={noteId:string;path:string};

type BuiltinId='translate'|'explain'|'document';

const PRESETS_BY_LOCALE:Record<Locale,Record<BuiltinId,{name:string;prompt:string}>>={
 'zh-CN':{
  translate:{name:'翻译',prompt:'请将以下选中文字翻译成中文，保留原意：\n\n{{selection}}'},
  explain:{name:'详细解释',prompt:'请详细解释以下选中文字：\n\n{{selection}}'},
  document:{name:'整理成文档',prompt:'请将以下选中文字整理为结构清晰的 Markdown 文档：\n\n{{selection}}'},
 },
 en:{
  translate:{name:'Translate',prompt:'Please translate the following selected text into English while preserving its original meaning:\n\n{{selection}}'},
  explain:{name:'Explain',prompt:'Please explain the following selected text in detail:\n\n{{selection}}'},
  document:{name:'Format as Doc',prompt:'Please organize the following selected text into a well-structured Markdown document:\n\n{{selection}}'},
 },
 ja:{
  translate:{name:'翻訳',prompt:'以下の選択されたテキストを元の意味を保ったまま日本語に翻訳してください：\n\n{{selection}}'},
  explain:{name:'詳しく解説',prompt:'以下の選択されたテキストについて詳しく解説してください：\n\n{{selection}}'},
  document:{name:'文書に整理',prompt:'以下の選択されたテキストを構造化された分かりやすい Markdown 文書に整理してください：\n\n{{selection}}'},
 },
 ko:{
  translate:{name:'번역',prompt:'다음 선택한 텍스트를 원래 의미를 유지하면서 한국어로 번역해 주세요:\n\n{{selection}}'},
  explain:{name:'상세 설명',prompt:'다음 선택한 텍스트를 자세히 설명해 주세요:\n\n{{selection}}'},
  document:{name:'문서로 정리',prompt:'다음 선택한 텍스트를 구조가 명확한 Markdown 문서로 정리해 주세요:\n\n{{selection}}'},
 },
 es:{
  translate:{name:'Traducir',prompt:'Por favor, traduce el siguiente texto seleccionado al español manteniendo su significado original:\n\n{{selection}}'},
  explain:{name:'Explicar',prompt:'Por favor, explica detalladamente el siguiente texto seleccionado:\n\n{{selection}}'},
  document:{name:'Organizar en doc',prompt:'Por favor, organiza el siguiente texto seleccionado en un documento Markdown bien estructurado:\n\n{{selection}}'},
 },
};

const KNOWN_PRESET_NAMES:Record<BuiltinId,Set<string>>={
 translate:new Set(Object.values(PRESETS_BY_LOCALE).map(p=>p.translate.name)),
 explain:new Set(Object.values(PRESETS_BY_LOCALE).map(p=>p.explain.name)),
 document:new Set(Object.values(PRESETS_BY_LOCALE).map(p=>p.document.name)),
};

const KNOWN_PRESET_PROMPTS:Record<BuiltinId,Set<string>>={
 translate:new Set(Object.values(PRESETS_BY_LOCALE).map(p=>p.translate.prompt.trim())),
 explain:new Set(Object.values(PRESETS_BY_LOCALE).map(p=>p.explain.prompt.trim())),
 document:new Set(Object.values(PRESETS_BY_LOCALE).map(p=>p.document.prompt.trim())),
};

export function defaultQuickActions(locale:Locale='zh-CN'):QuickAction[]{
 const p=PRESETS_BY_LOCALE[locale]??PRESETS_BY_LOCALE['zh-CN'];
 return [
  {id:'translate',name:p.translate.name,prompt:p.translate.prompt,output:'display',enabled:true},
  {id:'explain',name:p.explain.name,prompt:p.explain.prompt,output:'display',enabled:true},
  {id:'document',name:p.document.name,prompt:p.document.prompt,output:'document',enabled:true},
 ];
}

/**
 * Updates built-in quick actions (`translate`, `explain`, `document`) to match `locale`
 * if their current name/prompt is still one of the standard preset values (preserving
 * any custom user-edited prompts).
 */
export function localizePresetQuickActions(actions:QuickAction[],locale:Locale):{actions:QuickAction[];changed:boolean}{
 const target=PRESETS_BY_LOCALE[locale]??PRESETS_BY_LOCALE['zh-CN'];
 let changed=false;
 const next=actions.map(action=>{
  if(action.id!=='translate'&&action.id!=='explain'&&action.id!=='document')return action;
  const id=action.id as BuiltinId;
  const preset=target[id];
  const nextName=KNOWN_PRESET_NAMES[id].has(action.name.trim())?preset.name:action.name;
  const nextPrompt=KNOWN_PRESET_PROMPTS[id].has(action.prompt.trim())?preset.prompt:action.prompt;
  if(nextName!==action.name||nextPrompt!==action.prompt){
   changed=true;
   return {...action,name:nextName,prompt:nextPrompt};
  }
  return action;
 });
 return {actions:next,changed};
}

const bytes=(s:string)=>new TextEncoder().encode(s).length;
export function validateQuickActions(actions:QuickAction[],directory:string):string|null{
 if(!directory.startsWith('/')||bytes(directory)>4096||/[\u0000-\u001f\u007f]/.test(directory))return '请填写绝对文档输出目录';
 if(bytes(JSON.stringify({revision:0,actions,outputDirectory:directory}))>600000)return '快捷提示词配置过大';
 if(actions.length<2||actions.length>32)return '动作数量须为 2–32 个';
 const ids=new Set<string>();for(const a of actions){if(!/^[A-Za-z0-9_-]{1,64}$/.test(a.id)||ids.has(a.id))return '动作标识无效或重复';ids.add(a.id);if(!a.name.trim()||bytes(a.name)>200||/[\u0000-\u001f\u007f]/.test(a.name))return '动作名称无效';if(!a.prompt.trim()||bytes(a.prompt)>16384||a.prompt.includes('\0'))return '提示词须为 1–16384 字节';if(!['display','document'].includes(a.output)||typeof a.enabled!=='boolean')return '动作输出方式无效';}
 return ids.has('translate')&&ids.has('explain')?null:'请保留翻译和详细解释动作';
}
export function renderQuickActionPrompt(prompt:string,selection:string):string{return prompt.includes('{{selection}}')?prompt.split('{{selection}}').join(selection):`${prompt}\n\n${selection}`;}
