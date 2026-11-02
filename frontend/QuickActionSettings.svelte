<script lang="ts">
 import {onMount} from 'svelte';
 import {validateQuickActions,type QuickAction,type QuickActionConfig,type QuickActionPort} from './lib/quick-actions';
 let {port,onChanged}:{port:QuickActionPort;onChanged:(config:QuickActionConfig)=>void}=$props();
 let actions=$state<QuickAction[]>([]),directory=$state(''),revision=$state(0),loaded=$state(false),busy=$state(false),error=$state(''),notice=$state('');
 async function load(){busy=true;error='';try{const c=await port.load();actions=c.actions.map(a=>({...a}));directory=c.outputDirectory;revision=c.revision;loaded=true;onChanged(c);}catch(e){error=e instanceof Error?e.message:'读取快捷提示词失败';}finally{busy=false;}}
 async function save(){error=validateQuickActions(actions,directory)??'';if(error)return;busy=true;notice='';try{const c=await port.save(revision,actions.map(a=>({...a})),directory);revision=c.revision;onChanged(c);notice='快捷提示词已保存';}catch(e){error=e instanceof Error?e.message:'保存失败，请保留修改并重试';}finally{busy=false;}}
 function move(i:number,d:number){const copy=[...actions];[copy[i],copy[i+d]]=[copy[i+d],copy[i]];actions=copy;}
 function add(){actions=[...actions,{id:`action_${crypto.randomUUID().replaceAll('-','')}`,name:'新动作',prompt:'{{selection}}',output:'display',enabled:true}];}
 onMount(()=>{void load();});
</script>
<section data-quick-action-settings class="quick-actions" aria-label="快捷提示词设置"><h2>划词与快捷提示词</h2><p>点击动作后才发送选中文字。使用 {'{{selection}}'} 插入选区；未填写占位符时会在末尾附加选区。</p>
 {#if loaded}<form onsubmit={e=>{e.preventDefault();void save();}}><fieldset disabled={busy}>
 <label>文档输出目录<input aria-label="文档输出目录" bind:value={directory} maxlength="4096" placeholder="/home/用户/Documents/Zwen"/></label><p>保存文档时先收藏到知识库，再导出 Markdown；不会更改知识库目录。</p>
 {#each actions as action,i (action.id)}<article><div class="row"><label class="enabled"><input type="checkbox" bind:checked={action.enabled}/>启用</label><button type="button" aria-label={`上移动作 ${action.name}`} disabled={i===0} onclick={()=>move(i,-1)}>↑</button><button type="button" aria-label={`下移动作 ${action.name}`} disabled={i===actions.length-1} onclick={()=>move(i,1)}>↓</button>{#if action.id!=='translate'&&action.id!=='explain'}<button type="button" onclick={()=>actions=actions.filter(a=>a.id!==action.id)}>删除</button>{/if}</div>
 <label>名称<input bind:value={action.name} maxlength="200"/></label><label>提示词<textarea bind:value={action.prompt} rows="4" maxlength="16384"></textarea></label><label>输出方式<select bind:value={action.output}><option value="display">仅显示</option><option value="document">保存 Markdown 文档</option></select></label></article>{/each}
 <div class="row"><button type="button" disabled={actions.length>=32} onclick={add}>新增动作</button><button type="submit">保存快捷提示词</button></div></fieldset></form>{/if}
 {#if error}<p role="alert" class="error">{error}</p>{/if}{#if notice}<p role="status">{notice}</p>{/if}<button type="button" disabled={busy} onclick={()=>void load()}>重新读取（替换当前编辑）</button>
</section>
<style>.quick-actions{max-width:620px;margin:24px auto;padding:0 18px;color:#333d36}h2{font-size:22px}p{font-size:12px;line-height:1.8;color:#697663}fieldset{border:0;padding:0;margin:0}article{padding:16px;border:1px solid #d7dece;border-radius:10px;margin:14px 0}label{display:block;font-size:12px;margin:10px 0}input,textarea,select{display:block;box-sizing:border-box;width:100%;margin-top:6px;padding:9px;border:1px solid #d7dece;border-radius:6px;font:inherit;background:#fffef9;color:inherit}.row{display:flex;align-items:center;gap:8px}.enabled{display:flex;align-items:center;gap:8px;margin-right:auto}.enabled input{width:auto;margin:0}button{border:1px solid #d7dece;border-radius:6px;padding:8px 12px;background:#f4f7ec;color:#3b7157;cursor:pointer}button:disabled{opacity:.5}.error{color:#a44134}</style>
