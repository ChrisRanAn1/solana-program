import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
import { SimpleOrderbookDex } from "../target/types/simple_orderbook_dex";

describe("simple-orderbook-dex", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.simpleOrderbookDex as Program<SimpleOrderbookDex>;
  const connection = provider.connection;

  // user key
  const maker = Keypair.generate();
  const taker = Keypair.generate();

  let baseMint: PublicKey;
  let quoteMint: PublicKey;
  let market: PublicKey;
  let baseVault: PublicKey;
  let quoteVault: PublicKey;

  let makerBase: PublicKey;
  let makerQuote: PublicKey;
  let takerBase: PublicKey;
  let takerQuote: PublicKey;

  const BASE_DECIMALS = 6;
  const QUOTE_DECIMALS = 6;

  const airdrop = async (pk: PublicKey, sol = 2) => {
    const sig = await connection.requestAirdrop(pk, sol * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");
  };

  const orderPda = (marketKey: PublicKey, id: anchor.BN) =>
    PublicKey.findProgramAddressSync(
      [Buffer.from("order"), marketKey.toBuffer(), id.toArrayLike(Buffer, "le", 8)],
      program.programId
    )[0];

  before(async () => {
    await airdrop(provider.wallet.publicKey, 10);
    await airdrop(maker.publicKey);
    await airdrop(taker.publicKey);

    baseMint = await createMint(
      connection,
      (provider.wallet as anchor.Wallet).payer,
      provider.wallet.publicKey,
      null,
      BASE_DECIMALS
    );
    quoteMint = await createMint(
      connection,
      (provider.wallet as anchor.Wallet).payer,
      provider.wallet.publicKey,
      null,
      QUOTE_DECIMALS
    );

    [market] = PublicKey.findProgramAddressSync(
      [Buffer.from("market"), baseMint.toBuffer(), quoteMint.toBuffer()],
      program.programId
    );
    [baseVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("base_vault"), market.toBuffer()],
      program.programId
    );
    [quoteVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("quote_vault"), market.toBuffer()],
      program.programId
    );

    const walletPayer = (provider.wallet as anchor.Wallet).payer;
    makerBase = await createAssociatedTokenAccount(connection, walletPayer, baseMint, maker.publicKey);
    makerQuote = await createAssociatedTokenAccount(connection, walletPayer, quoteMint, maker.publicKey);
    takerBase = await createAssociatedTokenAccount(connection, walletPayer, baseMint, taker.publicKey);
    takerQuote = await createAssociatedTokenAccount(connection, walletPayer, quoteMint, taker.publicKey);

    // fund wallet
    await mintTo(connection, walletPayer, baseMint, makerBase, provider.wallet.publicKey, 1_000_000_000);
    await mintTo(connection, walletPayer, quoteMint, makerQuote, provider.wallet.publicKey, 1_000_000_000);
    await mintTo(connection, walletPayer, baseMint, takerBase, provider.wallet.publicKey, 1_000_000_000);
    await mintTo(connection, walletPayer, quoteMint, takerQuote, provider.wallet.publicKey, 1_000_000_000);
  });

  it("initialize market", async () => {
    await program.methods
      .initializeMarket()
      .accountsStrict({
        authority: provider.wallet.publicKey,
        baseMint,
        quoteMint,
        market,
        baseVault,
        quoteVault,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const m = await program.account.market.fetch(market);
    assert.ok(m.baseMint.equals(baseMint));
    assert.ok(m.quoteMint.equals(quoteMint));
    assert.equal(m.nextOrderId.toNumber(), 0);
  });

  it("place an ask, fill it partially, then cancel the rest", async () => {
    const askId = (await program.account.market.fetch(market)).nextOrderId as BN;
    const askPda = orderPda(market, askId);

    const price = new BN(2); // 1 base <-> 2 quote
    const amount = new BN(100);

    await program.methods
      .placeOrder({ ask: {} } as any, price, amount)
      .accountsStrict({
        owner: maker.publicKey,
        market,
        order: askPda,
        baseVault,
        quoteVault,
        ownerBaseAccount: makerBase,
        ownerQuoteAccount: makerQuote,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc();

    const vaultBaseAfter = await getAccount(connection, baseVault);
    assert.equal(vaultBaseAfter.amount.toString(), "100");

    // Taker buy 60 base, pay 120 quote
    await program.methods
      .fillOrder(new BN(60))
      .accountsStrict({
        taker: taker.publicKey,
        market,
        order: askPda,
        baseVault,
        quoteVault,
        takerBaseAccount: takerBase,
        takerQuoteAccount: takerQuote,
        makerBaseAccount: makerBase,
        makerQuoteAccount: makerQuote,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([taker])
      .rpc();

    const orderAfter = await program.account.order.fetch(askPda);
    assert.equal(orderAfter.amount.toNumber(), 40);

    // Maker cancle 40 base
    await program.methods
      .cancelOrder()
      .accountsStrict({
        owner: maker.publicKey,
        market,
        order: askPda,
        baseVault,
        quoteVault,
        ownerBaseAccount: makerBase,
        ownerQuoteAccount: makerQuote,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc();

    const vaultBaseEnd = await getAccount(connection, baseVault);
    assert.equal(vaultBaseEnd.amount.toString(), "0");
  });

  it("place a bid, taker sells into it", async () => {
    const bidId = (await program.account.market.fetch(market)).nextOrderId as BN;
    const bidPda = orderPda(market, bidId);

    const price = new BN(3);
    const amount = new BN(50);

    await program.methods
      .placeOrder({ bid: {} } as any, price, amount)
      .accountsStrict({
        owner: maker.publicKey,
        market,
        order: bidPda,
        baseVault,
        quoteVault,
        ownerBaseAccount: makerBase,
        ownerQuoteAccount: makerQuote,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc();

    const vaultQuoteAfter = await getAccount(connection, quoteVault);
    assert.equal(vaultQuoteAfter.amount.toString(), "150");

    await program.methods
      .fillOrder(new BN(50))
      .accountsStrict({
        taker: taker.publicKey,
        market,
        order: bidPda,
        baseVault,
        quoteVault,
        takerBaseAccount: takerBase,
        takerQuoteAccount: takerQuote,
        makerBaseAccount: makerBase,
        makerQuoteAccount: makerQuote,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([taker])
      .rpc();

    const filled = await program.account.order.fetch(bidPda);
    assert.equal(filled.amount.toNumber(), 0);

    const vaultQuoteEnd = await getAccount(connection, quoteVault);
    assert.equal(vaultQuoteEnd.amount.toString(), "0");
  });
});
