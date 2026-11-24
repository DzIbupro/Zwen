import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount, unmount, flushSync, tick } from 'svelte';
vi.mock('svelte', () => vi.importActual<typeof import('svelte')>('../../node_modules/svelte/src/index-client.js'));
import HistoryPanel from '../HistoryPanel.svelte';
import ArchiveSettings from '../ArchiveSettings.svelte';
import Sidebar from '../Sidebar.svelte';
import App from '../App.svelte';
import { summarizeTitle } from '../lib/title-summary';
import {
  archiveRecords,
  assignToProject,
  bindReplyToActiveProject,
  createProject,
  deleteProject,
  detachFromProject,
  getArchivedList,
  getDisplayTitle,
  getProjectIdFor,
  getProjects,
  isArchived,
  isPinned,
  renameRecord,
  resetHistoryOrgForTest,
  setActiveProjectId,
  togglePin,
  unarchiveRecord,
} from '../lib/history-org.svelte';
import type { ChatReply } from '../lib/chat';
import type { Summary } from '../lib/history';

const settle = async () => {
  for (let i = 0; i < 8; i++) {
    await tick();
    await Promise.resolve();
    flushSync();
  }
};

describe('title summarization & history organization', () => {
  beforeEach(() => {
    localStorage.clear();
    resetHistoryOrgForTest();
  });

  it('summarizes prompts cleanly without calling a model and supports custom renaming', () => {
    expect(summarizeTitle('请问一下，Rust 里的 Arc 和 Mutex 有什么区别？顺便举个例子。')).toBe(
      'Rust 里的 Arc 和 Mutex 有什么区别'
    );
    expect(summarizeTitle('给我讲一下高斯混合模型')).toBe('高斯混合模型');
    expect(
      summarizeTitle('请将以下选中文字翻译成中文，保留原意：\n\nLazarus is a cross-platform IDE.')
    ).toBe('翻译：Lazarus is a cross-platform IDE');
    expect(
      summarizeTitle('请详细解释以下选中文字：\n\n结果可以看成一张“时间 × Mel 频带”的能量图。')
    ).toBe('解释：结果可以看成一张“时间 × Mel 频带”的能量图');
    expect(
      summarizeTitle('论文阅读 · 02_Odyssey2026_WhySayItLikeThat_Phoneme_Level_Explainable.pdf\n\n请按阅读要求分析这篇论文。')
    ).toBe('论文：Odyssey2026 Why Say It Like That Phoneme Level…');
    expect(summarizeTitle('Please explain how Wayland layer-shell works in detail.')).toBe(
      'explain how Wayland layer-shell works in detail'
    );
    expect(summarizeTitle('[文件: main.rs]\n```rust\nfn main() {}\n```')).toBe('文件：main.rs');

    expect(getDisplayTitle('r1', '请帮我总结 Transformer 的注意力机制。')).toBe(
      '总结 Transformer 的注意力机制'
    );
    renameRecord('r1', 'Transformer 自注意力笔记');
    expect(getDisplayTitle('r1', '请帮我总结 Transformer 的注意力机制。')).toBe('Transformer 自注意力笔记');
  });

  it('supports pinning, archiving, unarchiving, and project assignment/detachment', () => {
    const p = createProject('毕业论文')!;
    expect(getProjects().map(x => x.name)).toEqual(['毕业论文']);

    setActiveProjectId(p.id);
    bindReplyToActiveProject('req-1', 'conv-1');
    expect(getProjectIdFor('req-1', 'conv-1')).toBe(p.id);

    detachFromProject([{ requestId: 'req-1', conversationId: 'conv-1' }]);
    expect(getProjectIdFor('req-1', 'conv-1')).toBeNull();

    assignToProject([{ requestId: 'req-1', conversationId: 'conv-1' }], p.id);
    expect(getProjectIdFor('req-1', 'conv-1')).toBe(p.id);

    togglePin('req-1');
    expect(isPinned('req-1')).toBe(true);

    archiveRecords([{ requestId: 'req-1', conversationId: 'conv-1', title: '测试标题', model: 'm', createdAt: 100 }]);
    expect(isArchived('req-1')).toBe(true);
    expect(isPinned('req-1')).toBe(false);
    expect(getArchivedList()).toHaveLength(1);

    unarchiveRecord('req-1');
    expect(isArchived('req-1')).toBe(false);
    expect(getArchivedList()).toHaveLength(0);

    // Deleting a project turns its chats back into unassigned chats
    deleteProject(p.id);
    expect(getProjects()).toHaveLength(0);
    expect(getProjectIdFor('req-1', 'conv-1')).toBeNull();
  });

  it('enforces edit mode for selection and supports batch pin, project assign/detach, archive, and restore in Settings', async () => {
    const items: Summary[] = [
      { requestId: 'a', conversationId: 'a', title: '请问如何配置 Linux 音频服务？', model: 'm1', state: 'complete', createdAt: 10 },
      { requestId: 'b', conversationId: 'b', title: '帮我对比 SQLite 和 DuckDB 的适用场景。', model: 'm1', state: 'complete', createdAt: 20 },
    ];
    const replies: Record<string, ChatReply> = {
      a: { requestId: 'a', conversationId: 'a', contextTurns: 0, revision: 1, prompt: items[0].title, answer: '回答 A', model: 'm1', state: 'complete', error: null, saved: true, createdAt: 10 },
      b: { requestId: 'b', conversationId: 'b', contextTurns: 0, revision: 1, prompt: items[1].title, answer: '回答 B', model: 'm1', state: 'complete', error: null, saved: true, createdAt: 20 },
    };
    const p = createProject('系统学习')!;

    const target = document.createElement('div');
    document.body.append(target);
    const app = mount(HistoryPanel, {
      target,
      props: {
        port: { list: async () => items, get: async id => replies[id] ?? null },
        canContinue: true,
        canDelete: true,
        onContinue: () => {},
      },
    });

    try {
      await settle();

      // 1. Default mode: NO checkboxes visible
      expect(target.querySelectorAll('input[type="checkbox"]').length).toBe(0);

      // 2. Clicking a row in normal mode opens its detail & allows inline renaming
      const firstRow = target.querySelector<HTMLButtonElement>('.row-card')!;
      firstRow.click();
      await settle();
      expect(target.querySelector('.detail-title')?.textContent).toBe('如何配置 Linux 音频服务');

      target.querySelector<HTMLButtonElement>('[data-history-rename]')!.click();
      flushSync();
      const renameInput = target.querySelector<HTMLInputElement>('[data-rename-form] input')!;
      renameInput.value = 'Linux PipeWire 配置指南';
      renameInput.dispatchEvent(new Event('input', { bubbles: true }));
      target.querySelector<HTMLFormElement>('[data-rename-form]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
      flushSync();
      expect(target.querySelector('.detail-title')?.textContent).toBe('Linux PipeWire 配置指南');

      // 3. Enter Edit Mode -> checkboxes appear
      target.querySelector<HTMLButtonElement>('[data-history-edit]')!.click();
      flushSync();
      expect(target.querySelectorAll('input[type="checkbox"]').length).toBe(2);

      // 4. Select 'b' and pin it -> 'b' floats to top
      target.querySelector<HTMLInputElement>('[data-history-check="b"]')!.click();
      flushSync();
      target.querySelector<HTMLButtonElement>('[data-history-bulk-pin]')!.click();
      flushSync();
      const titlesAfterPin = [...target.querySelectorAll('.row-card strong')].map(el => el.textContent);
      expect(titlesAfterPin[0]).toBe('对比 SQLite 和 DuckDB 的适用场景');
      expect(target.querySelector('[data-pin-badge]')).not.toBeNull();

      // 5. Assign 'a' to project '系统学习' via custom menu (no '移入项目…' placeholder inside menu), then detach it
      target.querySelector<HTMLInputElement>('[data-history-check="a"]')!.click();
      flushSync();
      const projMenuBtn = target.querySelector<HTMLButtonElement>('[data-history-bulk-project]')!;
      projMenuBtn.click();
      flushSync();
      const menuText = target.querySelector('.bulk-project-menu')?.textContent ?? '';
      expect(menuText).not.toContain('移入项目');
      target.querySelector<HTMLButtonElement>(`[data-bulk-project-item="${p.id}"]`)!.click();
      flushSync();
      expect(getProjectIdFor('a', 'a')).toBe(p.id);

      target.querySelector<HTMLInputElement>('[data-history-check="a"]')!.click();
      flushSync();
      target.querySelector<HTMLButtonElement>('[data-history-bulk-detach]')!.click();
      flushSync();
      expect(getProjectIdFor('a', 'a')).toBeNull();

      // 6. Archive 'a' via Edit Mode -> hidden from History list
      target.querySelector<HTMLInputElement>('[data-history-check="a"]')!.click();
      flushSync();
      target.querySelector<HTMLButtonElement>('[data-history-bulk-archive]')!.click();
      flushSync();
      expect(target.querySelectorAll('.record-row').length).toBe(1);

      // 7. Exit Edit Mode -> checkboxes disappear again
      target.querySelector<HTMLButtonElement>('[data-history-edit]')!.click();
      flushSync();
      expect(target.querySelectorAll('input[type="checkbox"]').length).toBe(0);

      // 8. Verify archived record appears in dedicated ArchiveSettings and can be restored
      const settingsTarget = document.createElement('div');
      document.body.append(settingsTarget);
      const settingsApp = mount(ArchiveSettings, {
        target: settingsTarget,
      });
      try {
        flushSync();
        expect(settingsTarget.querySelector('[data-archived-item="a"]')?.textContent).toContain('Linux PipeWire 配置指南');
        settingsTarget.querySelector<HTMLButtonElement>('[data-unarchive-item="a"]')!.click();
        flushSync();
        expect(settingsTarget.querySelector('[data-archived-item="a"]')).toBeNull();
        expect(target.querySelectorAll('.record-row').length).toBe(2);
      } finally {
        await unmount(settingsApp);
        settingsTarget.remove();
      }
    } finally {
      await unmount(app);
      target.remove();
    }
  });

  it('shows only project conversations in the sidebar tree and keeps chat home titlebar clean', async () => {
    vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });
    const proj = createProject('Zwen 项目')!;
    const nowSec = Math.floor(Date.now() / 1000) - 600;
    const recentSummaries: Summary[] = [
      { requestId: 'p1', conversationId: 'p1', title: '项目内对话测试', model: 'm1', state: 'complete', createdAt: nowSec },
      { requestId: 's1', conversationId: 's1', title: '零散对话不应出现在侧边栏', model: 'm1', state: 'complete', createdAt: nowSec },
    ];
    assignToProject([{ requestId: 'p1', conversationId: 'p1', title: '项目内对话测试', model: 'm1', createdAt: nowSec }], proj.id);

    const opened: string[] = [];
    const sbTarget = document.createElement('div');
    document.body.append(sbTarget);
    const sb = mount(Sidebar, {
      target: sbTarget,
      props: {
        collapsed: false,
        active: 'chat',
        recentSummaries,
        onToggle: () => {},
        onNavigate: () => {},
        onNew: () => {},
        onNewInProject: () => {},
        onOpenConversation: id => opened.push(id),
      },
    });
    try {
      flushSync();
      expect(sbTarget.querySelector(`[data-sidebar-project="${proj.id}"]`)?.textContent).toContain('Zwen 项目');
      expect(sbTarget.querySelector('[data-sidebar-chat="p1"]')?.textContent).toContain('项目内对话测试');
      expect(sbTarget.querySelector('[data-sidebar-chat="s1"]')).toBeNull();
      expect(sbTarget.textContent).not.toContain('零散对话不应出现在侧边栏');

      sbTarget.querySelector<HTMLButtonElement>('[data-sidebar-chat="p1"]')!.click();
      expect(opened).toEqual(['p1']);
    } finally {
      await unmount(sb);
      sbTarget.remove();
    }

    // Verify blank App home titlebar does not show "当前对话" or project dropdown noise
    const appTarget = document.createElement('div');
    document.body.append(appTarget);
    const appInst = mount(App, { target: appTarget });
    try {
      await settle();
      const titlebarText = appTarget.querySelector('.titlebar')?.textContent ?? '';
      expect(titlebarText).not.toContain('当前对话');
      expect(titlebarText).not.toContain('零散对话');
      expect(titlebarText).not.toContain('无项目');
      expect(appTarget.querySelector('[data-chat-project-select]')).toBeNull();
    } finally {
      await unmount(appInst);
      appTarget.remove();
      vi.unstubAllGlobals();
    }
  });
});
