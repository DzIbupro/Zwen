import type {Draft} from './draft';
export const DRAFT_KEY='zwen.browser-draft.v1';
export const LEGACY_DRAFT_KEY='study-assistant.browser-draft.v1';
export function readBrowserDraft(storage:Pick<Storage,'getItem'>):Draft {
 const raw=storage.getItem(DRAFT_KEY)??storage.getItem(LEGACY_DRAFT_KEY);
 if(raw===null)return {text:'',revision:0};
 const d:unknown=JSON.parse(raw);
 if(typeof d!=='object'||!d||!('text' in d)||typeof d.text!=='string'||!('revision' in d)||!Number.isSafeInteger(d.revision)||Number(d.revision)<0)throw Error('Zwen 草稿无法读取；原内容已保留');
 return d as Draft;
}
