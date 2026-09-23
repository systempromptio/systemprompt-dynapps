import json,glob,re,os,subprocess,sys
S=os.path.dirname(os.path.abspath(__file__))
def psql(sql):
    p=subprocess.run(['docker','exec','-i','sp-systemprompt-astound-dcd323ed-local-postgres-1','psql','-U','systemprompt','-d','sp_prod_analysis','-X','-q','-At','-v','ON_ERROR_STOP=1'],input=sql,capture_output=True,text=True)
    return p.stdout,p.stderr
# realistic parameter values from the data
uid=psql("select user_id from ai_requests group by 1 order by count(*) desc limit 1")[0].strip()
sid=psql("select session_id from ai_requests group by 1 order by count(*) desc limit 1")[0].strip()
ctx=psql("select context_id from ai_requests group by 1 order by count(*) desc limit 1")[0].strip()
rid=psql("select id from ai_requests order by created_at desc limit 1")[0].strip()
def val(t,i):
    return {'Timestamptz':"'2026-09-01'::timestamptz" if i%2==0 else "'2026-09-23'::timestamptz",
            'Timestamp':"'2026-09-01'::timestamp" if i%2==0 else "'2026-09-23'::timestamp",
            'Date':"'2026-09-01'::date" if i%2==0 else "'2026-09-23'::date",
            'Text':"NULL::text",'Varchar':"NULL::text",'Bool':"false",'Int8':"50::int8",'Int4':"50::int4",'Float8':"0.5::float8",
            'TextArray':"NULL::text[]",'Interval':"'30 days'::interval",'Uuid':"NULL::uuid",'Jsonb':"NULL::jsonb"}.get(t,"NULL")
big={'logs','ai_request_messages','event_outbox','governance_decisions','analytics_events','ai_requests','ai_request_payloads','conversation_facts','plugin_usage_events','user_sessions','users','mcp_tool_executions','feedback_ingestion_outbox','ai_safety_findings','ai_request_tool_calls','ai_request_scopes'}
def collect(root,label):
    out=[]
    for f in glob.glob(root+'/query-*.json'):
        j=json.load(open(f)); q=j['query']
        if not re.match(r'\s*(SELECT|WITH)',q,re.I) or 'sqlx_' in q: continue
        tables=set(re.findall(r'(?:FROM|JOIN)\s+([a-z_]+)',q))&big
        if not tables: continue
        out.append((len(tables)*10+len(q)//200,label,os.path.basename(f)[6:22],q,j['describe']['parameters'].get('Left',[]),sorted(tables)))
    return out
qs=collect('../systemprompt-core/crates/domain/analytics/.sqlx','core-analytics')+collect('../systemprompt-core/crates/domain/ai/.sqlx','core-ai')+collect('../systemprompt-core/crates/infra/security/.sqlx','core-security')
qs.sort(reverse=True)
seen=set(); results=[]
for score,label,h,q,params,tables in qs:
    if len(results)>=int(sys.argv[1]) : break
    if h in seen: continue
    seen.add(h)
    sql=q
    # substitute $N (highest first)
    for i in range(len(params),0,-1):
        v=val(params[i-1],i)
        # smarter guesses for text params by name proximity
        ctxs=re.findall(r'(\w+)\s*(?:=|IS NOT DISTINCT FROM|IN|ANY)\s*\(?\$%d\b'%i,q)
        if params[i-1] in('Text','Varchar'):
            if any('user_id' in c for c in ctxs): v=f"'{uid}'::text"
            elif any('session_id' in c for c in ctxs): v=f"'{sid}'::text"
            elif any('context_id' in c for c in ctxs): v=f"'{ctx}'::text"
            elif any(c in('id','request_id','ai_request_id') for c in ctxs): v=f"'{rid}'::text"
        sql=re.sub(r'\$%d\b'%i,v,sql)
    out,err=psql("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) "+sql)
    if err.strip():
        results.append(dict(label=label,hash=h,tables=tables,error=err.strip().splitlines()[0][:160])); continue
    try: plan=json.loads(out)[0]
    except Exception as e:
        results.append(dict(label=label,hash=h,tables=tables,error='parse:'+str(e)[:100])); continue
    seq=[];disk=[];rows=0
    def walk(n):
        global rows
        if n.get('Node Type')=='Seq Scan' and n.get('Relation Name') in big: seq.append((n['Relation Name'],n.get('Actual Rows',0),n.get('Actual Loops',1)))
        if n.get('Sort Method','').startswith('external') or n.get('Sort Space Type')=='Disk': disk.append(n.get('Sort Space Used',0))
        if 'Batches' in n and n['Batches']>1: disk.append('hash-batches-%d'%n['Batches'])
        for c in n.get('Plans',[]): walk(c)
    walk(plan['Plan'])
    results.append(dict(label=label,hash=h,tables=tables,ms=round(plan['Execution Time'],1),plan_ms=round(plan['Planning Time'],1),seq=seq,disk=disk,shared_read=plan['Plan'].get('Shared Read Blocks',0),shared_hit=plan['Plan'].get('Shared Hit Blocks',0),temp=plan['Plan'].get('Temp Written Blocks',0)))
json.dump(results,open(S+'/f33_explain.json','w'),indent=1)
for r in results:
    if 'error' in r: print(f"ERR  {r['label']:14} {r['hash']} {r['tables']} :: {r['error']}")
    else: print(f"{r['ms']:8.1f}ms {r['label']:14} {r['hash']} seq={[(t,n) for t,n,l in r['seq']]} disk={r['disk']} temp={r['temp']} {r['tables']}")
