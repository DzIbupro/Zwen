#!/usr/bin/python3
"""Render the original logo through one rounded SVG clip for all app surfaces.
Uses Fedora's existing GdkPixbuf SVG loader; no network or image generation.
"""
import base64
from pathlib import Path
import gi
gi.require_version('GdkPixbuf','2.0')
from gi.repository import GdkPixbuf
root=Path(__file__).resolve().parents[1]
source=root/'assets/brand/zwen-logo-original.png'
original=source.read_bytes()
scaled=GdkPixbuf.Pixbuf.new_from_file_at_scale(str(source),256,256,True)
_,png=scaled.save_to_bufferv('png',[],[])
svg=f'''<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="256" height="256" viewBox="0 0 256 256"><defs><clipPath id="rounded"><rect width="256" height="256" rx="56"/></clipPath></defs><image width="256" height="256" clip-path="url(#rounded)" xlink:href="data:image/png;base64,{base64.b64encode(png).decode()}"/></svg>'''
loader=GdkPixbuf.PixbufLoader.new_with_type('svg');loader.write(svg.encode());loader.close()
rendered=loader.get_pixbuf()
for path,size in [(root/'src-tauri/icons/icon.png',256),(root/'frontend/assets/zwen-logo.png',96)]:
 pix=rendered if size==256 else rendered.scale_simple(size,size,GdkPixbuf.InterpType.HYPER)
 assert pix.get_has_alpha()
 pixels=pix.get_pixels();stride=pix.get_rowstride();channels=pix.get_n_channels()
 alpha=lambda x,y:pixels[y*stride+x*channels+3]
 assert all(alpha(x,y)==0 for x,y in [(0,0),(size-1,0),(0,size-1),(size-1,size-1)])
 assert alpha(size//2,size//2)==255
 pix.savev(str(path),'png',[],[])
 print(f'{path.relative_to(root)}: {size}px, transparent corners verified')
assert source.read_bytes()==original,'original source must remain unchanged'
