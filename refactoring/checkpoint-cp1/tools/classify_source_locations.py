#!/usr/bin/env python3
"""Diagnostic-only source-location evidence; never changes comparison acceptance."""
import argparse,collections,functools,hashlib,importlib.util,json,pathlib,re,subprocess
HERE=pathlib.Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('trace_compare',HERE/'compare_traces.py')
trace=importlib.util.module_from_spec(spec);spec.loader.exec_module(trace)

@functools.lru_cache(maxsize=4)
def raw_lines(path):return pathlib.Path(path).read_text(encoding='utf-8-sig').splitlines()
@functools.lru_cache(maxsize=512)
def parsed_record(path,line):return trace.parse_line(raw_lines(path)[line-1])
def at_path(value,path):
    if not path.startswith('$'):raise ValueError('invalid structural path')
    position=1
    for token in re.finditer(r'\.([A-Za-z_][A-Za-z_0-9]*)|\[(\d+)\]',path[1:]):
        if token.start()+1!=position:raise ValueError('unsupported structural path')
        value=value[token[1]] if token[1] else value[int(token[2])]
        position=token.end()+1
    if position!=len(path):raise ValueError('unparsed structural path')
    return value

def location(value):
    f=trace.fields(value,'Location')
    if set(f)!={'file','line','column'}:raise ValueError('unknown Location shape')
    return {'file':f['file'],'line':int(f['line'].text),'column':int(f['column'].text)}
def repo_relative(label,repo):
    path=pathlib.Path(label.replace('\\','/'))
    if path.is_absolute():
        try:path=path.resolve().relative_to(repo)
        except ValueError:raise ValueError('absolute source is outside explicit --repo; no suffix guessing')
    if '..' in path.parts:raise ValueError('source path contains parent traversal')
    if not path.parts:raise ValueError('empty source path')
    return path.as_posix()

def occurrences(lines,context):
    n=len(context)
    return [i for i in range(len(lines)-n+1) if lines[i:i+n]==context]

class Sources:
    def __init__(self,repo,commit,current_commit=None):self.repo=repo;self.commit=commit;self.current_commit=current_commit;self.cache={}
    def get(self,label,side):
        path=repo_relative(label,self.repo);key=(side,path)
        if key in self.cache:return self.cache[key]
        if side=='baseline' or self.current_commit:
            source_commit=self.commit if side=='baseline' else self.current_commit
            result=subprocess.run(['git','show',source_commit+':'+path],cwd=self.repo,capture_output=True)
            if result.returncode:raise ValueError('git show failed: '+result.stderr.decode('utf-8','replace').strip())
            data=result.stdout
        else:data=(self.repo/path).read_bytes()
        result={'path':path,'sha256':hashlib.sha256(data).hexdigest(),'lines':data.decode('utf-8-sig').splitlines()}
        self.cache[key]=result;return result
    def evidence(self,old,new):
        try:
            a=self.get(old['file'],'baseline');b=self.get(new['file'],'current')
            i=old['line']-1;j=new['line']-1
            meta={'baseline_source_path':a['path'],'current_source_path':b['path'],'baseline_source_sha256':a['sha256'],'current_source_sha256':b['sha256']}
            if not 0<=i<len(a['lines']) or not 0<=j<len(b['lines']):return {**meta,'classification':'source_line_out_of_range','baseline_line_count':len(a['lines']),'current_line_count':len(b['lines'])}
            meta.update({'baseline_line_text':a['lines'][i],'current_line_text':b['lines'][j]})
            if a['lines'][i]!=b['lines'][j]:return {**meta,'classification':'source_line_changed'}
            attempts=[]
            for radius in [1,2,3,5,8,12]:
                start=max(0,i-radius);end=min(len(a['lines']),i+radius+1);context=a['lines'][start:end]
                if len(context)<3:continue
                expected=j-(i-start)
                if expected<0 or b['lines'][expected:expected+len(context)]!=context:continue
                old_matches=occurrences(a['lines'],context);new_matches=occurrences(b['lines'],context)
                attempts.append({'radius':radius,'line_count':len(context),'baseline_occurrences':len(old_matches),'current_occurrences':len(new_matches),'claimed_mapping_matches':expected in new_matches})
                if old_matches==[start] and new_matches==[expected]:
                    classification='unique_unchanged_source_context' if old['column']==new['column'] else 'unchanged_context_column_changed'
                    return {**meta,'classification':classification,'baseline_context_start':start+1,'current_context_start':expected+1,'context_lines':context,'context_unique_in_both_files':True,'columns_equal':old['column']==new['column'],'attempts':attempts}
            single_old=occurrences(a['lines'],[a['lines'][i]]);single_new=occurrences(b['lines'],[b['lines'][j]])
            return {**meta,'classification':'unchanged_line_unresolved_context','attempts':attempts,'baseline_identical_line_count':len(single_old),'current_identical_line_count':len(single_new),'baseline_identical_line_preview':[x+1 for x in single_old[:20]],'current_identical_line_preview':[x+1 for x in single_new[:20]],'context_unique_in_both_files':False}
        except (ValueError,OSError,UnicodeError) as error:return {'classification':'source_resolution_unresolved','message':str(error)}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('comparison',type=pathlib.Path);p.add_argument('--repo',type=pathlib.Path,default=HERE.parents[2]);p.add_argument('--baseline-commit',default='b6e7ff2');p.add_argument('--current-commit');p.add_argument('--output',type=pathlib.Path);args=p.parse_args()
    repo=args.repo.resolve();out=args.output or args.comparison/'source-location-evidence';out.mkdir(parents=True,exist_ok=True)
    summary=json.loads((args.comparison/'summary.json').read_text(encoding='utf-8'));oldroot=pathlib.Path(summary['baseline']);newroot=pathlib.Path(summary['current']) if summary['current'] else None
    # Comparator paths are relative to its invocation working directory, normally the repo.
    if not oldroot.is_absolute():oldroot=repo/oldroot
    if newroot is not None and not newroot.is_absolute():newroot=repo/newroot
    groups={};unresolved=[];seen=set();leaf_count=0
    for line in (args.comparison/'details.jsonl').open(encoding='utf-8'):
        item=json.loads(line)
        if item['category']!='source_location_changed':continue
        leaf_count+=1;name=item['file'];prefix=item['path'].rsplit('.fields.',1)[0]
        occurrence=(name,item['baseline_line'],item['current_line'],prefix)
        if occurrence in seen:continue
        seen.add(occurrence)
        try:
            if newroot is None:raise ValueError('no current directory in report')
            stat=summary['file_stats'][name];oldname=stat.get('baseline_raw_name',name);newname=stat.get('current_raw_name',name)
            old=location(at_path(parsed_record(str(oldroot/oldname),item['baseline_line']),prefix));new=location(at_path(parsed_record(str(newroot/newname),item['current_line']),prefix))
            key=json.dumps([old,new],sort_keys=True)
            group=groups.setdefault(key,{'baseline':old,'current':new,'occurrence_count':0,'examples':[]})
            group['occurrence_count']+=1
            if len(group['examples'])<10:group['examples'].append({'logical_trace_file':name,'baseline_raw_file':oldname,'current_raw_file':newname,'baseline_trace_line':item['baseline_line'],'current_trace_line':item['current_line'],'location_path':prefix})
        except (ValueError,KeyError,IndexError,trace.ParseError) as error:unresolved.append({'category':'location_pair_extraction_unresolved','occurrence':occurrence,'message':str(error)})
    sources=Sources(repo,args.baseline_commit,args.current_commit);counts=collections.Counter();occurrence_counts=collections.Counter()
    with (out/'pairs.jsonl').open('w',encoding='utf-8') as stream:
        for group in groups.values():
            group['evidence']=sources.evidence(group['baseline'],group['current']);category=group['evidence']['classification'];counts[category]+=1;occurrence_counts[category]+=group['occurrence_count'];stream.write(json.dumps(group,ensure_ascii=False)+'\n')
        for item in unresolved:stream.write(json.dumps(item,ensure_ascii=False)+'\n')
    report={'purpose':'DIAGNOSTIC_ONLY_NO_COMPARISON_DIFFERENCES_ACCEPTED_OR_DROPPED','comparison_status_unchanged':summary['status'],'baseline_commit':args.baseline_commit,'current_source_commit':args.current_commit,'repo':str(repo),'source_location_leaf_differences':leaf_count,'unique_location_occurrences':len(seen),'unique_old_new_location_pairs':len(groups),'pair_classification_counts':dict(counts),'occurrence_classification_counts':dict(occurrence_counts),'extraction_unresolved':len(unresolved),'evidence_rule':'At least 3 identical contiguous source lines including the target, unique in both files, with the claimed old/new target offset; columns reported independently. No whitespace normalization, fuzzy matching, guessed paths, or generic filename renames.'}
    (out/'summary.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report,indent=2));return 0
if __name__=='__main__':main()
