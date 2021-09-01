const fs = require('fs');
const path = require('path');
const {
  Actor,
  ActorSubclass,
  HttpAgent,
  SignIdentity,
} = require('@dfinity/agent');
const { Principal } = require('@dfinity/principal');
const { Ed25519KeyIdentity } = require('@dfinity/identity');
const fetch = require('node-fetch');
const { idlFactory } = require('../candid/xtc');

const DANK_CANISTER_ID = 'aanaa-xaaaa-aaaah-aaeiq-cai';

const createIdentity = (seed) => {
  const seed1 = new Array(32).fill(0);
  seed1[0] = seed;
  return Ed25519KeyIdentity.generate(new Uint8Array(seed1));
};

const getActorWithIdentity = (defaultAgent, identity) => {
  const agent = new HttpAgent({
    source: defaultAgent,
    identity,
  });

  const actor = Actor.createActor(idlFactory, {
    canisterId: DANK_CANISTER_ID,
    agent,
  });

  return actor;
};

const setupXtc = () => {
  const defaultAgent = new HttpAgent({ host: 'https://ic0.app', fetch });
  const identity = createIdentity();
  const xtc = getActorWithIdentity(defaultAgent, identity);

  return xtc;
};

module.exports = setupXtc;
