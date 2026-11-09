import type {ChatReply} from './chat';
export interface ReadingOptions {mode:'deep'|'quick'|'reproduce'|'interactive';startPage:number;endPage:number;focus:string;outputRequirements:string;background:string;readImages:boolean}
export interface PaperMetadata {id:string;name:string;pageCount:number;readablePages:number}
export interface MetadataPage {items:PaperMetadata[];nextCursor:number|null}
export interface LegacyRun {runId:string;name:string;state:string}
export interface SessionInfo {source:PaperMetadata;options:ReadingOptions;endpoint:string;model:string}
export interface PaperPort {importPdf?(path:string):Promise<PaperMetadata>;open?(id:string):Promise<void>;get?(id:string):Promise<PaperMetadata>;listPage?(cursor?:number|null):Promise<MetadataPage>;defaults():Promise<ReadingOptions>;legacyPapers?():Promise<PaperMetadata[]>;legacyOpen?(id:string):Promise<void>;sessionInfo?(id:string):Promise<SessionInfo>;legacy?():Promise<LegacyRun[]>;legacyReport?(id:string):Promise<string>;list():Promise<PaperMetadata[]>;choose():Promise<PaperMetadata|null>;sessions(id:string):Promise<ChatReply[]>;source(id:string):Promise<string[]>}
export const defaultReadingOptions=():ReadingOptions=>({mode:'deep',startPage:1,endPage:0,focus:'',outputRequirements:'',background:'',readImages:false});
