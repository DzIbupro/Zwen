import { invoke } from '@tauri-apps/api/core';
import type { Draft, DraftPort } from './draft';
export const isDesktop = '__TAURI_INTERNALS__' in window;
const KEY = 'study-assistant.browser-draft.v1';
async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try { return await invoke<T>(name, args); }
  catch (error) {
    const message = typeof error === 'object' && error && 'message' in error ? String(error.message) : '本地操作未成功，当前内容已保留';
    throw Error(message);
  }
}
function browserDraft(): Draft {
  const raw = localStorage.getItem(KEY);
  if (!raw) return { text: '', revision: 0 };
  const d: unknown = JSON.parse(raw);
  if (typeof d !== 'object' || !d || !('text' in d) || typeof d.text !== 'string' || !('revision' in d) || !Number.isSafeInteger(d.revision) || Number(d.revision) < 0) throw Error('浏览器草稿无法读取；原内容已保留');
  return d as Draft;
}
export const draftPort: DraftPort = isDesktop ? {
  load: ()=>command<Draft>('load_draft'),
  save: (revision, text)=>command<Draft>('save_draft', { expectedRevision: revision, text }),
} : {
  load: async()=>browserDraft(),
  save: async(revision, text)=>{
    if (new TextEncoder().encode(text).length > 256*1024) throw Error('草稿超过 256 KiB，请拆分内容后保存');
    if (browserDraft().revision !== revision) throw Error('草稿已在别处更新；当前输入已保留');
    const d={text,revision:revision+1};localStorage.setItem(KEY,JSON.stringify(d));return d;
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
export const resizeCapsule=(height:number,generation:number):Promise<void>=>isDesktop?command('resize_capsule',{height,generation}):Promise.resolve();
