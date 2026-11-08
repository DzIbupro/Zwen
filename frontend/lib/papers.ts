import type {ChatReply} from './chat';
export interface ReadingOptions {mode:'deep'|'quick'|'reproduce'|'interactive';startPage:number;endPage:number;focus:string;background:string}
export interface PaperMetadata {id:string;name:string;pageCount:number;readablePages:number}
export interface LegacyRun {runId:string;name:string;state:string}
export interface SessionInfo {source:PaperMetadata;options:ReadingOptions;endpoint:string;model:string}
export interface PaperPort {legacyPapers?():Promise<PaperMetadata[]>;legacyOpen?(id:string):Promise<void>;sessionInfo?(id:string):Promise<SessionInfo>;legacy?():Promise<LegacyRun[]>;legacyReport?(id:string):Promise<string>;list():Promise<PaperMetadata[]>;choose():Promise<PaperMetadata|null>;sessions(id:string):Promise<ChatReply[]>;source(id:string):Promise<string[]>}
export const defaultReadingOptions=():ReadingOptions=>({mode:'deep',startPage:1,endPage:0,focus:'',background:''});
