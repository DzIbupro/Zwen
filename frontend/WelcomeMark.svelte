<script lang="ts">
 import mask from '../src-tauri/icons/icon.png';
</script>
<div class="welcome-mark" data-welcome-background aria-hidden="true">
 <svg viewBox="54 64 148 124" focusable="false">
  <defs>
   <!-- Threshold the original luminance: its near-black texture must not become a rectangular haze. -->
   <filter id="zwen-field-threshold" color-interpolation-filters="sRGB">
    <feComponentTransfer><feFuncR type="discrete" tableValues="0 0 0 1"/><feFuncG type="discrete" tableValues="0 0 0 1"/><feFuncB type="discrete" tableValues="0 0 0 1"/></feComponentTransfer>
   </filter>
   <mask id="zwen-field-mask" maskUnits="userSpaceOnUse" x="0" y="0" width="256" height="256" style="mask-type:luminance"><image href={mask} x="0" y="0" width="256" height="256" filter="url(#zwen-field-threshold)"/></mask>
   <pattern id="zwen-field-bars" x="0" y="0" width="1.4" height="3.6" patternUnits="userSpaceOnUse"><rect x=".35" y=".2" width=".65" height="3.1" fill="currentColor"/></pattern>
  </defs>
  <rect x="54" y="64" width="148" height="124" fill="url(#zwen-field-bars)" mask="url(#zwen-field-mask)"/>
 </svg>
</div>
<style>
 .welcome-mark{position:absolute;inset:7% 6%;pointer-events:none;user-select:none;color:var(--text);opacity:.13;z-index:0;overflow:hidden}
 svg{display:block;width:100%;height:100%}
 :global(:root[data-theme=dark]) .welcome-mark{opacity:.11}
</style>
