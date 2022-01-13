import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { AnchorTokenStaking } from '../target/types/anchor_token_staking';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { assert } from 'chai';

describe('anchor-token-staking', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorTokenStaking as Program<AnchorTokenStaking>;

  // Initial Mint amount
  const MINT_A_AMOUNT = 1_000;

  // User Keypairs
  const user1 = anchor.web3.Keypair.generate();


  // Mint Authority Keypairs
  const mintAAuthority = anchor.web3.Keypair.generate();

  // Mint Accounts
  let mintA = null;
  
  // Associated Token Accounts
  let user1TokenAccountA = null;
  let pdaVaultTokenAccountA = null;



  it('Test set up', async () => {
    // Airdrop sol to user1
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user1.publicKey, LAMPORTS_PER_SOL),
      "confirmed"
    );
    // Airdrop sol to mintAAuthority
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(mintAAuthority.publicKey, LAMPORTS_PER_SOL),
      "confirmed"
    );

    // Create our token A mint
    mintA = await Token.createMint(
      provider.connection,
      user1,
      mintAAuthority.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );

    // Create our user1 token A account
    user1TokenAccountA = await mintA.createAccount(user1.publicKey);

    // Mint some token A to user1TokenAccountA
    await mintA.mintTo(
      user1TokenAccountA,
      mintAAuthority.publicKey,
      [mintAAuthority],
      MINT_A_AMOUNT,
    );

    const [pdaVaultAddress, pdaVaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("vault")],
      program.programId
    )

    console.log(`Bump: ${pdaVaultBump}, PDA Vault Address: ${pdaVaultAddress.toBase58()}`);

    let amount = (await mintA.getAccountInfo(user1TokenAccountA)).amount.toNumber();
    console.log("User1 Token A Amount: ", amount);

    assert.equal(MINT_A_AMOUNT, amount);
  });
});
