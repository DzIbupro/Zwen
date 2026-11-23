<script lang="ts">
 import {onMount} from 'svelte';
 import {validateQuickActions,type QuickAction,type QuickActionConfig,type QuickActionPort} from './lib/quick-actions';
 let {port,onChanged}:{port:QuickActionPort;onChanged:(config:QuickActionConfig)=>void}=$props();
 let actions=$state<QuickAction[]>([]),directory=$state(''),revision=$state(0),loaded=$state(false),busy=$state(false),error=$state(''),notice=$state('');
 let expandedId=$state<string|null>(null);
 async function load(){busy=true;error='';try{const c=await port.load();actions=c.actions.map(a=>({...a}));directory=c.outputDirectory;revision=c.revision;loaded=true;onChanged(c);}catch(e){error=e instanceof Error?e.message:'读取快捷提示词失败';}finally{busy=false;}}
 async function save(){error=validateQuickActions(actions,directory)??'';if(error)return;busy=true;notice='';try{const c=await port.save(revision,actions.map(a=>({...a})),directory);revision=c.revision;onChanged(c);notice='快捷动作设置已保存';}catch(e){error=e instanceof Error?e.message:'保存失败，请保留修改并重试';}finally{busy=false;}}
 function move(i:number,d:number){const copy=[...actions];[copy[i],copy[i+d]]=[copy[i+d],copy[i]];actions=copy;}
 function add(){const id=`action_${crypto.randomUUID().replaceAll('-','')}`;actions=[...actions,{id,name:'新动作',prompt:'{{selection}}',output:'display',enabled:true}];expandedId=id;}
 onMount(()=>{void load();});
</script>

<section data-quick-action-settings class="quick-actions" aria-label="快捷动作设置">
 {#if loaded}
  <form onsubmit={e=>{e.preventDefault();void save();}}>
   <fieldset disabled={busy}>
    <div class="group-card">
     <div class="section-head">
      <div class="section-title">
       <h2>快捷动作</h2>
       <span class="count-badge">{actions.length} 项</span>
      </div>
      <button type="button" class="add-btn" disabled={actions.length>=32} onclick={add}>+ 新增动作</button>
     </div>

     <div class="action-list">
      {#each actions as action,i (action.id)}
       <details class="action-card" open={expandedId===action.id} ontoggle={e=>{if(e.currentTarget.open)expandedId=action.id;else if(expandedId===action.id)expandedId=null;}}>
        <summary class="action-summary">
         <input type="checkbox" class="action-enable" aria-label={`启用 ${action.name}`} bind:checked={action.enabled} onclick={e=>e.stopPropagation()}/>
         <span class="action-name" class:muted={!action.enabled}>{action.name||'未命名动作'}</span>
         <span class="output-badge" class:doc={action.output==='document'}>{action.output==='document'?'保存文档':'仅显示'}</span>
         <span class="summary-ops">
          <button type="button" class="op-btn" aria-label={`上移动作 ${action.name}`} title="上移" disabled={i===0} onclick={e=>{e.preventDefault();e.stopPropagation();move(i,-1);}}>↑</button>
          <button type="button" class="op-btn" aria-label={`下移动作 ${action.name}`} title="下移" disabled={i===actions.length-1} onclick={e=>{e.preventDefault();e.stopPropagation();move(i,1);}}>↓</button>
          {#if action.id!=='translate'&&action.id!=='explain'}
           <button type="button" class="op-btn danger" aria-label={`删除动作 ${action.name}`} title="删除" onclick={e=>{e.preventDefault();e.stopPropagation();actions=actions.filter(a=>a.id!==action.id);}}>删除</button>
          {/if}
         </span>
        </summary>
        <div class="action-body">
         <div class="field-row">
          <label class="field"><span>名称</span><input bind:value={action.name} maxlength="200"/></label>
          <label class="field"><span>输出方式</span><select bind:value={action.output}><option value="display">仅显示</option><option value="document">保存 Markdown 文档</option></select></label>
         </div>
         <label class="field"><span>提示词模板</span><textarea bind:value={action.prompt} rows="4" maxlength="16384" placeholder="使用 {'{{selection}}'} 插入选区内容"></textarea></label>
        </div>
       </details>
      {/each}
     </div>
    </div>

    <div class="group-card">
     <h2>文档导出</h2>
     <label class="field">
      <span>文档输出目录</span>
      <input aria-label="文档输出目录" bind:value={directory} maxlength="4096" placeholder="/home/用户/Documents/Zwen"/>
     </label>
     <details class="hint-disclosure">
      <summary>提示词与输出说明</summary>
      <div class="hint-disclosure-body">
       <p>点击动作后才发送选中文字。使用 <code>{'{{selection}}'}</code> 插入选区；未填写占位符时会在末尾附加选区。</p>
       <p>保存文档时先收藏到知识库，再导出 Markdown；不会更改知识库目录。</p>
      </div>
     </details>
    </div>

    {#if error}<p role="alert" class="error-msg">{error}</p>{/if}
    {#if notice}<p role="status" class="status-msg">{notice}</p>{/if}

    <div class="footer-actions">
     <button type="submit" class="primary-save">保存快捷动作</button>
     <button type="button" disabled={busy} onclick={()=>void load()}>重置为已保存</button>
    </div>
   </fieldset>
  </form>
 {:else if error}
  <p role="alert" class="error-msg">{error}</p>
  <button type="button" disabled={busy} onclick={()=>void load()}>重新读取</button>
 {/if}
</section>

<style>
 .quick-actions{color:var(--text);font-size:14px;line-height:1.55}
 fieldset{border:0;padding:0;margin:0}
 .group-card{background:var(--surface);border:1px solid var(--edge);border-radius:var(--radius-card);padding:14px 16px;margin-bottom:14px}
 .section-head{display:flex;align-items:center;justify-content:space-between;gap:10px;margin-bottom:12px}
 .section-title{display:flex;align-items:center;gap:8px}
 h2{margin:0;font-size:12.5px;font-weight:600;color:var(--muted);letter-spacing:.02em}
 .count-badge{font-size:11.5px;font-weight:500;color:var(--muted);background:var(--surface-soft);border:1px solid var(--edge);border-radius:999px;padding:1px 8px;font-variant-numeric:tabular-nums}
 .add-btn{font-size:12.5px;font-weight:500;padding:5px 10px}
 .action-list{display:flex;flex-direction:column;gap:8px}
 .action-card{border:1px solid var(--edge);border-radius:var(--radius-control);background:var(--surface-soft);overflow:hidden;transition:border-color 140ms ease}
 .action-card[open]{border-color:var(--edge-strong);background:var(--surface)}
 .action-summary{display:flex;align-items:center;gap:9px;padding:9px 11px;cursor:pointer;user-select:none;list-style:none}
 .action-summary::after{content:'▾';font-size:12px;color:var(--muted);margin-left:2px;transition:transform 160ms var(--ease-out)}
 .action-card[open]>.action-summary::after{transform:rotate(180deg)}
 .action-enable{width:15px;height:15px;margin:0;accent-color:var(--accent);flex-shrink:0;cursor:pointer}
 .action-name{font-size:13.5px;font-weight:600;color:var(--text);min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
 .action-name.muted{color:var(--muted);font-weight:500;text-decoration:line-through}
 .output-badge{font-size:11px;font-weight:500;padding:2px 7px;border-radius:999px;background:var(--surface);border:1px solid var(--edge);color:var(--muted);flex-shrink:0}
 .output-badge.doc{background:var(--accent-soft);border-color:transparent;color:var(--text)}
 .summary-ops{margin-left:auto;display:inline-flex;align-items:center;gap:4px;flex-shrink:0}
 .op-btn{padding:2px 7px;font-size:12px;min-height:24px;border-radius:6px;background:var(--surface)}
 .op-btn.danger{color:var(--danger)}
 .action-body{padding:12px;border-top:1px solid var(--edge);display:flex;flex-direction:column;gap:10px}
 .field-row{display:grid;grid-template-columns:minmax(0,1fr) 170px;gap:10px}
 .field{display:flex;flex-direction:column;gap:5px;margin:0}
 .group-card>h2+.field{margin-top:10px}
 .field>span{font-size:12.5px;font-weight:500;color:var(--muted)}
 input:not([type="checkbox"]),textarea,select{display:block;box-sizing:border-box;width:100%;padding:7px 10px;border:1px solid var(--edge);border-radius:var(--radius-control);font:inherit;font-size:13.5px;line-height:1.55;background:var(--surface);color:var(--text)}
 textarea{resize:vertical;min-height:84px;font-family:var(--font-mono);font-size:12.5px}
 .footer-actions{display:flex;align-items:center;gap:10px;flex-wrap:wrap;margin-top:4px}
 .primary-save{background:var(--accent);color:var(--on-accent);border-color:var(--accent);font-weight:500;padding:7px 14px}
 @media(hover:hover) and (pointer:fine){.primary-save:hover:not(:disabled){background:var(--accent);opacity:.9;color:var(--on-accent)}}
 .error-msg{color:var(--danger);font-size:13px;margin:0 0 10px}
 .status-msg{color:var(--muted);font-size:13px;margin:0 0 10px}
 @media(max-width:560px){.field-row{grid-template-columns:1fr}}
</style>
