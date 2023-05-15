import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { GreenDapp } from "../target/types/green_dapp";
import {
  PublicKey,
  SystemProgram,
  Transaction,
  Connection,
  Commitment,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { assert } from "chai";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

describe("green_dapp", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local("http://127.0.0.1:8899");
  anchor.setProvider(provider);

  const program = anchor.workspace.GreenDapp as Program<GreenDapp>;

  const vaultSeed = "vault";
  const stateSeed = "cityStateSeed";

  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const initializer = anchor.web3.Keypair.generate();

  let mintA = null as PublicKey;
  let mintB = null as PublicKey;
  let TokenAccountA = null;
  let TokenAccountVault = null;
  let TokenAccountB = null;
  let TokenAccountVaultB = null;

  const cityStateKey = PublicKey.findProgramAddressSync(
    [
      Buffer.from(anchor.utils.bytes.utf8.encode(stateSeed)),
      initializer.publicKey.toBuffer(),
    ],
    program.programId
  )[0];

  console.log("city state: ", cityStateKey);

  it("Initialize program state", async () => {
    // 1. Airdrop 1 SOL to payer
    const fromAirdropSignature = await provider.connection.requestAirdrop(
      payer.publicKey,
      10000000000
    );
    await provider.connection.confirmTransaction(fromAirdropSignature);

    // 2. Fund main roles: initializer and taker
    const fundingTx = new Transaction();
    fundingTx.add(
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: initializer.publicKey,
        lamports: 100000000,
      })
    );

    await provider.sendAndConfirm(fundingTx, [payer]);

    // 3. Create dummy token mints: mintA and mintB
    mintA = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      9
    );

    mintB = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      9
    );

    //4. Create token accounts for dummy token mints and both main roles
    TokenAccountA = await createAccount(
      provider.connection,
      initializer,
      mintA,
      initializer.publicKey
    );
    //console.log("wtf aaaa ", mintA);
    TokenAccountB = await createAccount(
      provider.connection,
      initializer,
      mintB,
      initializer.publicKey
    );

    TokenAccountVault = findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(vaultSeed)),
        initializer.publicKey.toBuffer(),
        mintA.toBuffer(),
      ],
      program.programId
    )[0];

    TokenAccountVaultB = findProgramAddressSync(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode(vaultSeed)),
        initializer.publicKey.toBuffer(),
        mintB.toBuffer(),
      ],
      program.programId
    )[0];

    console.log(TokenAccountA);

    console.log("pub key: " + initializer.publicKey);

    console.log(TokenAccountVault);

    // 5. Mint dummy tokens to initializerTokenAccountA and takerTokenAccountB
  });

  it("Initialize city", async () => {
    await program.methods
      .initialize("Lisbon")
      .accounts({
        cityState: cityStateKey,
        signer: initializer.publicKey,
        signerGreenTokenAccount: TokenAccountA,
        greenTokenAccountVault: TokenAccountVault,
        mintGreen: mintA,
        signerRedTokenAccount: TokenAccountB,
        redTokenAccountVault: TokenAccountVaultB,
        mintRed: mintB,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([initializer])
      .rpc();

    program.account.cityState.all();
    let fetchedVault = await getAccount(
      provider.connection,
      TokenAccountVault
    ); /*
    let fetchedEscrowState = await program.account.escrowState.fetch(
      escrowStateKey
    );*/

    console.log("vault: ", fetchedVault);
  });

  it("send coins to vault", async () => {
    await mintTo(
      provider.connection,
      initializer,
      mintA,
      TokenAccountVault,
      mintAuthority,
      50
    );

    let fetchedVault = await getAccount(provider.connection, TokenAccountVault);

    /*
    let fetchedEscrowState = await program.account.escrowState.fetch(
      escrowStateKey
    );*/

    console.log("vault 2: -> ", fetchedVault);
  });

  it("send coins to red vault", async () => {
    await mintTo(
      provider.connection,
      initializer,
      mintB,
      TokenAccountVaultB,
      mintAuthority,
      15
    );
  });

  it("withdraw city", async () => {
    await program.methods
      .withdrawFromTokenAccount(new anchor.BN(40))
      .accounts({
        cityState: cityStateKey,
        signer: initializer.publicKey,
        takerDepositTokenAccount: TokenAccountA,
        greenTokenAccountVault: TokenAccountVault,
        redTokenAccountVault: TokenAccountVaultB,
        mintGreen: mintA,
        mintRed: mintB,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([initializer])
      .rpc();

    let fetchedVault = await getAccount(
      provider.connection,
      TokenAccountVault
    ); /*
    let fetchedEscrowState = await program.account.escrowState.fetch(
      escrowStateKey
    );*/

    console.log("vault: ", fetchedVault);
  });
});
