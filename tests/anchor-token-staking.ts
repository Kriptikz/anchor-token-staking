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

  // Simple Delay
  const delay = ms => new Promise(res => setTimeout(res, ms));

  // Initial Mint amount
  const MINT_A_AMOUNT = 1_000;
  const MINT_A_REWARD_VAULT_AMOUNT = 10_000;

  // Amount to Stake/Unstake
  const AMOUNT_TO_STAKE = 200;

  // User Keypairs
  const user1 = anchor.web3.Keypair.generate();

  // Payer Keypair
  const payer = anchor.web3.Keypair.generate();


  // Mint Authority Keypairs
  const mintAAuthority = anchor.web3.Keypair.generate();

  // Mint Accounts
  let mintA = null;
  
  // Associated Token Accounts
  let user1TokenAAccount = null;

  // StakeAccounts
  let user1StakeAccountAddress = null;
  let user1StakeAccountBump = null;


  // Program Token Stake Vault PDA
  let pdaStakeVaultTokenAAddress = null;
  let pdaStakeVaultTokenABump = null;

  // Program Token Reward Vault PDA
  let pdaRewardVaultTokenAAddress = null;
  let pdaRewardVaultTokenABump = null;





  it('Test Set Up', async () => {
    // Airdrop sol to user1
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user1.publicKey, LAMPORTS_PER_SOL),
      "confirmed"
    );

    // Airdrop sol to user1
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 2 * LAMPORTS_PER_SOL),
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
      payer,
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

    // Find our stake vault PDA
    [pdaStakeVaultTokenAAddress, pdaStakeVaultTokenABump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("stake-vault"), mintA.publicKey.toBuffer()],
      program.programId
    );

    // Find our reward vault PDA
    [pdaRewardVaultTokenAAddress, pdaRewardVaultTokenABump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("reward-vault"), mintA.publicKey.toBuffer()],
      program.programId
    );

    console.log(`PDA Token A Vault Address: ${pdaStakeVaultTokenAAddress}, Bump: ${pdaStakeVaultTokenABump}`);
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

  it('Initialize Token A Program Staking Vault', async () => {
    await provider.connection.confirmTransaction(
      await program.rpc.initializeStakeVault(
        pdaStakeVaultTokenABump, {
          accounts: {
            stakeVault: pdaStakeVaultTokenAAddress,
            payer: payer.publicKey,
            mint: mintA.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
          signers: [payer]
      })
    );

    let pdaStakeVaultTokenAAccountOwner = (await provider.connection.getAccountInfo(pdaStakeVaultTokenAAddress)).owner;
    assert.equal(TOKEN_PROGRAM_ID.toString(), pdaStakeVaultTokenAAccountOwner.toString(), );

    let pdaStakeVaultTokenAAccountAmount = await (await mintA.getAccountInfo(pdaStakeVaultTokenAAddress)).amount.toNumber();
    assert.equal(0, pdaStakeVaultTokenAAccountAmount);
  });

  it('Initialize Token A Program Reward Vault', async () => {
    await provider.connection.confirmTransaction(
      await program.rpc.initializeRewardVault(
        pdaRewardVaultTokenABump, {
          accounts: {
            rewardVault: pdaRewardVaultTokenAAddress,
            payer: payer.publicKey,
            mint: mintA.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
          signers: [payer]
      })
    );

    // Mint 10_000 Token A to reward vault
    await mintA.mintTo(
      pdaRewardVaultTokenAAddress,
      mintAAuthority.publicKey,
      [mintAAuthority],
      MINT_A_REWARD_VAULT_AMOUNT,
    );

    let pdaRewardVaultTokenAAccountOwner = (await provider.connection.getAccountInfo(pdaRewardVaultTokenAAddress)).owner;
    assert.equal(TOKEN_PROGRAM_ID.toString(), pdaRewardVaultTokenAAccountOwner.toString(), );

    let pdaRewardVaultTokenAAccountAmount = await (await mintA.getAccountInfo(pdaRewardVaultTokenAAddress)).amount.toNumber();
    assert.equal(MINT_A_REWARD_VAULT_AMOUNT, pdaRewardVaultTokenAAccountAmount);
  });

  it('Initialize a StakeAccount for our user1', async () => {
    // Create our users StakeAccount PDA
    [user1StakeAccountAddress, user1StakeAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("stake-account"), mintA.publicKey.toBuffer(), user1.publicKey.toBuffer()], program.programId);

    await provider.connection.confirmTransaction(
      await program.rpc.initializeStakeAccount(
        user1StakeAccountBump, {
          accounts: {
            stakeAccount: user1StakeAccountAddress,
            stakeAuthority: user1.publicKey,
            mint: mintA.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
          signers: [user1]
      })
    );

    let stakeAccount = await program.account.stakeAccount.fetch(user1StakeAccountAddress);
    let authority = stakeAccount.authority;
    let amount = stakeAccount.stakedAmount;

    console.log("Before Staking");
    console.log("Staked At: ", stakeAccount.stakeStartTime.toNumber());
    console.log("Unclaimed Amount: ", stakeAccount.unclaimedAmount.toNumber());

    assert.equal(user1.publicKey.toString(), authority.toString());
    assert.equal(0, amount);

  });

  it('Stake Tokens with user1', async () => {

    await provider.connection.confirmTransaction(
      await program.rpc.stakeTokens(
        new anchor.BN(AMOUNT_TO_STAKE), {
          accounts: {
            stakeVault: pdaStakeVaultTokenAAddress,
            stakeAccount: user1StakeAccountAddress,
            staker: user1.publicKey,
            stakerTokenAccount: user1TokenAAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
          signers: [user1]
      })
    );

    let pdaTokenAAccountAmount = await (await mintA.getAccountInfo(pdaStakeVaultTokenAAddress)).amount.toNumber();
    assert.equal(AMOUNT_TO_STAKE, pdaTokenAAccountAmount);

    let stakeAccount = await program.account.stakeAccount.fetch(user1StakeAccountAddress);
    let amount = stakeAccount.stakedAmount;

    console.log("After Staking");
    console.log("Staked At: ", stakeAccount.stakeStartTime.toNumber());
    console.log("Unclaimed Amount: ", stakeAccount.unclaimedAmount.toNumber());

    assert.equal(AMOUNT_TO_STAKE, amount);

  });

  it('Unstake tokens fail - Stake Locked', async () => {
    // Create our users StakeAccount PDA
    [user1StakeAccountAddress, user1StakeAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("stake-account"), mintA.publicKey.toBuffer(), user1.publicKey.toBuffer()], program.programId);

    try {
      await provider.connection.confirmTransaction(
        await program.rpc.unstakeTokens(
          new anchor.BN(AMOUNT_TO_STAKE), {
            accounts: {
              stakeVault: pdaStakeVaultTokenAAddress,
              stakeAccount: user1StakeAccountAddress,
              to: user1TokenAAccount,
              authority: user1.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
            },
            signers: [user1]
        })
      )
    } catch (errorMessage) {
      console.log(errorMessage.toString());
    }

  });

  it('Unstake tokens from our program', async () => {

    // Create our users StakeAccount PDA
    [user1StakeAccountAddress, user1StakeAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("stake-account"), mintA.publicKey.toBuffer(), user1.publicKey.toBuffer()], program.programId);

    await delay(5000);

    await provider.connection.confirmTransaction(
      await program.rpc.unstakeTokens(
        new anchor.BN(AMOUNT_TO_STAKE), {
          accounts: {
            stakeVault: pdaStakeVaultTokenAAddress,
            stakeAccount: user1StakeAccountAddress,
            to: user1TokenAAccount,
            authority: user1.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
          signers: [user1]
      })
    );

    let pdaTokenAAccountAmount = await (await mintA.getAccountInfo(pdaStakeVaultTokenAAddress)).amount.toNumber();
    assert.equal(0, pdaTokenAAccountAmount);

    let stakeAccount = await program.account.stakeAccount.fetch(user1StakeAccountAddress);
    let amount = stakeAccount.stakedAmount;

    console.log("After Unstaking");
    console.log("Staked At: ", stakeAccount.stakeStartTime.toNumber());
    console.log("Unclaimed Amount: ", stakeAccount.unclaimedAmount.toNumber());

    assert.equal(0, amount);

  });

  it('Claim stake rewards', async () => {

    // Create our users StakeAccount PDA
    [user1StakeAccountAddress, user1StakeAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("stake-account"), mintA.publicKey.toBuffer(), user1.publicKey.toBuffer()], program.programId);

    let stakeAccount = await program.account.stakeAccount.fetch(user1StakeAccountAddress);
    let unclaimedAmount = stakeAccount.unclaimedAmount.toNumber();
    console.log("Before Claiming");
    console.log("Staked At: ", stakeAccount.stakeStartTime.toNumber());
    console.log("Unclaimed Amount: ", unclaimedAmount);

    await provider.connection.confirmTransaction(
      await program.rpc.claimRewards({
          accounts: {
            rewardVault: pdaRewardVaultTokenAAddress,
            stakeAccount: user1StakeAccountAddress,
            to: user1TokenAAccount,
            authority: user1.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          },
          signers: [user1]
      })
    );

    let pdaRewardVaultTokenAAccountAmount = await (await mintA.getAccountInfo(pdaRewardVaultTokenAAddress)).amount.toNumber();
    assert.equal(MINT_A_REWARD_VAULT_AMOUNT - unclaimedAmount, pdaRewardVaultTokenAAccountAmount);

    stakeAccount = await program.account.stakeAccount.fetch(user1StakeAccountAddress);
    console.log("After Claiming");
    console.log("Staked At: ", stakeAccount.stakeStartTime.toNumber());
    console.log("Unclaimed Amount: ", stakeAccount.unclaimedAmount.toNumber());

    let user1TokenAAccountAmount = await (await mintA.getAccountInfo(user1TokenAAccount)).amount.toNumber();
    assert.equal(unclaimedAmount + MINT_A_AMOUNT, user1TokenAAccountAmount);

  });

});
