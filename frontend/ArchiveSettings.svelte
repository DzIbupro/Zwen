<script lang="ts">
  import {getLocale, t} from './lib/i18n.svelte';
  import {AUTO_ARCHIVE_INTERVALS, getAutoArchiveDays, getArchivedList, getDisplayTitle, getProjectById, setAutoArchiveDays, unarchiveAll, unarchiveRecord, type AutoArchiveDays} from './lib/history-org.svelte';

  let search = $state('');
  const allArchived = $derived(getArchivedList());
  const filteredArchived = $derived.by(() => {
    const q = search.trim().toLowerCase();
    if (!q) return allArchived;
    return allArchived.filter(item => {
      const display = getDisplayTitle(item.requestId, item.title).toLowerCase();
      return display.includes(q) || item.title.toLowerCase().includes(q) || item.model.toLowerCase().includes(q);
    });
  });

  const formatArchivedDate = (ts: number) =>
    new Date(ts).toLocaleString(getLocale(), {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
</script>

<section aria-label={t('settings.archive')} class="archive-settings" data-archive-settings>
  <div class="head-row">
    <div class="title-group">
      <h1>{t('settings.archive')}</h1>
      <span class="count-pill">{t('general.archiveCount', {count: allArchived.length})}</span>
    </div>
    {#if allArchived.length > 0}
      <button type="button" class="restore-all-btn" data-unarchive-all onclick={() => unarchiveAll()}>
        {t('general.unarchiveAll')}
      </button>
    {/if}
  </div>

  <div class="group-card">
    <div class="archive-policy">
      <div>
        <label for="auto-archive-days">{t('archive.inactive')}</label>
        <p id="auto-archive-hint">{t('archive.autoHint')}</p>
      </div>
      <select id="auto-archive-days" aria-describedby="auto-archive-hint" value={getAutoArchiveDays()}
        onchange={event => setAutoArchiveDays(Number(event.currentTarget.value) as AutoArchiveDays)}>
        {#each AUTO_ARCHIVE_INTERVALS as days}
          <option value={days}>{days ? t('archive.days', {days}) : t('archive.off')}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="group-card">
    {#if allArchived.length > 0}
      <div class="search-bar">
        <input
          type="search"
          aria-label={t('history.searchPlaceholder')}
          placeholder={t('history.searchPlaceholder')}
          bind:value={search}
        />
      </div>
    {/if}

    {#if allArchived.length === 0}
      <p class="empty-state">{t('general.archiveEmpty')}</p>
    {:else if filteredArchived.length === 0}
      <p class="empty-state">{t('history.emptySearch')}</p>
    {:else}
      <div class="archived-list">
        {#each filteredArchived as item (item.requestId)}
          {@const proj = getProjectById(item.projectId)}
          {@const title = getDisplayTitle(item.requestId, item.title)}
          <div class="archived-row" data-archived-item={item.requestId}>
            <div class="archived-info">
              <strong title={title}>{title}</strong>
              <span class="archived-meta">
                {#if proj}<span class="archived-project">{proj.name}</span><span>·</span>{/if}
                {#if item.automatic}<span>{t('archive.automatic')}</span><span>·</span>{/if}
                <span>{item.model || 'Zwen'}</span>
                <span>·</span>
                <span>{formatArchivedDate(item.archivedAt)}</span>
              </span>
            </div>
            <button
              type="button"
              class="restore-btn"
              data-unarchive-item={item.requestId}
              onclick={() => unarchiveRecord(item.requestId)}
            >
              {t('general.unarchive')}
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</section>

<style>
  .archive-settings{max-width:600px;margin:20px auto;padding:0 18px 28px;color:var(--text);font-size:1rem}
  .head-row{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-bottom:16px;flex-wrap:wrap}
  .title-group{display:flex;align-items:center;gap:10px}
  h1{margin:0;font-size:1.35rem;font-weight:600}
  .count-pill{font-size:.82rem;font-weight:500;padding:2px 9px;border-radius:999px;background:var(--surface);border:1px solid var(--edge);color:var(--muted);font-variant-numeric:tabular-nums}
  .restore-all-btn{font-size:.88rem;padding:6px 12px}
  .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px}
  .group-card+.group-card{margin-top:14px}
  .archive-policy{display:flex;justify-content:space-between;align-items:center;gap:14px}
  .archive-policy label{font-size:.96rem;font-weight:500}
  .archive-policy p{margin:3px 0 0;color:var(--muted);font-size:.85rem}
  .archive-policy select{font:inherit;font-size:.92rem;flex-shrink:0;padding:6px 10px;color:var(--text);background:var(--surface-soft);border:1px solid var(--edge);border-radius:var(--radius-control)}
  .archive-policy select:focus-visible{outline:2px solid var(--focus-ring);outline-offset:1px}
  .search-bar{margin-bottom:12px}
  .search-bar input{width:100%;padding:7px 11px;border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface-soft);color:var(--text);font:inherit;font-size:.92rem}
  .search-bar input:focus-visible{outline:2px solid var(--focus-ring);outline-offset:1px;background:var(--surface)}
  .empty-state{margin:18px 0;text-align:center;font-size:.94rem;color:var(--muted)}
  .archived-list{display:flex;flex-direction:column;gap:6px}
  .archived-row{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:10px 12px;border-radius:var(--radius-control);background:var(--surface-soft);border:1px solid var(--edge);transition:border-color 140ms ease}
  @media(hover:hover) and (pointer:fine){.archived-row:hover{border-color:var(--edge-strong)}}
  .archived-info{min-width:0;display:flex;flex-direction:column;gap:4px}
  .archived-info strong{font-size:.95rem;font-weight:500;color:var(--text);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .archived-meta{display:flex;align-items:center;gap:6px;font-size:.82rem;color:var(--muted);font-variant-numeric:tabular-nums;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .archived-project{padding:1px 6px;border-radius:4px;background:var(--accent-soft);color:var(--text);font-weight:500}
  .restore-btn{font-size:.86rem;padding:5px 11px;border-radius:7px;flex-shrink:0}
  @media(max-width:520px){.archive-settings{padding-inline:0}}
</style>
