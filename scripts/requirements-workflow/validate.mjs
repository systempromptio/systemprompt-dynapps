import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'../../services/requirements-workflow/upstream');
const {loadConfig}=await import(join(root,'.project/eval/lib/config.mjs'));
const {resolveArtifact,resolveContext,resolveGolden}=await import(join(root,'.project/eval/lib/resolve.mjs'));
const {judgeArtifact,previewJudgePrompt,generateArtifact,previewGenerationPrompt}=await import(join(root,'.project/eval/lib/judge.mjs'));
const {createClient}=await import(join(root,'.project/eval/lib/bedrock.mjs'));
const {checkStructure}=await import(join(root,'.project/eval/lib/structure.mjs'));
const [inputPath,outputPath]=process.argv.slice(2);
const input=JSON.parse(readFileSync(inputPath,'utf8'));
const work=resolve(dirname(inputPath)), change=join(work,input.workflow.name);
mkdirSync(join(change,'.context'),{recursive:true});
const packet=input.workflow.evidence.map(d=>`## ${d.source_type}: ${d.source_ref}\nSource: ${d.uri}\nVersion: ${d.version}\n\n${d.body}`).join('\n\n');
writeFileSync(join(change,'.context/initial-context.md'),packet+'\n\nDelivery scope selected by user: functional storefront change. Integration implementation is owned by separate changes.\n');
for(const artifact of Object.values(input.workflow.artifacts)) for(const [path,body] of Object.entries(artifact.files)) {
 const target=resolve(change,path); if(!target.startsWith(change+'/')) throw Error('Invalid artifact path');
 mkdirSync(dirname(target),{recursive:true});writeFileSync(target,body);
 if(/^(discovery|design|proposal|grooming)\.md$/.test(path)) { const settled=join(change,path.replace('.md','/final.md'));mkdirSync(dirname(settled),{recursive:true});writeFileSync(settled,body); }
}
const config=loadConfig();
const goldenDir=join(root,'.project/eval/evalsuite/openspec-feature/fixtures/store-locator');
const reports=[];
if(input.phase==='fsd' || input.operation==='scaffold') {
 const generation=spawnSync(process.execPath,[join(root,'skills/plan/openspec-workflow/scripts/generate-functional-artifacts.mjs'),change,'--template',join(root,'.project/templates/openspec/schemas/feature/templates/fsd.md'),'--out-dir',join(work,'projection')],{encoding:'utf8',timeout:60000});
 if(generation.status!==0) throw Error('FSD projection failed: '+generation.stderr+generation.stdout);
 if(input.operation==='scaffold') {
  const {readdirSync}=await import('node:fs');
  const files={};const collect=(dir,prefix='')=>{for(const e of readdirSync(dir,{withFileTypes:true})){const p=join(dir,e.name),rel=prefix+e.name;if(e.isDirectory())collect(p,rel+'/');else files[rel==='stories.md'?rel:'fsd/'+rel]=readFileSync(p,'utf8');}};
  collect(join(work,'projection'));writeFileSync(outputPath,JSON.stringify({files}));process.exit(0);
 }
 const {readdirSync}=await import('node:fs');
 writeFileSync(join(change,'fsd-scaffold.md'),readdirSync(join(work,'projection')).filter(p=>p!=='stories.md').map(p=>readFileSync(join(work,'projection',p),'utf8')).join('\n\n'));
 const fsdFiles=Object.entries(input.workflow.artifacts.fsd.files);
 const expected=readdirSync(join(work,'projection')).filter(p=>p!=='stories.md').sort();
 if(JSON.stringify(fsdFiles.map(([p])=>p.split('/').at(-1)).sort())!==JSON.stringify(expected)) throw Error('Supply every projected FSD document exactly once');
 for(const [path,body] of fsdFiles) {
  const projected=join(work,'projection',path.split('/').at(-1));
  if(!existsSync(projected)) throw Error('FSD path must match the deterministic scaffold');
  const extract=text=>text.split(/(?=^#### )/m).filter(s=>/^#### (User Story|Acceptance Criteria|Key behaviours)\s*$/m.test(s)).map(s=>s.split(/^### /m)[0].trim());
  if(JSON.stringify(extract(body))!==JSON.stringify(extract(readFileSync(projected,'utf8')))) throw Error('FSD changes a generated story, criterion or behaviour; fix upstream');
 }
}
const resolved=resolveArtifact(config,'openspec-feature',input.phase,{changeName:input.workflow.name});
const contextBlocks=resolveContext(change,resolved.contextIds,{flow:'ba',flowsConfig:config.flows,states:config.states});
if(!contextBlocks.some(c=>c.found && c.id==='.context/initial-context')) contextBlocks.push({id:'source-snapshot',path:'source-snapshot',found:true,content:packet});
contextBlocks.push({id:'live-source-binding',found:true,content:'This is a live Knowledge Bank export. Cite only actual source URIs and IDs in the packet. No project-context inbox files were returned; do not invent those paths. Preserve the complete template heading/table interface and every RQ/WBS in the programme, dispositioning other workstreams outside this linen-polo browsing change. No human workflow approval exists unless a signed receipt is supplied.'});
if(input.feedback) contextBlocks.push({id:'revision-findings',found:true,content:input.feedback});
if(input.workflow.grooming_review?.receipt) contextBlocks.push({id:'human-approval',path:'human-approval',found:true,content:input.workflow.grooming_review.receipt.payload});
let client;
if(input.operation==='generate') {
 client=createClient(config);
 const generated=await generateArtifact({client,resolved,contextBlocks});
 writeFileSync(outputPath,JSON.stringify({...generated,model:config.genModel,prompt:previewGenerationPrompt({resolved,contextBlocks})}));process.exit(0);
}
const candidates=input.phase==='specs' ? [['specs',Object.entries(input.workflow.artifacts.specs.files).map(([p,b])=>`--- ${p} ---\n${b}`).join('\n\n')]] : Object.entries(input.workflow.artifacts[input.phase].files);
for(const [path,candidate] of candidates) {
 const golden=resolveGolden(goldenDir,input.phase,{flow:'ba',flowsConfig:config.flows,states:config.states});
 const shape=golden.found?checkStructure(candidate,golden.content):{ok:false,findings:['Missing structural golden']};
 if(!shape.ok) {reports.push({path,threshold:resolved.passThreshold,verdict:{pass:false,score:0,critical_findings:shape.findings,structure:shape},semantic_status:'not_run_structure_failed'});continue;}
 const prompt=previewJudgePrompt({resolved,contextBlocks,golden,candidate});
 let cached;
 try {cached=JSON.parse(input.workflow.artifacts[input.phase].validation?.report??'{}').reports?.find(r=>r.path===path && JSON.stringify(r.model)===JSON.stringify(config.judgeModel) && r.threshold===resolved.passThreshold && JSON.stringify(r.prompt)===JSON.stringify(prompt) && r.raws?.length===3);} catch {}
 if(cached) {reports.push({...cached,cache:'exact_prompt'});continue;}
 client??=createClient(config);
 const result=await judgeArtifact({client,resolved,contextBlocks,golden,candidate,samples:3});
 reports.push({path,prompt,model:config.judgeModel,threshold:resolved.passThreshold,...result});
}
writeFileSync(outputPath,JSON.stringify({passed:reports.length>0&&reports.every(r=>r.verdict.pass===true && r.verdict.score>=r.threshold && r.verdict.critical_findings.length===0 && !r.verdict.validity?.error),reports}));
