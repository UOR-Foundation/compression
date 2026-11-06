// Request metadata only; actual source behavior and certification belong to LexLean.
import fs from 'node:fs/promises';
const [targetsPath,manifestPath]=process.argv.slice(2);
if(!targetsPath||!manifestPath)throw Error('usage: node request.mjs TARGETS_JSON FIXTURE_MANIFEST');
const descriptor=JSON.parse(await fs.readFile(targetsPath,'utf8'));
const manifest=JSON.parse(await fs.readFile(manifestPath,'utf8'));
if(descriptor.spec!=='lexlean/target-descriptors/1')throw Error('unexpected target descriptor schema');
if(manifest.schema!=='uorc/runtime-image-copy-production-fixture/1')throw Error('unexpected fixture schema');
const target=descriptor.targets.filter(t=>t.id==='rust-std');
if(target.length!==1||!/^[a-f0-9]{64}$/.test(target[0].digest))throw Error('missing actual rust-std target digest');
const roots=manifest.production_roots;
if(roots.length!==5||new Set(roots).size!==5||JSON.stringify([...roots].sort())!==JSON.stringify(roots))
 throw Error('expected exact sorted five-root inventory');
for(const root of roots)if(!root.startsWith('Uorc.Uorc.Core.RuntimeImageCopy.'))throw Error('unexpected requested module');
console.log(JSON.stringify({spec:'lexlean/production-request/1',roots:roots.map(root=>({
 root,package:{name:'uorc_image_copy_'+root.split('.').at(-1).toLowerCase(),version:'0.1.0'},
 targets:[{id:'rust-std',digest:target[0].digest}]
}))}));
