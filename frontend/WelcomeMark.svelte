<script lang="ts">
 import mask from '../src-tauri/icons/icon.png';
</script>
<div class="welcome-mark">
 <svg viewBox="0 0 320 300" aria-hidden="true" focusable="false">
  <defs>
   <mask id="zwen-field-mask" maskUnits="userSpaceOnUse" x="10" y="0" width="300" height="300" style="mask-type:luminance"><image href={mask} x="10" y="0" width="300" height="300"/></mask>
   <pattern id="zwen-field-bars" x="0" y="0" width="4" height="9" patternUnits="userSpaceOnUse"><rect x="1" y=".5" width="1.8" height="8" fill="currentColor"/></pattern>
  </defs>
  <g class="field-grid">{#each Array.from({length:69},(_,i)=>i) as i}<path d={`M ${24+i*4} 28 V 272`}/>{/each}</g>
  <rect class="field-symbol" x="10" y="0" width="300" height="300" fill="url(#zwen-field-bars)" mask="url(#zwen-field-mask)"/>
 </svg>
 <p>今天想问 Zwen 什么问题？</p>
</div>
<style>
 .welcome-mark{display:flex;flex-direction:column;align-items:center;gap:22px;max-width:100%;color:var(--text)}svg{display:block;width:min(280px,80%);height:auto;max-height:min(28vh,260px);overflow:visible;flex-shrink:1}.field-grid{stroke:currentColor;stroke-width:.5;opacity:.09}.field-symbol{opacity:.64}p{margin:0;font-size:22px;font-weight:500;line-height:1.5;letter-spacing:.02em;overflow-wrap:anywhere}@media(max-width:640px){.welcome-mark{gap:16px}p{font-size:18px}}
</style>
