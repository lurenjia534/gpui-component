import sys, json, time, subprocess
from pathlib import Path
sys.path.insert(0,'/tmp/gpui-kit-3368-verification')
import native_ui as ui
version=sys.argv[1]
out=Path('/tmp/gpui-kit-3368-verification')
scenarios=[
    ('intro-to-formula',(573,107),(785,139),'Intro{sep}{math}'),
    ('formula-to-intro',(785,139),(573,107),'Intro{sep}{math}'),
    ('formula-to-after',(785,139),(612,175),'{math}{sep}After'),
    ('after-to-formula',(612,175),(785,139),'{math}{sep}After'),
    ('inside-formula',(782,136),(789,143),'{math}'),
    ('inside-formula-reverse',(789,143),(782,136),'{math}'),
    ('across-formula',(573,107),(612,175),'Intro{sep}{math}{sep}After'),
    ('across-formula-reverse',(612,175),(573,107),'Intro{sep}{math}{sep}After'),
    ('intro-only',(573,107),(608,107),'Intro'),
]
results=[]
for fmt in ['Plain','Source']:
    if fmt=='Source':
        ui.click(874,726)
        time.sleep(.5)
    math='x' if fmt=='Plain' else '$$\nx\n$$'
    sep='\n' if fmt=='Plain' else '\n\n'
    for name,start,end,expected in scenarios:
        sentinel=ui.Sentinel()
        ui.drag(start,end)
        ui.hotkey('c')
        got=ui.clipboard()
        want=expected.format(math=math,sep=sep)
        result={'version':version,'format':fmt,'scenario':name,'expected':want,'clipboard':got,'passes':got==want}
        results.append(result)
        print(json.dumps(result,ensure_ascii=False),flush=True)
        if name in ['intro-to-formula','inside-formula','across-formula']:
            subprocess.run(['import','-display',':96','-window','root',str(out/f'{version}-{name}-{fmt.lower()}.png')],check=True)
        sentinel.stop=True
    sentinel=ui.Sentinel()
    ui.click(785,139)
    ui.hotkey('c')
    got=ui.clipboard()
    result={'version':version,'format':fmt,'scenario':'click-only','expected':'SENTINEL_NO_COPY','clipboard':got,'passes':got=='SENTINEL_NO_COPY'}
    results.append(result)
    print(json.dumps(result),flush=True)
    sentinel.stop=True
(out/f'{version}-native-results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2))
