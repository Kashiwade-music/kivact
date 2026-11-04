#!/usr/bin/env python3
"""Strict CP0/CP1 trace audit. Only explicit schema mappings are projected."""
import argparse, bisect, collections, decimal, difflib, hashlib, json, pathlib, re, struct, subprocess, sys
D=decimal.Decimal
class ParseError(ValueError): pass
class Num:
    def __init__(self,text): self.text=text
    def __eq__(self,other): return isinstance(other,Num) and self.text==other.text
    def __repr__(self): return self.text

def tag(name,**fields): return {'tag':name,'fields':fields}
def atom(name): return {'tag':name}
def is_tag(value,name): return isinstance(value,dict) and value.get('tag')==name
def fields(value,name=None):
    if not isinstance(value,dict) or 'fields' not in value or (name and value.get('tag')!=name):
        raise ParseError('expected struct '+str(name)+', got '+repr(value)[:150])
    return value['fields']
def some(value): return {'tag':'Some','args':[value]}
def unoption(value):
    if is_tag(value,'None'): return None
    if is_tag(value,'Some') and len(value.get('args',[]))==1: return value['args'][0]
    raise ParseError('expected Option, got '+repr(value)[:150])

class Parser:
    def __init__(self,text): self.s=text; self.p=0
    def ws(self):
        while self.p<len(self.s) and self.s[self.p].isspace(): self.p+=1
    def take(self,t):
        self.ws()
        if self.s.startswith(t,self.p): self.p+=len(t); return True
        return False
    def need(self,t):
        if not self.take(t): raise ParseError('expected '+t+' at '+str(self.p)+': '+self.s[self.p:self.p+70])
    def ident(self):
        self.ws();m=re.match(r'[A-Za-z_][A-Za-z_0-9]*(?:::[A-Za-z_][A-Za-z_0-9]*)*',self.s[self.p:])
        if not m: raise ParseError('identifier at '+str(self.p))
        self.p+=len(m[0]);return m[0]
    def quoted(self,q):
        start=self.p;self.p+=1;escaped=False
        while self.p<len(self.s):
            c=self.s[self.p];self.p+=1
            if escaped: escaped=False
            elif c=='\\': escaped=True
            elif c==q:
                raw=self.s[start+1:self.p-1]
                raw=re.sub(r'\\u\{([0-9a-fA-F]+)\}',lambda m:json.dumps(chr(int(m[1],16)))[1:-1],raw)
                raw=raw.replace(r'\0',r'\u0000')
                if q=="'": raw=raw.replace('\\\'','\'').replace('"','\\"')
                try:return json.loads('"'+raw+'"')
                except ValueError as e:raise ParseError('Rust string escape: '+str(e))
        raise ParseError('unterminated quote')
    def seq(self,end):
        out=[]
        if self.take(end):return out
        while True:
            out.append(self.value())
            if self.take(end):return out
            self.need(',')
            if self.take(end):return out
    def value(self):
        self.ws()
        if self.p>=len(self.s):raise ParseError('missing value')
        c=self.s[self.p]
        if c in ['"',"'"]:v=self.quoted(c)
        elif self.take('['):v=self.seq(']')
        elif self.take('('):v={'tuple':self.seq(')')}
        else:
            m=re.match(r'-?(?:\d+\.\d+(?:[eE][+-]?\d+)?|\d+[eE][+-]?\d+|\d+)(?![A-Za-z_])',self.s[self.p:])
            if m:self.p+=len(m[0]);v=Num(m[0])
            else:
                name=self.ident()
                if name in ['true','false']:v=name=='true'
                elif self.take('{'):
                    f={}
                    if not self.take('}'):
                        while True:
                            key=self.ident();self.need(':')
                            if key in f:raise ParseError('duplicate struct field '+key)
                            f[key]=self.value()
                            if self.take('}'):break
                            self.need(',')
                            if self.take('}'):break
                    v={'tag':name,'fields':f}
                elif self.take('('):v={'tag':name,'args':self.seq(')')}
                else:v=atom(name)
        if self.take('..='):v={'range_inclusive':[v,self.value()]}
        elif self.take('..'):v={'range':[v,self.value()]}
        return v
    def all(self):
        value=self.value();self.ws()
        if self.p!=len(self.s):raise ParseError('unparsed suffix at '+str(self.p)+': '+self.s[self.p:self.p+90])
        return value

def parse_line(text):
    if text.startswith('semantics '):
        m=re.fullmatch(r'semantics (n\d+) (.*)',text)
        if not m:raise ParseError('semantics envelope')
        return {'kind':'semantics','node':m[1],'value':atom('removed') if m[2]=='removed' else Parser(m[2]).all()}
    if text.startswith('host_events '):
        rest=text[len('host_events '):];direction,_,body=rest.partition(' ')
        if direction=='commit':
            m=re.fullmatch(r'revision=(\d+) writes=(\d+)',body)
            if not m:raise ParseError('commit envelope')
            return {'kind':'commit','revision':Num(m[1]),'writes':Num(m[2])}
        if direction not in ['start','receive','output']:raise ParseError('unknown host direction '+direction)
        return {'kind':direction,'value':Parser(body).all()}
    if text.startswith('stage '):
        rest=text[6:]
        if ' events=' not in rest:return {'kind':'stage','name':rest}
        p=Parser(rest);turn=p.value();values={}
        while p.p<len(p.s):
            p.ws()
            if p.p==len(p.s):break
            key=p.ident();p.need('=')
            if key in values:raise ParseError('duplicate summary field')
            values[key]=p.value()
        required={'events','causes','writes','reruns','runs','ops','notes'}
        if set(values)!=required:raise ParseError('summary field schema '+repr(set(values)))
        return {'kind':'summary','turn':turn,**values}
    raise ParseError('unknown line prefix')

SEM_ORDINARY=['role','name','description','value_text','selected','checked','expanded','invalid','actions','test_id']
SEM_ADVANCED={'value_min':atom('None'),'value_max':atom('None'),'value_now':atom('None'),'live_region':atom('None'),'sort':atom('None'),'orientation':atom('Unspecified'),'active_descendant':atom('None'),'described_by':[],'controls':[],'labelled_by':[],'level':atom('None'),'set_size':atom('None'),'pos_in_set':atom('None'),'multiselectable':atom('None'),'busy':atom('None'),'required':atom('None'),'read_only':atom('None'),'has_popup':atom('None'),'current_item':atom('None'),'text':atom('None')}
ADDED_SEM={'disabled','hidden','focusable','protected'}

class Adapter:
    def __init__(self,file,side,emit):self.file=file;self.side=side;self.emit=emit;self.line=0;self.nested_semantics=False
    def note(self,category,path,value):self.emit({'category':category,'file':self.file,'side':self.side,'line':self.line,'path':path,'value':value})
    def normalize(self,value,path='$'):
        if isinstance(value,list):return [self.normalize(v,path+'['+str(i)+']') for i,v in enumerate(value)]
        if not isinstance(value,dict):return value
        name=value.get('tag');f=value.get('fields');args=value.get('args')
        if name=='Finite' and args and len(args)==1:return self.normalize(args[0],path)
        if name=='Failure' and f is not None:
            if set(f)!={'kind','detail'}:raise ParseError('unknown Failure fields')
            self.note('schema_addition',path+'.kind',f['kind'])
            return self.normalize(f['detail'],path+'.detail')
        if name=='SemanticsTextRun' and f is not None:
            f=dict(f)
            if is_tag(f.get('id'),'TextRunId'):
                f['id']={**f['id'],'tag':'NodeId'}
                self.note('schema_rename',path+'.id','TextRunId -> baseline NodeId')
            return {'tag':name,'fields':{k:self.normalize(v,path+'.'+k) for k,v in f.items()}}
        if name=='HostInfo' and f is not None and 'surface_active' in f:
            f=dict(f);surface=f.pop('surface_active')
            env=self.normalize(f.pop('env'),path+'.env')
            fields(env,'HostEnvironment')['surface_active']=surface
            return {'tag':'HostInfo','fields':{**{k:self.normalize(v,path+'.'+k) for k,v in f.items()},'env':env}}
        if name=='EnvChanged' and f is not None:
            if set(f)!={'environment','surface_active'}:raise ParseError('unknown EnvChanged fields')
            env=self.normalize(f['environment'],path+'.environment')
            fields(env,'HostEnvironment')['surface_active']=f['surface_active']
            return {'tag':'EnvChanged','args':[env]}
        if name in ['HostEnv','Environment'] and f is not None:
            f=dict(f)
            if name=='Environment':
                f['scale']=f.pop('scale_factor');f['window']=f.pop('window_size')
                for k in ['keyboard_inset','pointer','fonts']:
                    if k in f:self.note('schema_addition',path+'.'+k,f.pop(k))
            return {'tag':'HostEnvironment','fields':{k:self.normalize(v,path+'.'+k) for k,v in f.items()}}
        if name in ['Pointer','Key','Wheel','File'] and args and len(args)==1 and is_tag(args[0],'Routed'):
            r=dict(fields(args[0],'Routed'));input_value=r.pop('input');data=dict(fields(input_value))
            expected={'target','region','local','captured','activation'}
            if set(r)!=expected:raise ParseError('unknown Routed metadata '+repr(set(r)))
            target=r.pop('target');region=r.pop('region');local=r.pop('local')
            for k in ['captured','activation']:self.note('schema_addition',path+'.routed.'+k,r[k])
            if not is_tag(r['activation'],'None'):self.note('schema_addition_inconsistent',path+'.routed.activation',r['activation'])
            if name=='Pointer':
                node=unoption(target)
                if node is None:raise ParseError('Pointer missing route target')
                phase=data.get('phase');expected_press=phase.get('args',[None])[0] if isinstance(phase,dict) and phase.get('tag') in ['Down','Up'] else None
                expected_option=atom('None') if expected_press is None else some(expected_press)
                expected_button=atom('None') if expected_press is None else some(fields(expected_press,'ButtonPress')['button'])
                if data.get('press')!=expected_option:self.note('schema_addition_inconsistent',path+'.input.press',data.get('press'))
                if data.get('button')!=expected_button:self.note('schema_addition_inconsistent',path+'.input.button',data.get('button'))
                for k in ['time_ns','button','press']:
                    if k in data:self.note('schema_addition',path+'.input.'+k,data.pop(k))
                data['pos']=local;data['window_pos']=data.pop('position')
                v=tag('Pointer',node=node,region=region,event={'tag':'PointerEvent','fields':data})
            elif name=='Key':
                self.note('schema_addition',path+'.routed.local',local)
                v=tag('Key',node=target,region=region,**data)
            elif name=='Wheel':
                node=unoption(target)
                if node is None:raise ParseError('Wheel missing route target')
                self.note('schema_addition',path+'.input.position',data.pop('position'))
                v=tag('Wheel',node=node,region=region,pos=local,**data)
            else:
                node=unoption(target)
                if node is None:raise ParseError('File missing route target')
                self.note('schema_addition',path+'.routed.region',region)
                v=tag('File',node=node,pos=local,window_pos=data.pop('position'),**data)
            return self.normalize(v,path)
        # Preserve all unrecognized tags/fields recursively, never drop them.
        return {k:({fk:self.normalize(fv,path+'.'+fk) for fk,fv in v.items()} if k=='fields' else self.normalize(v,path+'.'+k)) for k,v in value.items()}
    def semantic(self,value):
        if is_tag(value,'removed'):return value
        inner=unoption(value)
        if self.nested_semantics and is_tag(value,'None'):
            self.note('schema_addition','$.semantics.presence',{'engine_node_present':False,'semantic_present':False,'baseline_None_node_presence':'not_observable'})
        if is_tag(inner,'Some') or is_tag(inner,'None'):
            self.note('schema_addition','$.semantics.presence',{'engine_node_present':True,'semantic_present':is_tag(inner,'Some'),'baseline_None_node_presence':'not_observable'})
            inner=unoption(inner)
        if inner is None:
            return {'tag':'SemanticProjection','fields':{**{k:atom('None') for k in SEM_ORDINARY},**SEM_ADVANCED}}
        name=inner.get('tag');f=dict(fields(inner))
        if name=='NodeSemantics':
            f['value_text']=f.pop('value');extra=unoption(f.pop('accessibility'))
            adv=dict(SEM_ADVANCED)
            if extra is not None:
                if not is_tag(extra,'SemanticsProperties'):raise ParseError('unknown accessibility container')
                adv.update(fields(extra))
            result={**f,**adv}
        elif name=='Semantics':
            for k in ADDED_SEM:
                if k in f:self.note('schema_addition','$.semantics.'+k,f.pop(k))
            result=f
        else:raise ParseError('unknown semantics payload '+str(name))
        expected=set(SEM_ORDINARY)|set(SEM_ADVANCED)
        if set(result)!=expected:raise ParseError('unknown/missing semantics fields '+repr(set(result)^expected))
        return {'tag':'SemanticProjection','fields':{k:self.normalize(v,'$.semantics.'+k) for k,v in result.items()}}
    def record(self,record):
        record=dict(record)
        if record['kind']=='semantics':record['value']=self.semantic(record['value'])
        elif 'value' in record:record['value']=self.normalize(record['value'])
        # Summary values still include exact locations, types, counts, and ordering.
        else:record=self.normalize(record)
        return record

def zero(v):return isinstance(v,Num) and D(v.text)==0
def empty_turn(records,prior_revision):
    if prior_revision is None:return False
    summaries=[r for _,r in records if r['kind']=='summary']
    outputs=[r for _,r in records if r['kind']=='output']
    if len(summaries)!=1 or len(outputs)!=1:return False
    summary=summaries[0]
    if not is_tag(summary['turn'],'Number'):return False
    if any(not zero(summary[k]) for k in ['events','runs','ops']):return False
    if any(summary[k]!=[] for k in ['causes','writes','reruns','notes']):return False
    if fields(outputs[0]['value'],'TurnOutput')!={'next_wake_ns':atom('None'),'exit':False}:return False
    for _,r in records:
        if r['kind']=='commit' and (not zero(r['writes']) or r['revision']!=prior_revision):return False
        if r['kind'] not in ['stage','summary','output','commit']:return False
    return True

def load(path,side,emit):
    parsed=[];errors=[]
    for line,text in enumerate(path.read_text(encoding='utf-8-sig').splitlines(),1):
        try:parsed.append((line,parse_line(text)))
        except (ParseError,ValueError,KeyError) as error:
            errors.append(line);emit({'category':'parse_error','file':path.name,'side':side,'line':line,'message':str(error),'raw':text})
    # A parse failure cannot be masked by an empty-turn decision.
    if errors:return None,{'parse_errors':len(errors)}
    chunks=[];before=[]
    for item in parsed:
        if item[1]=={'kind':'stage','name':'Begin'}:
            if before:chunks.append(before)
            before=[item]
        else:before.append(item)
    if before:chunks.append(before)
    output=[];removed=[];offset=0;prior_revision=None
    for chunk in chunks:
        can_remove=empty_turn(chunk,prior_revision)
        for _,entry in chunk:
            if entry['kind']=='commit':prior_revision=entry['revision']
        if can_remove:
            removed.append({'begin_line':chunk[0][0],'end_line':chunk[-1][0],'turn':next(r['turn'] for _,r in chunk if r['kind']=='summary')});offset+=1;continue
        for line,r in chunk:
            if r['kind']=='summary' and is_tag(r['turn'],'Number'):
                number=r['turn']['args'][0]
                r={**r,'turn':{'tag':'Number','args':[Num(str(int(number.text)-offset))]}}
            output.append((line,r))
    adapter=Adapter(path.name,side,emit);normalized=[]
    adapter.nested_semantics=any(r['kind']=='semantics' and is_tag(r['value'],'Some') and (is_tag(r['value']['args'][0],'Some') or is_tag(r['value']['args'][0],'None')) for _,r in parsed)
    for line,r in output:
        adapter.line=line
        try:normalized.append((line,adapter.record(r)))
        except (ParseError,KeyError,ValueError) as error:
            emit({'category':'adapter_error','file':path.name,'side':side,'line':line,'message':str(error),'record':r});errors.append(line)
    return (None if errors else normalized),{'raw_lines':len(parsed),'kept_lines':len(output),'empty_turns':removed,'parse_errors':len(errors),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}

def f32bits(text):return struct.pack('!f',float(text))
def diffs(a,b,path='$'):
    if re.search(r'\.(at|created_at)\.fields\.(file|line|column)$',path) and a!=b:
        yield 'source_location_changed',path,a,b;return
    if isinstance(a,Num) and isinstance(b,Num):
        if a.text==b.text:return
        if D(a.text)==D(b.text):
            category='numeric_signed_zero_changed' if D(a.text)==0 and D(a.text).is_signed()!=D(b.text).is_signed() else 'numeric_spelling'
            yield category,path,a,b;return
        if any(c in a.text+b.text for c in '.eE'):
            try:same=f32bits(a.text)==f32bits(b.text)
            except (OverflowError,ValueError):same=False
            yield ('numeric_same_f32_projection' if same else 'numeric_value_changed'),path,a,b
        else:yield 'integer_changed',path,a,b
    elif type(a)!=type(b):yield 'type_changed',path,a,b
    elif isinstance(a,dict):
        for k in a:
            if k not in b:yield 'field_removed',path+'.'+k,a[k],None
            else:yield from diffs(a[k],b[k],path+'.'+k)
        for k in b:
            if k not in a:yield 'field_added',path+'.'+k,None,b[k]
    elif isinstance(a,list):
        for i,(x,y) in enumerate(zip(a,b)):yield from diffs(x,y,path+'['+str(i)+']')
        if len(a)!=len(b):yield 'sequence_length',path,len(a),len(b)
    elif a!=b:
        category='diagnostic_type_name_changed' if path.endswith('.fields.ty') else ('diagnostic_cause_label_changed' if path.startswith('$.causes[') else 'value_changed')
        yield category,path,a,b

def identity_cause(text):
    # Alignment identity only; payload strings remain unchanged and compared.
    text=re.sub(r'Some\(NodeId\((\d+)\)\)',r'n\1',text)
    text=re.sub(r'Some\(RegionId\((\d+)\)\)',r'Some(\1)',text)
    return text

def fingerprint(value):return json.dumps(value,sort_keys=True,default=json_default,separators=(',',':'))
def discriminator(r):
    k=r['kind']
    if k=='semantics':return k+':'+r['node']
    if k=='stage':return k+':'+r['name']
    if k=='summary':return k+':'+fingerprint([identity_cause(c) for c in r['causes']])
    if k=='receive':
        value=r['value'];name=value.get('tag');f=value.get('fields',{})
        stable={x:f[x] for x in ['node','region'] if x in f}
        if name=='Key':stable.update({x:f[x] for x in ['key','code'] if x in f});stable['phase']=f.get('phase',{}).get('tag')
        if name=='Pointer':
            event=f.get('event',{}).get('fields',{});stable.update({x:event[x] for x in ['id','kind'] if x in event});stable['phase']=event.get('phase',{}).get('tag')
        if name=='File':stable['phase']=f.get('phase')
        if name=='FrameTick':stable['now_ns']=f.get('now_ns')
        return k+':'+str(name)+':'+fingerprint(stable)
    return k

def json_default(value):
    if isinstance(value,Num):return {'number':value.text}
    raise TypeError(type(value).__name__)

def aligned_opcodes(a,b):
    """Ordered patience anchors, then bounded LCS; all payloads still compared."""
    exact_a=[fingerprint(r) for r in a];exact_b=[fingerprint(r) for r in b]
    stable_a=[discriminator(r) for r in a];stable_b=[discriminator(r) for r in b]
    result=[]
    def anchors(keys_a,keys_b,lo,hi,start,end):
        left=collections.defaultdict(list);right=collections.defaultdict(list)
        for i in range(lo,hi):left[keys_a[i]].append(i)
        for j in range(start,end):right[keys_b[j]].append(j)
        pairs=sorted((v[0],right[key][0]) for key,v in left.items() if len(v)==1 and len(right.get(key,[]))==1)
        tails=[];indices=[];previous=[]
        for n,(_,j) in enumerate(pairs):
            at=bisect.bisect_left(tails,j);previous.append(indices[at-1] if at else -1)
            if at==len(tails):tails.append(j);indices.append(n)
            else:tails[at]=j;indices[at]=n
        if not indices:return []
        ordered=[];n=indices[-1]
        while n>=0:ordered.append(pairs[n]);n=previous[n]
        return ordered[::-1]
    def visit(lo,hi,start,end,depth=0):
        prefix=0
        while lo+prefix<hi and start+prefix<end and exact_a[lo+prefix]==exact_b[start+prefix]:prefix+=1
        if prefix:result.append(('equal',lo,lo+prefix,start,start+prefix));lo+=prefix;start+=prefix
        suffix=0
        while hi-suffix>lo and end-suffix>start and exact_a[hi-suffix-1]==exact_b[end-suffix-1]:suffix+=1
        x=hi-suffix;y=end-suffix
        if lo<x or start<y:
            if lo==x:result.append(('insert',lo,x,start,y))
            elif start==y:result.append(('delete',lo,x,start,y))
            else:
                selected=anchors(exact_a,exact_b,lo,x,start,y)
                if not selected:selected=anchors(stable_a,stable_b,lo,x,start,y)
                if selected and depth<64:
                    i=lo;j=start
                    for ai,bj in selected:
                        visit(i,ai,j,bj,depth+1);result.append(('equal',ai,ai+1,bj,bj+1));i=ai+1;j=bj+1
                    visit(i,x,j,y,depth+1)
                elif max(x-lo,y-start)<=4000:
                    result.extend((op,i+lo,j+lo,k+start,l+start) for op,i,j,k,l in difflib.SequenceMatcher(None,stable_a[lo:x],stable_b[start:y],autojunk=False).get_opcodes())
                else:result.append(('alignment_unresolved',lo,x,start,y))
        if suffix:result.append(('equal',x,hi,y,end))
    visit(0,len(a),0,len(b))
    return result

# Exactly six validated test artifact aliases; no generic f32/f64 filename rule.
ALIAS_SOURCE='crates/kivact-core/src/test/screen/window_tests.rs'
ALIAS_OLD_FN='resize_dimensions_must_fit_finite_positive_f32'
ALIAS_NEW_FN='resize_dimensions_must_fit_finite_positive_f64'
ALIAS_OLD_PREFIX='test__screen__window_tests__resize_dimensions_must_fit_finite_positive_f32'
ALIAS_NEW_PREFIX='test__screen__window_tests__resize_dimensions_must_fit_finite_positive_f64'
FILENAME_ALIASES={ALIAS_OLD_PREFIX+suffix+'.txt':ALIAS_NEW_PREFIX+suffix+'.txt' for suffix in ['', '-2', '-3', '-4', '-5', '-6']}
ACCEPTED_NOTES={'schema_addition','schema_rename','validated_filename_alias'}

def alias_source_evidence():
    repo=pathlib.Path(__file__).resolve().parents[3]
    proc=subprocess.run(['git','show','b6e7ff2:'+ALIAS_SOURCE],cwd=repo,capture_output=True)
    if proc.returncode:raise ParseError('cannot validate alias baseline source: '+proc.stderr.decode('utf-8','replace'))
    old=proc.stdout;new=(repo/ALIAS_SOURCE).read_bytes()
    old_lines=old.decode('utf-8').splitlines();new_lines=new.decode('utf-8').splitlines()
    old_matches=[i+1 for i,line in enumerate(old_lines) if re.match(r'fn '+ALIAS_OLD_FN+r'\(\)',line)]
    new_matches=[i+1 for i,line in enumerate(new_lines) if re.match(r'fn '+ALIAS_NEW_FN+r'\(\)',line)]
    if len(old_matches)!=1 or len(new_matches)!=1:raise ParseError('filename alias source function is absent or ambiguous')
    return {'baseline_commit':'b6e7ff2','source_path':ALIAS_SOURCE,'baseline_function':ALIAS_OLD_FN,'current_function':ALIAS_NEW_FN,'baseline_source_line':old_matches[0],'current_source_line':new_matches[0],'baseline_source_sha256':hashlib.sha256(old).hexdigest(),'current_source_sha256':hashlib.sha256(new).hexdigest()}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--baseline',type=pathlib.Path,required=True);p.add_argument('--current',type=pathlib.Path);p.add_argument('--output',type=pathlib.Path,required=True);p.add_argument('--audit-only',action='store_true');args=p.parse_args()
    if not args.audit_only and args.current is None:p.error('--current required unless --audit-only')
    args.output.mkdir(parents=True,exist_ok=True);counts=collections.Counter();per_file={};file_stats={};differences=0
    with (args.output/'details.jsonl').open('w',encoding='utf-8') as detail:
        def emit(record):
            nonlocal differences
            category=record['category'];counts[category]+=1;per_file.setdefault(record.get('file',''),collections.Counter())[category]+=1
            if category not in ACCEPTED_NOTES:differences+=1
            detail.write(json.dumps(record,ensure_ascii=False,default=json_default)+'\n')
        left={f.name:f for f in args.baseline.glob('*.txt')};right={} if args.current is None else {f.name:f for f in args.current.glob('*.txt')}
        alias_ledger=[];source_evidence=None
        if not args.audit_only:
            for old_name,new_name in FILENAME_ALIASES.items():
                if old_name not in left or new_name not in right:continue
                if old_name in right:
                    emit({'category':'filename_alias_conflict','file':old_name,'baseline_raw_name':old_name,'current_raw_names':[old_name,new_name]});continue
                try:
                    if source_evidence is None:source_evidence=alias_source_evidence()
                except (OSError,ParseError) as error:
                    emit({'category':'filename_alias_validation_failed','file':old_name,'message':str(error)});continue
                current_path=right.pop(new_name);right[old_name]=current_path
                entry={'category':'validated_filename_alias','file':old_name,'baseline_raw_name':old_name,'current_raw_name':new_name,'baseline_sha256':hashlib.sha256(left[old_name].read_bytes()).hexdigest(),'current_sha256':hashlib.sha256(current_path.read_bytes()).hexdigest(),'source_evidence':source_evidence}
                alias_ledger.append(entry);emit(entry)
        for name in sorted(set(left)|set(right)):
            if args.audit_only:
                _,stats=load(left[name],'baseline',emit);file_stats[name]={'baseline':stats};continue
            if name not in left or name not in right:
                emit({'category':'missing_current_file' if name not in right else 'new_current_file','file':name});continue
            a,sa=load(left[name],'baseline',lambda record:emit({**record,'file':name,'raw_file':left[name].name}));b,sb=load(right[name],'current',lambda record:emit({**record,'file':name,'raw_file':right[name].name}));file_stats[name]={'baseline':sa,'current':sb,'baseline_raw_name':left[name].name,'current_raw_name':right[name].name}
            if a is None or b is None:continue
            operations=aligned_opcodes([r for _,r in a],[r for _,r in b])
            for operation,i,j,k,l in operations:
                if operation=='equal':
                    for (la,ra),(lb,rb) in zip(a[i:j],b[k:l]):
                        for category,path,old,new in diffs(ra,rb):
                            emit({'category':category,'file':name,'baseline_line':la,'current_line':lb,'path':path,'baseline':old,'current':new})
                else:
                    emit({'category':'record_sequence_'+operation,'file':name,'baseline_lines':[x[0] for x in a[i:j]],'current_lines':[x[0] for x in b[k:l]],'baseline_count':j-i,'current_count':l-k,'payload_preview_only':operation=='alignment_unresolved','baseline_records':[x[1] for x in (a[i:j] if operation!='alignment_unresolved' else a[i:min(j,i+5)])],'current_records':[x[1] for x in (b[k:l] if operation!='alignment_unresolved' else b[k:min(l,k+5)])]})
    summary={'mode':'audit' if args.audit_only else 'comparison','baseline':str(args.baseline),'current':None if args.current is None else str(args.current),'baseline_files':len(left),'current_files':len(right),'counts':dict(counts),'unresolved_difference_records':differences,'per_file':{k:dict(v) for k,v in per_file.items()},'file_stats':file_stats,'status':'AUDIT_ONLY' if args.audit_only else ('EXACT_PROJECTED_MATCH' if differences==0 else 'REVIEW_REQUIRED')}
    (args.output/'filename-alias-ledger.json').write_text(json.dumps(alias_ledger,ensure_ascii=False,indent=2),encoding='utf-8')
    (args.output/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2,default=json_default),encoding='utf-8')
    print(json.dumps({k:v for k,v in summary.items() if k not in ['file_stats','per_file']},indent=2));return 0 if differences==0 else 1
if __name__=='__main__':sys.exit(main())
