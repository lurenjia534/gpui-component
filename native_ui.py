import os, sys, time, select, json, threading
sys.path.insert(0, '/tmp/gpui-kit-3368-verification/python')
from Xlib import X, XK, display, protocol
from Xlib.ext import xtest

D = display.Display(':96')
ROOT = D.screen().root

def windows():
    return [(w.id, w.get_wm_name(), w.get_geometry()._data) for w in ROOT.query_tree().children]

def key(name, down):
    code = D.keysym_to_keycode(XK.string_to_keysym(name))
    xtest.fake_input(D, X.KeyPress if down else X.KeyRelease, code)
    D.sync()

def hotkey(name):
    key('Control_L', True)
    key(name, True)
    key(name, False)
    key('Control_L', False)
    time.sleep(0.3)

def move(x, y):
    xtest.fake_input(D, X.MotionNotify, x=int(x), y=int(y))
    D.sync()

def click(x, y):
    move(x,y)
    xtest.fake_input(D, X.ButtonPress, 1)
    xtest.fake_input(D, X.ButtonRelease, 1)
    D.sync()
    time.sleep(0.3)

def type_text(text):
    for ch in text:
        sym = XK.string_to_keysym('Return') if ch == '\n' else ord(ch)
        code = D.keysym_to_keycode(sym)
        mapping = D.get_keyboard_mapping(code,1)[0]
        shift = mapping[0] != sym
        if shift: key('Shift_L',True)
        xtest.fake_input(D,X.KeyPress,code)
        xtest.fake_input(D,X.KeyRelease,code)
        if shift: key('Shift_L',False)
        D.sync()
        time.sleep(0.04)
    time.sleep(1)

def drag(start, end):
    move(*start)
    xtest.fake_input(D,X.ButtonPress,1)
    D.sync()
    time.sleep(0.2)
    for i in range(1,21):
        move(start[0]+(end[0]-start[0])*i/20, start[1]+(end[1]-start[1])*i/20)
        time.sleep(0.025)
    xtest.fake_input(D,X.ButtonRelease,1)
    D.sync()
    time.sleep(0.5)

def clipboard(timeout=4):
    d=display.Display(':96')
    w=d.screen().root.create_window(0,0,1,1,0,X.CopyFromParent)
    clip=d.intern_atom('CLIPBOARD')
    utf8=d.intern_atom('UTF8_STRING')
    prop=d.intern_atom('GPUI_3368_RESULT')
    w.convert_selection(clip,utf8,prop,X.CurrentTime)
    d.flush()
    until=time.monotonic()+timeout
    while time.monotonic()<until:
        if d.pending_events():
            e=d.next_event()
            if e.type == X.SelectionNotify:
                if not e.property:
                    return None
                val=w.get_full_property(prop,X.AnyPropertyType)
                return val.value.decode('utf-8') if val else None
        else: select.select([d.fileno()],[],[],0.05)
    return '<timeout>'

class Sentinel:
    def __init__(self):
        self.d=display.Display(':96')
        self.w=self.d.screen().root.create_window(0,0,1,1,0,X.CopyFromParent)
        self.clip=self.d.intern_atom('CLIPBOARD')
        self.utf8=self.d.intern_atom('UTF8_STRING')
        self.targets=self.d.intern_atom('TARGETS')
        self.w.set_selection_owner(self.clip,X.CurrentTime)
        self.d.sync()
        self.stop=False
        threading.Thread(target=self.serve,daemon=True).start()
    def serve(self):
        while not self.stop:
            if self.d.pending_events():
                e=self.d.next_event()
                if e.type==X.SelectionRequest:
                    prop=e.property or e.target
                    if e.target==self.targets:
                        e.requestor.change_property(prop,Xatom_ATOM,32,[self.targets,self.utf8,XA_STRING])
                    else:
                        e.requestor.change_property(prop,e.target,8,b'SENTINEL_NO_COPY')
                    e.requestor.send_event(protocol.event.SelectionNotify(time=e.time,requestor=e.requestor,selection=e.selection,target=e.target,property=prop),propagate=False)
                    self.d.flush()
            else: select.select([self.d.fileno()],[],[],0.05)
Xatom_ATOM=4
XA_STRING=31

if __name__=='__main__':
    if sys.argv[1]=='windows': print(json.dumps(windows(),indent=2,default=str))
    elif sys.argv[1]=='source':
        w=ROOT.query_tree().children[-1]
        w.set_input_focus(X.RevertToParent,X.CurrentTime)
        D.sync()
        click(260,112)
        hotkey('a')
        type_text('Intro\n\n$$\nx\n$$\n\nAfter')
    elif sys.argv[1]=='drag':
        s=Sentinel()
        drag(tuple(map(float,sys.argv[2:4])),tuple(map(float,sys.argv[4:6])))
        hotkey('c')
        print(json.dumps({'clipboard':clipboard()},ensure_ascii=False))
    elif sys.argv[1]=='clipboard': print(repr(clipboard()))
    elif sys.argv[1]=='click': click(float(sys.argv[2]),float(sys.argv[3]))
