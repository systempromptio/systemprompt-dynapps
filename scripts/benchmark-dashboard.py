#!/usr/bin/env python3
"""Benchmark dashboard SQL in an ALREADY INITIALIZED disposable *_test database.

Requires psycopg 3. Set DASHBOARD_BENCH_DATABASE_URL without printing credentials.
Only synthetic perf-* fixture rows are inserted. Never point this at a live DB.
Results and execution plans go to .build/dashboard-performance (gitignored).
Use --seed N then --measure; repeat at 10000, 100000 and 1000000 requests.
"""
import argparse
import concurrent.futures
import json
import os
from pathlib import Path
import re
import statistics
import time
import urllib.parse

import psycopg

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / '.build/dashboard-performance'


def connect():
    url = os.environ['DASHBOARD_BENCH_DATABASE_URL']
    if not urllib.parse.urlsplit(url).path.endswith('_test'):
        raise SystemExit('Refusing a database without the _test suffix')
    c = psycopg.connect(url, autocommit=True)
    c.execute("SET statement_timeout = '30s'")
    return c


def seed(c, count):
    c.execute("SET statement_timeout = '5min'")
    c.execute("""INSERT INTO users(id,name,email,display_name,roles)
        SELECT 'perf-user-'||n, 'Person '||n, 'perf-'||n||'@example.test',
               'Person '||n, ARRAY['user'] FROM generate_series(0,999) n
        ON CONFLICT DO NOTHING""")
    c.execute("""INSERT INTO user_contexts(context_id,user_id,name,created_at,updated_at)
        SELECT 'perf-context-'||n, 'perf-user-'||mod(n,1000),
               CASE WHEN mod(n,2)=0 THEN 'Topic '||n ELSE 'Gateway conversation' END,
               NOW()-make_interval(days=>mod(n,120)), NOW()-make_interval(days=>mod(n,120))
        FROM generate_series(0,%s) n ON CONFLICT DO NOTHING""", (count // 10 - 1,))
    c.execute("""INSERT INTO ai_requests(id,request_id,user_id,context_id,
        gateway_conversation_id,actor_kind,actor_id,provider,model,status,request_kind,
        input_tokens,output_tokens,cost_microdollars,created_at)
        SELECT 'perf-request-'||n,'perf-request-'||n,'perf-user-'||mod(n/10,1000),
               'perf-context-'||(n/10),'perf-thread-'||(n/10),'user','perf-user-'||mod(n/10,1000),
               'anthropic','perf-model-'||mod(n,3),'completed', CASE WHEN mod(n,10)=9 THEN 'probe' ELSE 'turn' END,
               1000,100,5000, NOW()-make_interval(days=>mod(n/10,120))+make_interval(secs=>mod(n,10))
        FROM generate_series(0,%s) n ON CONFLICT DO NOTHING""", (count - 1,))
    c.execute("""INSERT INTO ai_request_messages(id,request_id,role,content,sequence_number)
        SELECT 'perf-message-'||n, 'perf-request-'||(n*10), 'user',
          '=== USER PROMPT === <system-reminder>metadata</system-reminder> Question '||n||' '||repeat('context ',128),0
        FROM generate_series(0,%s) n ON CONFLICT DO NOTHING""", (count // 10 - 1,))
    c.execute('ANALYZE')
    c.execute("SET statement_timeout = '30s'")


def literal(c, value):
    return psycopg.sql.Literal(value).as_string(c)


def bind(c, query, values):
    return re.sub(r'\$(\d+)', lambda m: '(' + literal(c, values[int(m[1])-1]) + ')', query)


def queries(c):
    base = ROOT / 'extensions/web/admin/src/repositories'
    page = (base / 'analytics/conversation_rows/page.sql').read_text()
    now = c.execute('SELECT NOW()').fetchone()[0]
    from datetime import timedelta
    args = [None,None,None,None,now-timedelta(days=30),None,False,False,'activity',True,50,0,False,False]
    yield 'contexts-all-30d', bind(c,page,args)
    args[12] = True
    yield 'contexts-users-30d', bind(c,page,args)
    args[12],args[4] = False,None
    yield 'contexts-all-time', bind(c,page,args)
    args[8] = 'cost'
    yield 'contexts-cost', bind(c,page,args)
    args[8],args[11] = 'activity',50000
    yield 'contexts-deep-page', bind(c,page,args)
    args[11],args[3] = 0,'%Question 99%'
    yield 'contexts-search', bind(c,page,args)
    roster=(base/'users/roster/page.sql').read_text()
    yield 'users',bind(c,roster,[None,30,'',None,None,'cost',True,50,0])
    history=(base/'analytics/conversations/history_page.sql').read_text()
    yield 'history',bind(c,history,[None,None,50,0,'00000000-0000-0000-0000-4c4547414359',None,False])


def measure(c, label, query, size):
    c.execute("SET jit = off")
    t = time.perf_counter()
    try:
        plan = c.execute('EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) '+query).fetchone()[0]
        (OUT/f'{size}-{label}-plan.json').write_text(json.dumps(plan,indent=2))
        samples=[]
        for _ in range(5):
            t=time.perf_counter(); c.execute(query).fetchall(); samples.append((time.perf_counter()-t)*1000)
        return {'first_plan_ms':plan[0]['Execution Time'],'warm_p50_ms':statistics.median(samples),
                'warm_max_ms':max(samples)}
    except psycopg.errors.QueryCanceled:
        return {'timeout_ms':round((time.perf_counter()-t)*1000)}


def concurrent_reads(query):
    with connect() as c:
        c.execute("SET jit = off")
        samples=[]
        for _ in range(3):
            t=time.perf_counter();c.execute(query).fetchall();samples.append((time.perf_counter()-t)*1000)
        return samples


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seed',type=int)
    parser.add_argument('--measure',action='store_true')
    parser.add_argument('--concurrency',type=int,default=10)
    args=parser.parse_args();OUT.mkdir(parents=True,exist_ok=True)
    with connect() as c:
        if args.seed:
            seed(c,args.seed); print('seeded',args.seed,flush=True)
        if not args.measure:return
        size=c.execute('SELECT COUNT(*) FROM ai_requests').fetchone()[0]
        c.execute((ROOT/'extensions/web/schema/migrations/050_dashboard_query_scaling.sql').read_text())
        results={}
        for name,query in queries(c):
            results[name]=measure(c,name,query,size); print(name,results[name],flush=True)
            if name in ('contexts-all-30d','contexts-users-30d','users') and 'timeout_ms' not in results[name]:
                with concurrent.futures.ThreadPoolExecutor(max_workers=args.concurrency) as executor:
                    try:
                        samples=sorted(v for row in executor.map(concurrent_reads,[query]*args.concurrency) for v in row)
                        results[name]['concurrent_p95_ms']=samples[max(0,int(len(samples)*.95)-1)]
                    except psycopg.errors.QueryCanceled:
                        results[name]['concurrent_timeout']=True
                print('concurrent',name,results[name],flush=True)
        (OUT/f'{size}-results.json').write_text(json.dumps(results,indent=2))


if __name__ == '__main__':main()
