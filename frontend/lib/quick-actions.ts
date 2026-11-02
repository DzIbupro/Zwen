export type QuickAction={id:string;name:string;prompt:string;output:'display'|'document';enabled:boolean};
export type QuickActionConfig={revision:number;actions:QuickAction[];outputDirectory:string};
export type QuickActionPort={load():Promise<QuickActionConfig>;save(expectedRevision:number,actions:QuickAction[],outputDirectory:string):Promise<QuickActionConfig>};
export type ActionDocument={noteId:string;path:string};
export function defaultQuickActions():QuickAction[]{return [
 {id:'translate',name:'翻译',prompt:'请将以下选中文字翻译成中文，保留原意：\n\n{{selection}}',output:'display',enabled:true},
 {id:'explain',name:'详细解释',prompt:'请详细解释以下选中文字：\n\n{{selection}}',output:'display',enabled:true},
 {id:'document',name:'整理成文档',prompt:'请将以下选中文字整理为结构清晰的 Markdown 文档：\n\n{{selection}}',output:'document',enabled:true}
];}
const bytes=(s:string)=>new TextEncoder().encode(s).length;
export function validateQuickActions(actions:QuickAction[],directory:string):string|null{
 if(!directory.startsWith('/')||bytes(directory)>4096||/[\u0000-\u001f\u007f]/.test(directory))return '请填写绝对文档输出目录';
 if(bytes(JSON.stringify({revision:0,actions,outputDirectory:directory}))>600000)return '快捷提示词配置过大';
 if(actions.length<2||actions.length>32)return '动作数量须为 2–32 个';
 const ids=new Set<string>();for(const a of actions){if(!/^[A-Za-z0-9_-]{1,64}$/.test(a.id)||ids.has(a.id))return '动作标识无效或重复';ids.add(a.id);if(!a.name.trim()||bytes(a.name)>200||/[\u0000-\u001f\u007f]/.test(a.name))return '动作名称无效';if(!a.prompt.trim()||bytes(a.prompt)>16384||a.prompt.includes('\0'))return '提示词须为 1–16384 字节';if(!['display','document'].includes(a.output)||typeof a.enabled!=='boolean')return '动作输出方式无效';}
 return ids.has('translate')&&ids.has('explain')?null:'请保留翻译和详细解释动作';
}
export function renderQuickActionPrompt(prompt:string,selection:string):string{return prompt.includes('{{selection}}')?prompt.split('{{selection}}').join(selection):`${prompt}\n\n${selection}`;}
