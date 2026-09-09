"""Call a bridge-loopback MCP tool inside its own clean container; never export its key."""
import json,pathlib,urllib.request,sys
h=pathlib.Path.home();key=(h/'.config/astound/bridge-loopback.key').read_text().strip()
u='http://127.0.0.1:48217/mcp/'+sys.argv[1]
headers={'Authorization':'Bearer '+key,'Content-Type':'application/json','Accept':'application/json,text/event-stream'}
def call(body):
 r=urllib.request.Request(u,headers=headers,data=json.dumps(body).encode())
 with urllib.request.urlopen(r,timeout=45) as x:
  if x.headers.get('Mcp-Session-Id'):headers['Mcp-Session-Id']=x.headers['Mcp-Session-Id']
  raw=x.read().decode()
 if not raw.strip():return {}
 try:return json.loads(raw)
 except json.JSONDecodeError:
  messages=[]
  for l in raw.splitlines():
   if l.startswith('data:'):
    try:messages.append(json.loads(l[5:].strip()))
    except json.JSONDecodeError:pass
  return messages[-1] if messages else {'non_json':raw[:300]}
call({'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2025-06-18','capabilities':{},'clientInfo':{'name':'india-review','version':'1'}}})
call({'jsonrpc':'2.0','method':'notifications/initialized'})
if len(sys.argv)>2:
 body={'jsonrpc':'2.0','id':3,'method':'tools/call','params':{'name':sys.argv[2],'arguments':json.loads(sys.stdin.read())}}
else:body={'jsonrpc':'2.0','id':2,'method':'tools/list'}
print(json.dumps(call(body)))
