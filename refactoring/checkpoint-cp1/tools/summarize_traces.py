#!/usr/bin/env python3
"""Print exact counts and first unresolved locations from compare_traces output."""
import argparse,collections,json,pathlib
p=argparse.ArgumentParser(description=__doc__);p.add_argument('report',type=pathlib.Path);p.add_argument('--limit',type=int,default=25);args=p.parse_args()
summary=json.loads((args.report/'summary.json').read_text(encoding='utf-8'));first={}
with (args.report/'details.jsonl').open(encoding='utf-8') as stream:
    for line in stream:
        item=json.loads(line)
        if item['category'] not in ['schema_addition','schema_rename','validated_filename_alias']:
            first.setdefault(item.get('file',''),item)
print('Status:',summary['status'])
print('Files:',summary['baseline_files'],'baseline,',summary['current_files'],'current')
print('Unresolved difference records:',summary['unresolved_difference_records'])
print('\nCategory counts:')
for category,count in sorted(summary['counts'].items()):print(f'  {category}: {count}')
rank=[]
for name,counts in summary['per_file'].items():
    unresolved=sum(n for k,n in counts.items() if k not in ['schema_addition','schema_rename','validated_filename_alias'])
    if unresolved:rank.append((unresolved,name,counts))
print('\nFiles requiring review (descending difference-record count):')
for n,name,counts in sorted(rank,reverse=True)[:args.limit]:
    item=first[name];loc=f"{item.get('baseline_line',item.get('line','-'))}:{item.get('current_line','-')}"
    print(f"  {n:7} {name}\n          first {loc} {item['category']} {item.get('path','')}\n          {json.dumps(counts,sort_keys=True)}")
print('\nFull paths, values, schema additions and exact sequence spans: details.jsonl')
