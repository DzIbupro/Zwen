"""Recreate pinned third-party assets from npm registry tarballs."""
from pathlib import Path
import urllib.request,json,tarfile,io,hashlib,base64
root=Path(__file__).parent/'web'
for meta in json.loads((root/'vendor-lock.json').read_text()):
 with urllib.request.urlopen(meta['source'],timeout=60) as f:data=f.read()
 algorithm,expected=meta['integrity'].split('-',1)
 assert base64.b64encode(hashlib.new(algorithm,data).digest()).decode()==expected
 destination=root/'vendor'/meta['package'];destination.mkdir(parents=True,exist_ok=True)
 with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as tar:
  for member in tar.getmembers():
   parts=Path(member.name)
   if not member.isfile() or parts.is_absolute() or '..' in parts.parts:continue
   rel=Path(*parts.parts[1:])
   if rel.parts and (rel.parts[0] in ['dist','lib'] or rel.name.lower().startswith('license')):
    path=destination/rel;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(tar.extractfile(member).read())
 print(meta['package'],meta['version'],'integrity verified')
