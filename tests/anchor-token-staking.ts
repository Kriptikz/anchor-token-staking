import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { AnchorTokenStaking } from '../target/types/anchor_token_staking';
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, Token } from '@solana/spl-token';
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
  let user1TokenAAccount = null;


  // Program Token Vault PDA
  let pdaVaultTokenAAddress = null;
  let pdaVaultTokenABump = null;



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
    user1TokenAAccount = await mintA.createAccount(user1.publicKey);

    // Mint some token A to user1TokenAccountA
    await mintA.mintTo(
      user1TokenAAccount,
      mintAAuthority.publicKey,
      [mintAAuthority],
      MINT_A_AMOUNT,
    );

    // Find our vault PDA
    [pdaVaultTokenAAddress, pdaVaultTokenABump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("vault"), mintA.publicKey.toBuffer()],
      program.programId
    )

    console.log(`PDA Token A Vault Address: ${pdaVaultTokenAAddress}, Bump: ${pdaVaultTokenABump}`);
    console.log("User1 Pubkey: ", user1.publicKey.toString());
    console.log("User1 Token A Account: ", user1TokenAAccount.toString());
    console.log("MintA Authority Pubkey: ", mintAAuthority.publicKey.toString());
    console.log("MintA Pubkey: ", mintA.publicKey.toString());

    let amount = (await mintA.getAccountInfo(user1TokenAAccount)).amount.toNumber();
    let mintAMintInfoAuthority = (await mintA.getMintInfo()).mintAuthority.toString();
    console.log("MintA Authority: ", mintAMintInfoAuthority);

    assert.equal(MINT_A_AMOUNT, amount);
    assert.equal(mintAAuthority.publicKey, mintAMintInfoAuthority);
  });
});
