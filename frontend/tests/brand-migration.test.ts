import {it,expect} from 'vitest';
import {readBrowserDraft,DRAFT_KEY,LEGACY_DRAFT_KEY} from '../lib/browser-draft';
it('restores the legacy browser draft without overwriting it',()=>{const values=new Map([[LEGACY_DRAFT_KEY,JSON.stringify({text:'旧草稿',revision:7})]]);const s={getItem:(key:string)=>values.get(key)??null};expect(readBrowserDraft(s)).toEqual({text:'旧草稿',revision:7});expect(values.has(DRAFT_KEY)).toBe(false);});
it('does not fall back over a damaged new draft',()=>{const s={getItem:(key:string)=>key===DRAFT_KEY?'damaged':JSON.stringify({text:'old',revision:1})};expect(()=>readBrowserDraft(s)).toThrow();});
it('prefers the valid Zwen draft over the legacy backup',()=>{const s={getItem:(key:string)=>JSON.stringify(key===DRAFT_KEY?{text:'新草稿',revision:8}:{text:'旧草稿',revision:7})};expect(readBrowserDraft(s)).toEqual({text:'新草稿',revision:8});});
