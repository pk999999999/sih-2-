const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const solc = require('solc');
const ganache = require('ganache');
const { ethers } = require('ethers');

test('hash-only registry enforces registrar, immutable hashes and custody', async () => {
  const chain = ganache.provider({logging:{quiet:true}});
  try {
    const provider = new ethers.BrowserProvider(chain);
    const registrar = await provider.getSigner(0), stranger = await provider.getSigner(1);
    const output = JSON.parse(solc.compile(JSON.stringify({language:'Solidity',sources:{'EvidenceRegistry.sol':{content:fs.readFileSync(path.join(__dirname,'EvidenceRegistry.sol'),'utf8')}},settings:{outputSelection:{'*':{'*':['abi','evm.bytecode']}}}})));
    assert.equal((output.errors || []).filter(e => e.severity === 'error').length, 0);
    const artifact = output.contracts['EvidenceRegistry.sol'].EvidenceRegistry;
    const registry = await new ethers.ContractFactory(artifact.abi, artifact.evm.bytecode.object, registrar).deploy();
    await registry.waitForDeployment();
    const hash = s => ethers.sha256(ethers.toUtf8Bytes(s));
    const id = hash('id'), original = hash('original'), owner = hash('owner'), recipient = hash('recipient');
    await assert.rejects(registry.connect(stranger).registerEvidence(id, original, owner));
    await (await registry.registerEvidence(id, original, owner)).wait();
    assert.equal(await registry.verifyEvidence(id, original), true);
    assert.equal(await registry.verifyEvidence(id, hash('tampered')), false);
    await assert.rejects(registry.registerEvidence(id, hash('replacement'), owner));
    await assert.rejects(registry.transferCustody(id, recipient, owner));
    await assert.rejects(registry.connect(stranger).transferCustody(id, owner, recipient));
    await (await registry.transferCustody(id, owner, recipient)).wait();
    assert.equal((await registry.getEvidence(id)).custodian, recipient);
    assert.equal((await registry.getCustodyHistory(id)).length, 2);
  } finally { await chain.disconnect(); }
});
