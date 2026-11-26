import type { InactiveConversation, Summary } from './history';
import { summarizeTitle } from './title-summary';

export const HISTORY_ORG_STORAGE_KEY = 'zwen.historyOrg.v1';

export interface ChatProject {
  id: string;
  name: string;
  createdAt: number;
}

export interface ArchivedRecord {
  requestId: string;
  conversationId: string;
  title: string;
  model: string;
  createdAt: number;
  archivedAt: number;
  projectId: string | null;
  automatic?: true;
}

export interface ProjectItemSnapshot {
  requestId: string;
  conversationId: string;
  title: string;
  model: string;
  createdAt: number;
}

export const AUTO_ARCHIVE_INTERVALS = [0, 30, 90, 180] as const;
export type AutoArchiveDays = typeof AUTO_ARCHIVE_INTERVALS[number];
interface AutomaticArchive {
  record: ArchivedRecord;
  memberIds: string[];
}

interface PersistedHistoryOrg {
  version: 1;
  customTitles: Record<string, string>;
  pinnedOrder: string[];
  archived: Record<string, ArchivedRecord>;
  autoArchiveDays: AutoArchiveDays;
  automaticArchives: Record<string, AutomaticArchive>;
  archiveRestoredAt: Record<string, number>;
  projects: ChatProject[];
  itemProjects: Record<string, string>;
  conversationProjects: Record<string, string>;
  projectSnapshots: Record<string, ProjectItemSnapshot>;
  activeProjectId: string | null;
}

const EMPTY_STATE: PersistedHistoryOrg = {
  version: 1,
  customTitles: {},
  pinnedOrder: [],
  archived: {},
  autoArchiveDays: 0,
  automaticArchives: {},
  archiveRestoredAt: {},
  projects: [],
  itemProjects: {},
  conversationProjects: {},
  projectSnapshots: {},
  activeProjectId: null,
};

function sanitize(raw: unknown): PersistedHistoryOrg {
  if (!raw || typeof raw !== 'object') return structuredClone(EMPTY_STATE);
  const obj = raw as Partial<PersistedHistoryOrg>;
  const projects: ChatProject[] = Array.isArray(obj.projects)
    ? obj.projects
        .filter(
          (p): p is ChatProject =>
            !!p &&
            typeof p === 'object' &&
            typeof p.id === 'string' &&
            p.id.length > 0 &&
            typeof p.name === 'string' &&
            p.name.trim().length > 0
        )
        .map(p => ({
          id: p.id,
          name: p.name.trim().slice(0, 80),
          createdAt: typeof p.createdAt === 'number' ? p.createdAt : Date.now(),
        }))
    : [];
  const validProjectIds = new Set(projects.map(p => p.id));

  const customTitles: Record<string, string> = {};
  if (obj.customTitles && typeof obj.customTitles === 'object') {
    for (const [k, v] of Object.entries(obj.customTitles)) {
      if (typeof v === 'string' && v.trim()) customTitles[k] = v.trim().slice(0, 120);
    }
  }

  const pinnedOrder = Array.isArray(obj.pinnedOrder)
    ? [...new Set(obj.pinnedOrder.filter((id): id is string => typeof id === 'string' && id.length > 0))]
    : [];

  const archived: Record<string, ArchivedRecord> = {};
  if (obj.archived && typeof obj.archived === 'object') {
    for (const [k, v] of Object.entries(obj.archived)) {
      if (v && typeof v === 'object' && typeof v.requestId === 'string') {
        archived[k] = {
          requestId: v.requestId,
          conversationId: typeof v.conversationId === 'string' ? v.conversationId : v.requestId,
          title: typeof v.title === 'string' ? v.title : '',
          model: typeof v.model === 'string' ? v.model : '',
          createdAt: typeof v.createdAt === 'number' ? v.createdAt : 0,
          archivedAt: typeof v.archivedAt === 'number' ? v.archivedAt : Date.now(),
          projectId: typeof v.projectId === 'string' && validProjectIds.has(v.projectId) ? v.projectId : null,
        };
      }
    }
  }

  const itemProjects: Record<string, string> = {};
  if (obj.itemProjects && typeof obj.itemProjects === 'object') {
    for (const [k, v] of Object.entries(obj.itemProjects)) {
      if (typeof v === 'string' && validProjectIds.has(v)) itemProjects[k] = v;
    }
  }

  const conversationProjects: Record<string, string> = {};
  if (obj.conversationProjects && typeof obj.conversationProjects === 'object') {
    for (const [k, v] of Object.entries(obj.conversationProjects)) {
      if (typeof v === 'string' && validProjectIds.has(v)) conversationProjects[k] = v;
    }
  }

  const projectSnapshots: Record<string, ProjectItemSnapshot> = {};
  if (obj.projectSnapshots && typeof obj.projectSnapshots === 'object') {
    for (const [k, v] of Object.entries(obj.projectSnapshots)) {
      if (v && typeof v === 'object' && typeof v.requestId === 'string' && k in itemProjects) {
        projectSnapshots[k] = {
          requestId: v.requestId,
          conversationId: typeof v.conversationId === 'string' ? v.conversationId : v.requestId,
          title: typeof v.title === 'string' ? v.title : '',
          model: typeof v.model === 'string' ? v.model : '',
          createdAt: typeof v.createdAt === 'number' ? v.createdAt : 0,
        };
      }
    }
  }

  const automaticArchives: Record<string, AutomaticArchive> = {};
  for (const [id, entry] of Object.entries(obj.automaticArchives ?? {})) {
    const r = entry?.record;
    if (!r || r.conversationId !== id || typeof r.requestId !== 'string' || !Array.isArray(entry.memberIds)) continue;
    const memberIds = entry.memberIds.filter((member): member is string => typeof member === 'string');
    if (!memberIds.length) continue;
    automaticArchives[id] = {memberIds, record: {
      requestId: r.requestId, conversationId: id,
      title: typeof r.title === 'string' ? r.title : '',
      model: typeof r.model === 'string' ? r.model : '',
      createdAt: typeof r.createdAt === 'number' ? r.createdAt : 0,
      archivedAt: typeof r.archivedAt === 'number' ? r.archivedAt : Date.now(),
      projectId: null, automatic: true,
    }};
  }
  const archiveRestoredAt = Object.fromEntries(Object.entries(obj.archiveRestoredAt ?? {})
    .filter(([, at]) => typeof at === 'number' && Number.isFinite(at)));
  const autoArchiveDays = AUTO_ARCHIVE_INTERVALS.find(days => days === obj.autoArchiveDays) ?? 0;

  const activeProjectId =
    typeof obj.activeProjectId === 'string' && validProjectIds.has(obj.activeProjectId)
      ? obj.activeProjectId
      : null;

  return {
    version: 1,
    customTitles,
    pinnedOrder,
    archived,
    autoArchiveDays,
    automaticArchives,
    archiveRestoredAt,
    projects,
    itemProjects,
    conversationProjects,
    projectSnapshots,
    activeProjectId,
  };
}

function readFromStorage(): PersistedHistoryOrg {
  try {
    if (typeof localStorage !== 'undefined') {
      const text = localStorage.getItem(HISTORY_ORG_STORAGE_KEY);
      if (text) return sanitize(JSON.parse(text));
    }
  } catch {
    // Ignore storage read errors
  }
  return structuredClone(EMPTY_STATE);
}

let state = $state<PersistedHistoryOrg>(readFromStorage());
const automaticMemberIds = $derived(new Set(Object.values(state.automaticArchives).flatMap(item => item.memberIds)));

function persist() {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(HISTORY_ORG_STORAGE_KEY, JSON.stringify(state));
    }
  } catch {
    // Ignore storage quota/availability errors
  }
}

export function resetHistoryOrgForTest() {
  state = structuredClone(EMPTY_STATE);
  persist();
}

// --- Titles ---
export function getCustomTitle(requestId: string): string | null {
  return state.customTitles[requestId] ?? null;
}

export function getDisplayTitle(requestId: string, rawTitle: string): string {
  const custom = state.customTitles[requestId];
  if (custom && custom.trim()) return custom;
  return summarizeTitle(rawTitle);
}

export function renameRecord(requestId: string, nextTitle: string): void {
  const trimmed = nextTitle.trim().slice(0, 120);
  const customTitles = { ...state.customTitles };
  if (trimmed) {
    customTitles[requestId] = trimmed;
  } else {
    delete customTitles[requestId];
  }
  const archived = { ...state.archived };
  if (archived[requestId] && trimmed) {
    archived[requestId] = { ...archived[requestId], title: trimmed };
  }
  state = { ...state, customTitles, archived };
  persist();
}

// --- Pinning ---
export function isPinned(requestId: string): boolean {
  return state.pinnedOrder.includes(requestId);
}

export function pinRank(requestId: string): number {
  return state.pinnedOrder.indexOf(requestId);
}

export function togglePin(requestId: string): boolean {
  const exists = state.pinnedOrder.includes(requestId);
  const pinnedOrder = exists
    ? state.pinnedOrder.filter(id => id !== requestId)
    : [requestId, ...state.pinnedOrder];
  state = { ...state, pinnedOrder };
  persist();
  return !exists;
}

export function setPinned(ids: string[], pinned: boolean): void {
  if (!ids.length) return;
  const set = new Set(ids);
  let pinnedOrder = state.pinnedOrder.filter(id => !set.has(id));
  if (pinned) {
    pinnedOrder = [...ids, ...pinnedOrder];
  }
  state = { ...state, pinnedOrder };
  persist();
}

// --- Archiving ---
export function isArchived(requestId: string, conversationId?: string): boolean {
  return Boolean(state.archived[requestId]) || (conversationId
    ? (state.automaticArchives[conversationId]?.memberIds.includes(requestId) ?? false)
    : automaticMemberIds.has(requestId));
}

export function getArchivedList(): ArchivedRecord[] {
  const manual = Object.values(state.archived).filter(item => !state.automaticArchives[item.conversationId]);
  return [...manual, ...Object.values(state.automaticArchives).map(group => group.record)]
    .sort((a, b) => b.archivedAt - a.archivedAt);
}

export function getAutoArchiveDays(): AutoArchiveDays {
  return state.autoArchiveDays;
}

export function setAutoArchiveDays(days: AutoArchiveDays): void {
  if (state.autoArchiveDays === days || !AUTO_ARCHIVE_INTERVALS.includes(days)) return;
  state = {...state, autoArchiveDays: days};
  persist();
}

export function autoArchiveCutoff(days: AutoArchiveDays, now: number): number {
  return now - days * 86_400_000;
}

export function applyAutoArchive(
  candidates: InactiveConversation[], days: AutoArchiveDays, now: number, activeConversation: string | null,
): number {
  if (!days) return 0;
  const cutoff = autoArchiveCutoff(days, now);
  const automaticArchives = {...state.automaticArchives};
  const archiveRestoredAt = {...state.archiveRestoredAt};
  const pinned = new Set(state.pinnedOrder);
  let added = 0;
  for (const {latest, memberIds} of candidates) {
    const id = latest.conversationId;
    if (automaticArchives[id] || id === activeConversation || latest.state === 'streaming' || !memberIds.length) continue;
    if (latest.createdAt * 1000 > cutoff || (archiveRestoredAt[id] ?? 0) > cutoff) continue;
    if (state.conversationProjects[id] || memberIds.some(member => pinned.has(member) || state.itemProjects[member])) continue;
    automaticArchives[id] = {memberIds, record: {
      requestId: latest.requestId,
      conversationId: id, title: latest.title, model: latest.model,
      createdAt: latest.createdAt, archivedAt: now, projectId: null, automatic: true,
    }};
    delete archiveRestoredAt[id];
    added++;
  }
  if (added) {
    state = {...state, automaticArchives, archiveRestoredAt};
    persist();
  }
  return added;
}

export function restoreArchiveForNewTurn(reply: Pick<Summary, 'conversationId' | 'requestId' | 'createdAt'>): void {
  const archived = state.automaticArchives[reply.conversationId];
  if (archived && reply.createdAt > archived.record.createdAt && !archived.memberIds.includes(reply.requestId)) {
    restoreAutomaticArchive(reply.conversationId);
  }
}

function restoreAutomaticArchive(conversationId: string): void {
  const automaticArchives = {...state.automaticArchives};
  delete automaticArchives[conversationId];
  state = {...state, automaticArchives, archiveRestoredAt: {...state.archiveRestoredAt, [conversationId]: Date.now()}};
  persist();
}

export function archiveRecords(
  records: Pick<Summary, 'requestId' | 'conversationId' | 'title' | 'model' | 'createdAt'>[]
): number {
  if (!records.length) return 0;
  const now = Date.now();
  const archived = { ...state.archived };
  const idsToUnpin = new Set<string>();
  let added = 0;
  for (const rec of records) {
    if (!archived[rec.requestId]) added++;
    archived[rec.requestId] = {
      requestId: rec.requestId,
      conversationId: rec.conversationId,
      title: rec.title,
      model: rec.model,
      createdAt: rec.createdAt,
      archivedAt: now,
      projectId: getProjectIdFor(rec.requestId, rec.conversationId),
    };
    idsToUnpin.add(rec.requestId);
  }
  const pinnedOrder = state.pinnedOrder.filter(id => !idsToUnpin.has(id));
  state = { ...state, archived, pinnedOrder };
  persist();
  return added;
}

export function unarchiveRecord(requestId: string): void {
  const group = Object.values(state.automaticArchives).find(item => item.record.requestId === requestId);
  if (group) {
    restoreAutomaticArchive(group.record.conversationId);
    return;
  }
  if (!state.archived[requestId]) return;
  const archived = { ...state.archived };
  const conversationId = archived[requestId].conversationId;
  delete archived[requestId];
  state = { ...state, archived, archiveRestoredAt: {...state.archiveRestoredAt, [conversationId]: Date.now()} };
  persist();
}

export function unarchiveAll(): number {
  const groups = Object.keys(state.automaticArchives);
  const count = getArchivedList().length;
  if (!count) return 0;
  const archiveRestoredAt = {...state.archiveRestoredAt};
  const now = Date.now();
  for (const id of groups) archiveRestoredAt[id] = now;
  for (const item of Object.values(state.archived)) archiveRestoredAt[item.conversationId] = now;
  state = { ...state, archived: {}, automaticArchives: {}, archiveRestoredAt };
  persist();
  return count;
}

// --- Projects ---
export function getProjects(): ChatProject[] {
  return state.projects;
}

export function getProjectById(projectId: string | null | undefined): ChatProject | null {
  if (!projectId) return null;
  return state.projects.find(p => p.id === projectId) ?? null;
}

export function getActiveProjectId(): string | null {
  return state.activeProjectId;
}

export function setActiveProjectId(projectId: string | null): void {
  const valid = projectId && state.projects.some(p => p.id === projectId) ? projectId : null;
  if (state.activeProjectId === valid) return;
  state = { ...state, activeProjectId: valid };
  persist();
}

export function createProject(name: string): ChatProject | null {
  const trimmed = name.trim().slice(0, 80);
  if (!trimmed) return null;
  const id = `proj_${crypto.randomUUID().replaceAll('-', '').slice(0, 12)}`;
  const project: ChatProject = { id, name: trimmed, createdAt: Date.now() };
  state = { ...state, projects: [...state.projects, project] };
  persist();
  return project;
}

export function renameProject(projectId: string, name: string): boolean {
  const trimmed = name.trim().slice(0, 80);
  if (!trimmed) return false;
  if (!state.projects.some(p => p.id === projectId)) return false;
  const projects = state.projects.map(p => (p.id === projectId ? { ...p, name: trimmed } : p));
  state = { ...state, projects };
  persist();
  return true;
}

export function deleteProject(projectId: string): void {
  if (!state.projects.some(p => p.id === projectId)) return;
  const projects = state.projects.filter(p => p.id !== projectId);
  const itemProjects: Record<string, string> = {};
  const projectSnapshots: Record<string, ProjectItemSnapshot> = {};
  for (const [k, v] of Object.entries(state.itemProjects)) {
    if (v !== projectId) {
      itemProjects[k] = v;
      if (state.projectSnapshots[k]) projectSnapshots[k] = state.projectSnapshots[k];
    }
  }
  const conversationProjects: Record<string, string> = {};
  for (const [k, v] of Object.entries(state.conversationProjects)) {
    if (v !== projectId) conversationProjects[k] = v;
  }
  const archived: Record<string, ArchivedRecord> = {};
  for (const [k, v] of Object.entries(state.archived)) {
    archived[k] = v.projectId === projectId ? { ...v, projectId: null } : v;
  }
  const activeProjectId = state.activeProjectId === projectId ? null : state.activeProjectId;
  state = { ...state, projects, itemProjects, conversationProjects, projectSnapshots, archived, activeProjectId };
  persist();
}

export function getProjectIdFor(requestId: string, conversationId?: string): string | null {
  if (requestId in state.itemProjects) {
    return state.itemProjects[requestId] ?? null;
  }
  if (conversationId && conversationId in state.conversationProjects) {
    return state.conversationProjects[conversationId] ?? null;
  }
  return null;
}

export interface ProjectAssignInput {
  requestId: string;
  conversationId?: string;
  title?: string;
  model?: string;
  createdAt?: number;
}

export function assignToProject(items: ProjectAssignInput[], projectId: string): void {
  if (!items.length || !state.projects.some(p => p.id === projectId)) return;
  const itemProjects = { ...state.itemProjects };
  const conversationProjects = { ...state.conversationProjects };
  const projectSnapshots = { ...state.projectSnapshots };
  for (const item of items) {
    itemProjects[item.requestId] = projectId;
    if (item.conversationId) {
      conversationProjects[item.conversationId] = projectId;
    }
    const existing = projectSnapshots[item.requestId];
    projectSnapshots[item.requestId] = {
      requestId: item.requestId,
      conversationId: item.conversationId ?? existing?.conversationId ?? item.requestId,
      title: item.title ?? existing?.title ?? '',
      model: item.model ?? existing?.model ?? '',
      createdAt: item.createdAt ?? existing?.createdAt ?? Math.floor(Date.now() / 1000),
    };
  }
  state = { ...state, itemProjects, conversationProjects, projectSnapshots };
  persist();
}

export function detachFromProject(items: { requestId: string; conversationId?: string }[]): void {
  if (!items.length) return;
  const itemProjects = { ...state.itemProjects };
  const conversationProjects = { ...state.conversationProjects };
  const projectSnapshots = { ...state.projectSnapshots };
  for (const item of items) {
    delete itemProjects[item.requestId];
    delete projectSnapshots[item.requestId];
    if (item.conversationId) {
      delete conversationProjects[item.conversationId];
    }
  }
  state = { ...state, itemProjects, conversationProjects, projectSnapshots };
  persist();
}

export function bindReplyToActiveProject(
  requestId: string,
  conversationId: string,
  meta?: { title?: string; model?: string; createdAt?: number }
): void {
  const active = state.activeProjectId;
  if (!active) return;
  if (requestId in state.itemProjects || conversationId in state.conversationProjects) return;
  assignToProject([{ requestId, conversationId, ...meta }], active);
}

export function getProjectConversations(
  projectId: string,
  recentSummaries: Summary[] = []
): ProjectItemSnapshot[] {
  const byId = new Map<string, ProjectItemSnapshot>();
  for (const [reqId, pid] of Object.entries(state.itemProjects)) {
    if (pid !== projectId || isArchived(reqId)) continue;
    const snap = state.projectSnapshots[reqId];
    if (snap) byId.set(reqId, snap);
  }
  for (const item of recentSummaries) {
    if (isArchived(item.requestId, item.conversationId)) continue;
    if (getProjectIdFor(item.requestId, item.conversationId) !== projectId) continue;
    byId.set(item.requestId, {
      requestId: item.requestId,
      conversationId: item.conversationId,
      title: item.title,
      model: item.model,
      createdAt: item.createdAt,
    });
  }
  return [...byId.values()].sort((a, b) => {
    const pa = pinRank(a.requestId);
    const pb = pinRank(b.requestId);
    if (pa !== -1 && pb !== -1) return pa - pb;
    if (pa !== -1) return -1;
    if (pb !== -1) return 1;
    return b.createdAt - a.createdAt;
  });
}

export function cleanupDeletedRecords(ids: string[] | null): void {
  if (ids === null) {
    state = {
      ...state,
      customTitles: {},
      pinnedOrder: [],
      archived: {},
      automaticArchives: {},
      archiveRestoredAt: {},
      itemProjects: {},
      conversationProjects: {},
      projectSnapshots: {},
    };
    persist();
    return;
  }
  if (!ids.length) return;
  const set = new Set(ids);
  const customTitles = { ...state.customTitles };
  const archived = { ...state.archived };
  const itemProjects = { ...state.itemProjects };
  const projectSnapshots = { ...state.projectSnapshots };
  for (const id of ids) {
    delete customTitles[id];
    delete archived[id];
    delete itemProjects[id];
    delete projectSnapshots[id];
  }
  const pinnedOrder = state.pinnedOrder.filter(id => !set.has(id));
  const automaticArchives: Record<string, AutomaticArchive> = {};
  for (const [id, item] of Object.entries(state.automaticArchives)) {
    const memberIds = item.memberIds.filter(member => !set.has(member));
    if (!memberIds.length) continue;
    const record = set.has(item.record.requestId) ? {...item.record, requestId: memberIds[memberIds.length - 1]} : item.record;
    automaticArchives[id] = {memberIds, record};
  }
  state = { ...state, customTitles, archived, itemProjects, projectSnapshots, pinnedOrder, automaticArchives };
  persist();
}
