<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { DraftController, type Snapshot } from './lib/draft';
  import { draftPort, isDesktop, hideWindow, quitApp } from './lib/bridge';
  import { renderMarkdown } from './lib/render';
  import Capsule from './Capsule.svelte';
  import {getSurface,setSurface,resizeCapsule} from './lib/bridge';
  import {canApplySurface,ResizeQueue,type SurfaceInfo} from './lib/surface';
  import {DemoStream} from './lib/demo';
  let surface=$state<SurfaceInfo>({mode:'full',scaleFactor:1,generation:0,bounds:{width:480,initialHeight:96,maxHeight:594}});
  let response=$state('');let running=$state(false);let switching=$state(false);
  const demo=new DemoStream((text,active)=>{response=text;running=active;});
  function applySurface(next:SurfaceInfo){if(canApplySurface(surface.generation,next.generation)){surface=next;document.documentElement.dataset.surface=next.mode;}}
  async function switchSurface(mode:'full'|'capsule'){
    if(switching)return;switching=true;
    try{await controller.flush();applySurface(await setSurface(mode));if(mode==='full'&&response)view='reply';}
    catch(error){notice=error instanceof Error?error.message:'窗口切换未完成，内容已保留';}
    finally{switching=false;}
  }
  const resizeQueue=new ResizeQueue(resizeCapsule,()=>{notice='窗口尺寸调整未完成，可展开完整窗口';});
  function resize(height:number,generation:number){resizeQueue.request(height,generation);}

  let draft = $state<Snapshot>({text:'',revision:0,status:'loading',error:null,loaded:false});
  const controller = new DraftController(draftPort, s=>draft=s);
  let view = $state<'ask'|'preview'|'reply'>('ask');
  let notice = $state('');
  let cleared = $state<string|null>(null);
  let input = $state<HTMLTextAreaElement>();
  const example = `## 把复杂，拆成可以理解的部分\n\n正则化让模型在拟合数据与控制复杂度之间取得平衡。\n\n| 方法 | 思路 | 适用场景 |\n| --- | --- | --- |\n| L1 | 让部分参数变为零 | 特征选择 |\n| L2 | 限制参数的幅度 | 改善泛化 |\n\n行内公式 $E=mc^2$，也可以清楚地展示：\n\n$$\nL(w)=\\frac{1}{n}\\sum_{i=1}^{n}(y_i-w^Tx_i)^2+\\lambda\\lVert w\\rVert_2^2\n$$\n\n> 先理解一个概念，再把它与已有知识连起来。\n\n\`\`\`python\nloss = data_loss + strength * penalty\n\`\`\``;
  const statusText = $derived({loading:'正在读取草稿',saved:'已保存到本机',dirty:'等待保存',saving:'正在保存',error:'保存未完成'}[draft.status]);
  const preview = $derived(view==='reply' ? renderMarkdown(response) : view==='preview' ? renderMarkdown(draft.text.trim() ? draft.text : example) : '');
  function fill(text:string) { controller.insertPrompt(text);input?.focus(); }
  async function save() { try { await controller.flush(); notice='草稿已保存'; } catch { /* error is visible in draft state */ } }
  async function leave(quit=false) {
    if (!isDesktop) { notice='这是浏览器预览，草稿保存在此浏览器；桌面收起功能请在应用中使用';return; }
    try { await controller.leave(quit ? quitApp : hideWindow); } catch { /* Keep the window and current text. */ }
  }
  function keydown(e:KeyboardEvent) {
    if (e.isComposing || e.keyCode===229) return;
    if (e.key==='Escape') {e.preventDefault();void leave();}
    if ((e.ctrlKey||e.metaKey) && e.key.toLowerCase()==='s') {e.preventDefault();void save();}
  }
  onMount(()=>{
    document.documentElement.dataset.browserPreview=isDesktop?'false':'true';
    void controller.start().then(()=>input?.focus()).catch(()=>{});
    void getSurface().then(applySurface).catch(()=>{notice='窗口模式读取失败';});
    let disposed=false;const unlisteners:(()=>void)[]=[];
    if(isDesktop) void listen<SurfaceInfo>('surface-changed',e=>{applySurface(e.payload);if(e.payload.mode==='full'&&response)view='reply';}).then(fn=>{if(disposed)fn();else unlisteners.push(fn);});
    if(isDesktop) for(const [event,quit] of [['request-hide',false],['request-quit',true]] as const) {
      void listen(event,()=>void leave(quit)).then(fn=>{if(disposed)fn();else unlisteners.push(fn);}).catch(()=>{notice='关闭事件暂不可用，请使用面板按钮';});
    }
    return ()=>{disposed=true;for(const fn of unlisteners)fn();controller.dispose();demo.stop();resizeQueue.clear();};
  });
</script>

<svelte:window onkeydown={keydown}/>
{#if surface.mode==='capsule'}
<Capsule {notice} {draft} {surface} {response} {running} busy={switching} onEdit={s=>controller.edit(s)} onSave={save} onHide={()=>leave()} onExpand={()=>switchSurface('full')} onDemo={()=>demo.start()} onStop={()=>demo.stop()} onResize={resize}/>
{:else}
<div class="shell">
  <header class="titlebar">
    <div class="brand"><span class="brandmark" aria-hidden="true">s<span>·</span></span><span>Study Assistant</span><span class="edition">开发预览</span></div>
    <div class="window-actions"><button class="capsule-launch" disabled={switching} onclick={()=>switchSurface('capsule')}>快捷胶囊</button><span class="keyhint">Esc 收起</span><button class="icon-button" aria-label="收起面板" title="保存草稿并收起" onclick={()=>leave()}>−</button><details><summary aria-label="更多操作">···</summary><div class="menu"><button onclick={()=>leave(true)}>保存并退出</button></div></details></div>
  </header>
  {#if !isDesktop}<div class="preview-banner">浏览器交互预览 · 草稿仅保存在此浏览器；桌面版使用本机数据库</div>{/if}
  <div class="workspace">
    <nav aria-label="面板视图"><div class="tabs"><button class:active={view==='ask'} aria-pressed={view==='ask'} onclick={()=>view='ask'}>提问</button><button data-preview class:active={view==='preview'} aria-pressed={view==='preview'} onclick={()=>view='preview'}>格式预览</button>{#if response}<button aria-pressed={view==='reply'} class:active={view==='reply'} onclick={()=>view='reply'}>演示回复</button>{/if}</div><span class="model-status"><i></i> 模型未连接</span></nav>
    <main>
      {#if view==='ask'}
        <section class="welcome"><div class="eyebrow">A LITTLE SPACE FOR CURIOSITY</div><h1>从一个问题开始<span>。</span></h1><p>留住阅读时的疑问，慢慢把知识连起来。</p><div class="orbit" aria-hidden="true"><span></span><span></span><span></span><b><svg viewBox="0 0 32 32" aria-hidden="true"><path d="M16 4v24M4 16h24M7.5 7.5l17 17m-17 0 17-17"/></svg></b></div></section>
        <section class="suggestions" aria-label="开始一个草稿"><button disabled={!draft.loaded} onclick={()=>fill('请帮我解释这段内容：\n\n')}><span class="suggestion-icon">Aa</span><strong>读懂一段内容</strong><span>换个角度，把意思讲清楚</span><b>↗</b></button><button disabled={!draft.loaded} onclick={()=>fill('我想理解这个公式：\n\n')}><span class="suggestion-icon">ƒ</span><strong>理解一个公式</strong><span>从符号到它背后的直觉</span><b>↗</b></button><button disabled={!draft.loaded} onclick={()=>fill('帮我梳理这个想法：\n\n')}><span class="suggestion-icon"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3c0 5-4 9-9 9 5 0 9 4 9 9 0-5 4-9 9-9-5 0-9-4-9-9Z"/></svg></span><strong>梳理一个想法</strong><span>先记下来，再找到联系</span><b>↗</b></button></section>
        <div class="connection-note"><span aria-hidden="true">◌</span><p>这里会成为你的问答空间。<br/><span>当前可编辑和恢复草稿；接入模型后即可开始对话。</span></p></div>
      {:else}
        <section class="answer"><div class="answer-label"><span class="assistant-dot"></span><strong>阅读效果</strong><span>{view==='reply' ? '演示回复 · 非 AI' : draft.text.trim() ? '草稿排版预览 · 非 AI 回答' : '内置示例 · 非 AI 回答'}</span></div><div class="prose">{@html preview}</div>{#if running&&view==='reply'}<button class="text-button" onclick={()=>demo.stop()}>停止演示</button>{/if}</section>
      {/if}
    </main>
    {#if draft.error}<div class="error" role="alert"><strong>{draft.error}</strong><span>原有文件不会被自动替换。</span><button onclick={()=>draft.loaded ? save() : controller.start().catch(()=>{})}>{draft.loaded ? '重试保存' : '重新读取'}</button></div>{/if}
    <section class="composer" aria-label="问题草稿">
      <label class="sr-only" for="question">问题草稿</label><textarea bind:this={input} id="question" disabled={!draft.loaded} value={draft.text} oninput={e=>controller.edit(e.currentTarget.value)} placeholder="写下问题，或粘贴一段想理解的内容……" spellcheck="false" rows="4"></textarea>
      <div class="composer-tools"><div><span class="local-label"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m8 13 6-6a3 3 0 0 1 4 4l-8 8a5 5 0 0 1-7-7l9-9"/></svg> 本地草稿</span>{#if draft.text}<button class="text-button" onclick={()=>{cleared=draft.text;controller.edit('');}}>清空</button>{:else if cleared!==null}<button class="text-button" onclick={()=>{controller.edit(cleared!);cleared=null;}}>撤销清空</button>{/if}</div><button class="primary-button" disabled={!draft.loaded} onclick={save}><span aria-hidden="true">✓</span> 保存草稿 <kbd>Ctrl S</kbd></button></div>
    </section>
    <footer><span class:error-status={draft.status==='error'}><i class:saved={draft.status==='saved'}></i>{statusText}</span><span>{notice || '不采集当前应用内容 · 不自动上传'}</span></footer>
  </div>
</div>

{/if}
