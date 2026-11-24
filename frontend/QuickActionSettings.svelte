<script lang="ts">
 import {onMount} from 'svelte';
 import {localizePresetQuickActions,validateQuickActions,type QuickAction,type QuickActionConfig,type QuickActionPort} from './lib/quick-actions';
 import {getLocale,t} from './lib/i18n.svelte';
 let {port,onChanged}:{port:QuickActionPort;onChanged:(config:QuickActionConfig)=>void}=$props();
 let actions=$state<QuickAction[]>([]),directory=$state(''),revision=$state(0),loaded=$state(false),busy=$state(false),error=$state(''),notice=$state('');
 let expandedId=$state<string|null>(null);
 async function load(){
  busy=true;error='';
  try{
   const c=await port.load();
   const loc=localizePresetQuickActions(c.actions,getLocale());
   actions=loc.actions.map(a=>({...a}));
   directory=c.outputDirectory;
   revision=c.revision;
   loaded=true;
   onChanged({...c,actions:loc.actions});
  }catch(e){error=e instanceof Error?e.message:'读取快捷提示词失败';}
  finally{busy=false;}
 }
 $effect(()=>{
  const loc=getLocale();
  if(!loaded||busy)return;
  const res=localizePresetQuickActions(actions,loc);
  if(res.changed)actions=res.actions;
 });
 async function save(){error=validateQuickActions(actions,directory)??'';if(error)return;busy=true;notice='';try{const c=await port.save(revision,actions.map(a=>({...a})),directory);revision=c.revision;onChanged(c);notice=t('quick.saved');}catch(e){error=e instanceof Error?e.message:'保存失败，请保留修改并重试';}finally{busy=false;}}
 function move(i:number,d:number){const copy=[...actions];[copy[i],copy[i+d]]=[copy[i+d],copy[i]];actions=copy;}
 function add(){const id=`action_${crypto.randomUUID().replaceAll('-','')}`;actions=[...actions,{id,name:t('quick.newName'),prompt:'{{selection}}',output:'display',enabled:true}];expandedId=id;}
 onMount(()=>{void load();});
</script>

<section data-quick-action-settings class="quick-actions" aria-label={t('quick.aria')}>
 {#if loaded}
  <form onsubmit={e=>{e.preventDefault();void save();}}>
   <fieldset disabled={busy}>
    <div class="group-card">
     <div class="section-head">
      <div class="section-title">
       <h2>{t('quick.heading')}</h2>
       <span class="count-badge">{t('quick.count', {count: actions.length})}</span>
      </div>
      <button type="button" class="add-btn" disabled={actions.length>=32} onclick={add}>{t('quick.add')}</button>
     </div>

     <div class="action-list">
      {#each actions as action,i (action.id)}
       <details class="action-card" open={expandedId===action.id} ontoggle={e=>{if(e.currentTarget.open)expandedId=action.id;else if(expandedId===action.id)expandedId=null;}}>
        <summary class="action-summary">
         <input type="checkbox" class="action-enable" aria-label={`启用 ${action.name}`} bind:checked={action.enabled} onclick={e=>e.stopPropagation()}/>
         <span class="action-name" class:muted={!action.enabled}>{action.name||t('quick.unnamed')}</span>
         <span class="output-badge" class:doc={action.output==='document'}>{action.output==='document'?t('quick.outputDoc'):t('quick.outputDisplay')}</span>
         <span class="summary-ops">
          <button type="button" class="op-btn" aria-label={`${t('quick.moveUp')} ${action.name}`} title={t('quick.moveUp')} disabled={i===0} onclick={e=>{e.preventDefault();e.stopPropagation();move(i,-1);}}>↑</button>
          <button type="button" class="op-btn" aria-label={`${t('quick.moveDown')} ${action.name}`} title={t('quick.moveDown')} disabled={i===actions.length-1} onclick={e=>{e.preventDefault();e.stopPropagation();move(i,1);}}>↓</button>
          {#if action.id!=='translate'&&action.id!=='explain'}
           <button type="button" class="op-btn danger" aria-label={`${t('quick.delete')} ${action.name}`} title={t('quick.delete')} onclick={e=>{e.preventDefault();e.stopPropagation();actions=actions.filter(a=>a.id!==action.id);}}>{t('quick.delete')}</button>
          {/if}
         </span>
        </summary>
        <div class="action-body">
         <div class="field-row">
          <label class="field"><span>{t('quick.name')}</span><input bind:value={action.name} maxlength="200"/></label>
          <label class="field"><span>{t('quick.outputMode')}</span><select bind:value={action.output}><option value="display">{t('quick.outputDisplay')}</option><option value="document">{t('quick.outputDocLong')}</option></select></label>
         </div>
         <label class="field"><span>{t('quick.promptTemplate')}</span><textarea bind:value={action.prompt} rows="4" maxlength="16384" placeholder="{'{{selection}}'}"></textarea></label>
        </div>
       </details>
      {/each}
     </div>
    </div>

    <div class="group-card">
     <h2>{t('quick.exportGroup')}</h2>
     <label class="field">
      <span>{t('quick.outputDir')}</span>
      <input aria-label={t('quick.outputDir')} bind:value={directory} maxlength="4096" placeholder="/home/user/Documents/Zwen"/>
     </label>
     <details class="hint-disclosure">
      <summary>{t('quick.helpSummary')}</summary>
      <div class="hint-disclosure-body">
       <p>{t('quick.help1')}</p>
       <p>{t('quick.help2')}</p>
      </div>
     </details>
    </div>

    {#if error}<p role="alert" class="error-msg">{error}</p>{/if}
    {#if notice}<p role="status" class="status-msg">{notice}</p>{/if}

    <div class="footer-actions">
     <button type="submit" class="primary-save">{t('quick.save')}</button>
     <button type="button" disabled={busy} onclick={()=>void load()}>{t('quick.reset')}</button>
    </div>
   </fieldset>
  </form>
 {:else if error}
  <p role="alert" class="error-msg">{error}</p>
  <button type="button" disabled={busy} onclick={()=>void load()}>{t('general.reload')}</button>
 {/if}
</section>

<style>
 .quick-actions{color:var(--text);font-size:1rem;line-height:1.55}
 fieldset{border:0;padding:0;margin:0}
 .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px}
 .section-head{display:flex;align-items:center;justify-content:space-between;gap:10px;margin-bottom:12px}
 .section-title{display:flex;align-items:center;gap:8px}
 h2{margin:0;font-size:.86rem;font-weight:600;color:var(--muted);letter-spacing:.02em}
 .count-badge{font-size:.8rem;font-weight:500;color:var(--muted);background:var(--surface-soft);border:1px solid var(--edge);border-radius:999px;padding:1px 8px;font-variant-numeric:tabular-nums}
 .add-btn{font-size:.86rem;font-weight:500;padding:5px 10px}
 .action-list{display:flex;flex-direction:column;gap:8px}
 .action-card{border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface-soft);overflow:hidden;transition:border-color 140ms ease}
 .action-card[open]{border-color:var(--edge-strong);background:var(--surface)}
 .action-summary{display:flex;align-items:center;gap:9px;padding:9px 11px;cursor:pointer;user-select:none;list-style:none}
 .action-summary::after{content:'▾';font-size:.82rem;color:var(--muted);margin-left:2px;transition:transform 160ms var(--ease-out)}
 .action-card[open]>.action-summary::after{transform:rotate(180deg)}
 .action-enable{width:15px;height:15px;margin:0;accent-color:var(--accent);flex-shrink:0;cursor:pointer}
 .action-name{font-size:.96rem;font-weight:600;color:var(--text);min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
 .action-name.muted{color:var(--muted);font-weight:500;text-decoration:line-through}
 .output-badge{font-size:.78rem;font-weight:500;padding:2px 7px;border-radius:999px;background:var(--surface);border:1px solid var(--edge);color:var(--muted);flex-shrink:0}
 .output-badge.doc{background:var(--accent-soft);border-color:transparent;color:var(--text)}
 .summary-ops{margin-left:auto;display:inline-flex;align-items:center;gap:4px;flex-shrink:0}
 .op-btn{padding:2px 7px;font-size:.82rem;min-height:24px;border-radius:6px;background:var(--surface)}
 .op-btn.danger{color:var(--danger)}
 .action-body{padding:12px;border-top:1px solid var(--edge);display:flex;flex-direction:column;gap:10px}
 .field-row{display:grid;grid-template-columns:minmax(0,1fr) 170px;gap:10px}
 .field{display:flex;flex-direction:column;gap:5px;margin:0}
 .group-card>h2+.field{margin-top:10px}
 .field>span{font-size:.86rem;font-weight:500;color:var(--muted)}
 input:not([type="checkbox"]),textarea,select{display:block;box-sizing:border-box;width:100%;padding:7px 10px;border:1px solid var(--edge);border-radius:var(--radius-control);font:inherit;font-size:.95rem;line-height:1.55;background:var(--surface);color:var(--text)}
 textarea{resize:vertical;min-height:84px;font-family:var(--font-mono);font-size:.88rem}
 .footer-actions{display:flex;align-items:center;gap:10px;flex-wrap:wrap;margin-top:4px}
 .primary-save{background:var(--accent);color:var(--on-accent);border-color:var(--accent);font-weight:500;padding:7px 14px}
 @media(hover:hover) and (pointer:fine){.primary-save:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}
 .error-msg{color:var(--danger);font-size:13px;margin:0 0 10px}
 .status-msg{color:var(--muted);font-size:13px;margin:0 0 10px}
 @media(max-width:560px){.field-row{grid-template-columns:1fr}}
</style>
