#!/usr/bin/env python3
"""Summarize independent processes, preserving paired sampling and uncertainty."""
from collections import defaultdict
import argparse
import json
from pathlib import Path
import random
from statistics import median

parser = argparse.ArgumentParser()
parser.add_argument('--input-dir', type=Path, default=Path(__file__).resolve().parent)
args = parser.parse_args()
OUT = args.input_dir.resolve()
def records(name):
    return [json.loads(line) for line in OUT.joinpath(name+'.jsonl').read_text().splitlines()]
def quantile(values, q):
    values = sorted(values)
    at = (len(values)-1)*q
    low = int(at)
    return values[low] + (values[min(low+1,len(values)-1)]-values[low])*(at-low)
def interval(before, after):
    rng = random.Random(534)
    differences = []
    for _ in range(10000):
        indices = [rng.randrange(len(before)) for _ in before]
        differences.append(median([before[i] for i in indices])-median([after[i] for i in indices]))
    return [quantile(differences,.025),quantile(differences,.975)]
def heap(value):
    return value['heap_used']+value['mmap_bytes']

groups=defaultdict(dict)
for record in records('frames'):
    d=record['data']; groups[d['case'],d['mode']][record['variant'],record['run']]=d
frames=[]
for (case,mode), data in groups.items():
    runs=sorted({r for v,r in data})
    b=[data['baseline',r] for r in runs]; p=[data['patched',r] for r in runs]
    before=[median(d['samples_us']) for d in b]; after=[median(d['samples_us']) for d in p]
    assert len({json.dumps(d['geometry'],sort_keys=True) for d in b+p})==1
    row={'case':case,'mode':mode,'pairs':len(runs),'bytes':b[0]['source_bytes'],
         'before_us':median(before),'after_us':median(after),'saved_us':median(before)-median(after),
         'saved_percent':100*(1-median(after)/median(before)), 'saved_us_ci95':interval(before,after),
         'faster_pairs':sum(x>y for x,y in zip(before,after)),
         'before_range':[min(before),max(before)],'after_range':[min(after),max(after)],
         'before_frame_p95':median([quantile(d['samples_us'],.95) for d in b]),
         'after_frame_p95':median([quantile(d['samples_us'],.95) for d in p]),
         'first_before_us':median([d['first_us'] for d in b]),'first_after_us':median([d['first_us'] for d in p]),
         'first_saved_ci95':interval([d['first_us'] for d in b],[d['first_us'] for d in p]),
         'steady_heap_delta_bytes':median([heap(d['steady_memory'])-heap(d['before']) for d in p])-median([heap(d['steady_memory'])-heap(d['before']) for d in b]),
         'released_heap_delta_bytes':median([heap(d['released_memory'])-heap(d['before']) for d in p])-median([heap(d['released_memory'])-heap(d['before']) for d in b]),
         'steady_rss_delta_kib':median([d['steady_memory']['rss_kib']-d['before']['rss_kib'] for d in p])-median([d['steady_memory']['rss_kib']-d['before']['rss_kib'] for d in b]),
         'geometry':b[0]['geometry']}
    frames.append(row)
# The first draw is identical for all later modes of a given fixture. Pool
# these observations by fixture, retaining each (mode, run) A/B pair.
cold_groups=defaultdict(dict)
for record in records('frames'):
    d=record['data']
    cold_groups[d['case']][record['variant'],d['mode'],record['run']]=d['first_us']
cold=[]
for case, data in cold_groups.items():
    pairs=sorted({(mode,run) for variant,mode,run in data})
    before=[data['baseline',mode,run] for mode,run in pairs]
    after=[data['patched',mode,run] for mode,run in pairs]
    cold.append({'case':case,'pairs':len(pairs),'before_us':median(before),
                 'after_us':median(after),'change_percent':100*(median(after)/median(before)-1),
                 'saved_us_ci95':interval(before,after)})
groups=defaultdict(lambda:defaultdict(list))
for record in records('memory'):
    d=record['data']; groups[d['tables'],d['cols']][record['variant']].append(d)
mem=[]
for (count,cols),d in groups.items():
    row={'tables':count,'cols':cols,'exact':d['patched'][0]['exact']}
    for variant,values in d.items():
        row[variant]={
            'retained_heap_bytes':median([heap(v['measured'])-heap(v['before']) for v in values]),
            'measure_heap_bytes':median([heap(v['measured'])-heap(v['allocated']) for v in values]),
            'retained_rss_kib':median([v['measured']['rss_kib']-v['before']['rss_kib'] for v in values]),
            'repeat_heap_bytes':median([heap(v['repeated'])-heap(v['measured']) for v in values]),
            'released_heap_bytes':median([heap(v['released'])-heap(v['before']) for v in values]),
            'released_rss_kib':median([v['released']['rss_kib']-v['before']['rss_kib'] for v in values]),
        }
    row['extra_heap_bytes']=row['patched']['retained_heap_bytes']-row['baseline']['retained_heap_bytes']
    row['extra_heap_per_table']=row['extra_heap_bytes']/count
    row['extra_rss_kib']=row['patched']['retained_rss_kib']-row['baseline']['retained_rss_kib']
    row['extra_released_heap_bytes']=row['patched']['released_heap_bytes']-row['baseline']['released_heap_bytes']
    mem.append(row)
summary={'frames':frames,'cold':cold,'memory':mem,'method':'median of process medians; paired percentile bootstrap, 10000 resamples, seed 534; CI is descriptive with eight independent pairs; cold draws pooled by identical initial fixture across subsequent modes'}
OUT.joinpath('summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
for r in frames:
    print(f"{r['case']:8} {r['mode']:11} {r['before_us']:8.1f} -> {r['after_us']:8.1f} us  saved {r['saved_percent']:5.1f}%  CI {r['saved_us_ci95']}  wins {r['faster_pairs']}/{r['pairs']}")
for r in mem:
    print(f"memory {r['tables']:5} x {r['cols']:2}: extra {r['extra_heap_bytes']/1024:.2f} KiB, {r['extra_heap_per_table']:.2f} B/table, after drop {r['extra_released_heap_bytes']} B")
