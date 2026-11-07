"""Experimental anchor math; unavailable tiled geometry must fail closed."""
import math

def resolve_anchor(window,selected,toolbar=(319,44),output=(2560,1440)):
 layout=window.get('layout',{});origin=layout.get('tile_pos_in_workspace_view')
 if not window.get('is_floating') or origin is None:return None
 offset=layout.get('window_offset_in_tile',(0,0));size=layout.get('window_size')
 values=[*origin,*offset,selected['x'],selected['y'],selected['width'],selected['height'],*toolbar,*output]
 if not all(isinstance(v,(int,float)) and math.isfinite(v) for v in values):return None
 if selected['width']<=0 or selected['height']<=0 or selected['x']<0 or selected['y']<0:return None
 if size and (selected['x']+selected['width']>size[0] or selected['y']+selected['height']>size[1]):return None
 x=origin[0]+offset[0]+selected['x']+selected['width']
 y=origin[1]+offset[1]+selected['y']-toolbar[1]-8
 return {'x':max(0,min(x,output[0]-toolbar[0])),'y':max(0,min(y,output[1]-toolbar[1]))}

if __name__=='__main__':
 import unittest
 class Anchors(unittest.TestCase):
  def test_known_floating_geometry(self):
   w={'is_floating':True,'layout':{'tile_pos_in_workspace_view':[500,343],'window_offset_in_tile':[0,0],'window_size':[1268,1381]}}
   self.assertEqual(resolve_anchor(w,{'x':143,'y':116,'width':65,'height':22}),{'x':708,'y':407})
  def test_tiled_position_cannot_be_guessed(self):
   self.assertIsNone(resolve_anchor({'is_floating':False,'layout':{'tile_pos_in_workspace_view':None}},{'x':143,'y':116,'width':65,'height':22}))
  def test_nan_is_not_a_position(self):
   self.assertIsNone(resolve_anchor({'is_floating':True,'layout':{'tile_pos_in_workspace_view':[float('nan'),0]}},{'x':1,'y':1,'width':5,'height':5}))
 unittest.main()
