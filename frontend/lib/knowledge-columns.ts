import type {TopicView} from './knowledge';
export interface KnowledgeColumn {name:string;topics:TopicView[];noteCount:number}
// View grouping only: topic ids/memberships remain the original records.
export function knowledgeColumns(topics:TopicView[]):KnowledgeColumn[]{
 const columns=new Map<string,TopicView[]>();
 for(const topic of topics){const name=topic.topic.column.trim()||'未分栏目';const items=columns.get(name)??[];items.push(topic);columns.set(name,items);}
 return [...columns].map(([name,items])=>({name,topics:items,noteCount:new Set(items.flatMap(t=>t.noteIds)).size}));
}
