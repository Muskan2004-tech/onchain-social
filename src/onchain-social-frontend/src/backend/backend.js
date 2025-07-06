import { HttpAgent, Actor } from '@dfinity/agent';
import {
  idlFactory as backend_idl,
  canisterId as backend_id
} from '../../../declarations/onchain-social-backend'; // ✅ Correct path

const agent = new HttpAgent({ host: 'http://127.0.0.1:4943' });

if (process.env.DFX_NETWORK === 'local') {
  await agent.fetchRootKey();
}

const backendActor = Actor.createActor(backend_idl, {
  agent,
  canisterId: backend_id,
});

export default backendActor;
