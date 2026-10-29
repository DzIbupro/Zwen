from pathlib import Path
import struct,zlib
p=Path(__file__).parent/'web'/'fixture.png';w,h=1280,720
def chunk(t,d):return struct.pack('>I',len(d))+t+d+struct.pack('>I',zlib.crc32(t+d)&0xffffffff)
rows=b''.join(b'\0'+b''.join(bytes((int(25+120*x/w),int(45+90*y/h),120)) for x in range(w)) for y in range(h))
p.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b''))

icon=Path(__file__).parent/'tauri-probe/icons/icon.png'
icon.parent.mkdir(parents=True,exist_ok=True)
rows=b''.join(b'\0'+bytes((51,122,104,255))*32 for _ in range(32))
icon.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',32,32,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b''))
