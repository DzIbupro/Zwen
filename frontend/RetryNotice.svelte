<script lang="ts">
 import type {RetryState} from './lib/chat';
 let {retry}:{retry:RetryState}=$props();
 let now=$state(Date.now());
 const seconds=$derived(Math.max(0,Math.ceil((retry.nextRetryAt-now)/1000)));
 $effect(()=>{retry.nextRetryAt;now=Date.now();const timer=setInterval(()=>now=Date.now(),1000);return()=>clearInterval(timer);});
</script>
<div data-retry-status role="status">{retry.reason==='timeout'?'请求超时':'服务暂时限流'} · {seconds>0?`${seconds} 秒后重试（第${retry.attempt}次）`:`正在重新请求（第${retry.attempt}次）…`}<br/><small>问题与截图已保留，可随时停止。</small></div>
