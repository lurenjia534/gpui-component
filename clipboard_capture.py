import os, sys, json, subprocess
from pathlib import Path
os.environ['DISPLAY']=':96'
os.environ['GDK_BACKEND']='x11'
os.environ.pop('WAYLAND_DISPLAY',None)
sys.path.insert(0,'/tmp/gpui-kit-3368-verification')
import native_ui as ui
import gi
gi.require_version('Gtk','3.0')
from gi.repository import Gtk, GLib, Pango
version=sys.argv[1]
sentinel=ui.Sentinel()
ui.drag((782,136),(789,143))
ui.hotkey('c')
value=ui.clipboard()
out=Path('/tmp/gpui-kit-3368-verification')
(out/f'{version}-screenshot-clipboard.json').write_text(json.dumps({'version':version,'format':'Source','scenario':'inside-formula','clipboard':value},indent=2))
window=Gtk.Window(title='System clipboard inspection')
window.set_default_size(360,190)
window.set_decorated(False)
window.move(155,280)
window.connect('destroy',Gtk.main_quit)
box=Gtk.Box(orientation=Gtk.Orientation.VERTICAL,spacing=10)
for method in ['set_margin_top','set_margin_bottom','set_margin_start','set_margin_end']: getattr(box,method)(16)
window.add(box)
def label(text,font):
    element=Gtk.Label(label=text)
    element.set_xalign(0)
    element.override_font(Pango.FontDescription(font))
    box.pack_start(element,False,False,0)
label(('Before fix' if version=='before' else 'After fix')+' | Source mode','Sans Bold 13')
label('Clipboard read after Ctrl+C','Sans 11')
body='No new clipboard content\n(sentinel unchanged)' if value=='SENTINEL_NO_COPY' else str(value)
label(body,'Monospace 13')
window.show_all()
window.move(155,280)
def capture():
    subprocess.run(['import','-display',':96','-window','root',str(out/f'{version}.png')],check=True)
    window.destroy()
    return False
GLib.timeout_add(700,capture)
Gtk.main()
sentinel.stop=True
print(json.dumps({'version':version,'clipboard':value},ensure_ascii=False))
