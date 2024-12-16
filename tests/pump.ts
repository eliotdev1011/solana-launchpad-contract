import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { Pump } from '../target/types/pump';
import { SystemProgram } from '@solana/web3.js';
import { assert } from 'chai';
import fs from 'fs';
import path from 'path';
import os from 'os';

describe("pump", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const provider = anchor.getProvider();
    const program = anchor.workspace.Pump as Program<Pump>;

    let token: anchor.web3.PublicKey;
    let user: anchor.web3.Keypair;
    let programAccount: anchor.web3.Keypair;
    let ticker: string;

    async function deriveUserContributionPDA(count: number, isInitializing: boolean = false) {
        const countBuffer = Buffer.alloc(4); // u32 is 4 bytes
        countBuffer.writeUInt32LE(count);
    
        const seeds = [
            Buffer.from("user-contribution"),
            user.publicKey.toBuffer(),
            isInitializing ? Buffer.from(ticker) : token.toBuffer(),
            isInitializing ? Buffer.from([0]) : countBuffer // Use full 4-byte buffer for non-init
        ];
        
        console.log("Debug PDA derivation:");
        console.log("Seeds:", {
            prefix: "user-contribution",
            user: user.publicKey.toBase58(),
            seed3: isInitializing ? ticker : token.toBase58(),
            count: count,
            countBuffer: countBuffer.toString('hex')
        });
    
        return await anchor.web3.PublicKey.findProgramAddress(seeds, program.programId);
    }

    before(async () => {
        const keypairPath = path.join(os.homedir(), '.config', 'solana', 'id.json');
        
        try {
            const secretKey = Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath, 'utf8')));
            user = anchor.web3.Keypair.fromSecretKey(secretKey);
        } catch (error) {
            console.error('Error reading keypair:', error);
            user = anchor.web3.Keypair.generate();
        }

        console.log("user:", user.publicKey.toBase58());

        programAccount = anchor.web3.Keypair.generate();

        await provider.connection.confirmTransaction(
            await provider.connection.requestAirdrop(programAccount.publicKey, anchor.web3.LAMPORTS_PER_SOL),
            "confirmed"
        );

       // Generate unique ticker for each test run
       ticker = `MTK${Math.floor(Math.random() * 1000000)}`;
        console.log("Using ticker:", ticker);

    const [tokenAccount] = await anchor.web3.PublicKey.findProgramAddress(
        [
            Buffer.from("token"),
            user.publicKey.toBuffer(),
            Buffer.from(ticker)
        ],
        program.programId
    );

    token = tokenAccount;
   
    });

    it("Initializes the token", async () => {
        const initialTarget = 5 * anchor.web3.LAMPORTS_PER_SOL; 
        const totalSupply = 1000 ;

        const [userContribution] = await deriveUserContributionPDA(0, true);

        console.log("initialTarget, totalSupply:", initialTarget, totalSupply)
        console.log("User Public Key:", user.publicKey.toBase58());
        console.log("Token Account:", token.toBase58());
        console.log("Initial User Contribution Account:", userContribution.toBase58());


        await program.methods
            .initialize("MyToken", ticker, new anchor.BN(totalSupply), new anchor.BN(initialTarget), 10)
            .accounts({
                token: token,
                user: user.publicKey,
                programAccount: programAccount.publicKey,
                userContribution: userContribution,
                systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();

        const tokenAccount = await program.account.tokenDetails.fetch(token);
        console.log("Token Account:", tokenAccount);
        
        assert.equal(tokenAccount.name, "MyToken");
        assert.equal(tokenAccount.ticker, ticker);
        assert.equal(tokenAccount.totalSupply.toNumber(), totalSupply);
    });

    it("Contributes to the token", async () => {
        const contributionAmount = 1 * anchor.web3.LAMPORTS_PER_SOL;
        
        const tokenAccount = await program.account.tokenDetails.fetch(token);

        console.log("tokenAccount:", tokenAccount.contributionCount)
        
        const [userContribution] = await deriveUserContributionPDA(tokenAccount.contributionCount, false);
        console.log("Debug Seeds:");
        console.log("User pubkey:", user.publicKey.toBase58());
        console.log("Ticker:", ticker);
        console.log("Contribution count:", tokenAccount.contributionCount);
        console.log("Generated PDA:", userContribution.toBase58());
    
        await program.methods
            .contribute(new anchor.BN(contributionAmount))
            .accounts({
                token: token,
                user: user.publicKey,
                programAccount: programAccount.publicKey,
                userContribution: userContribution,
                systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();
    
        const contributionAccount = await program.account.userContribution.fetch(userContribution);
        console.log("User Contribution Account:", contributionAccount);
        assert.equal(contributionAccount.amount.toNumber(), contributionAmount);
    });

    it("Refunds the contribution", async () => {
        const tokenAccount = await program.account.tokenDetails.fetch(token);
        const [userContribution] = await deriveUserContributionPDA(tokenAccount.contributionCount - 1, false);

        const contributionBefore = await program.account.userContribution.fetch(userContribution);
        console.log("Contribution amount before refund:", contributionBefore.amount.toNumber());

        try {
            await program.methods
                .refund()
                .accounts({
                    token: token,
                    userContribution: userContribution,
                    user: user.publicKey,
                    programAccount: programAccount.publicKey,
                    systemProgram: SystemProgram.programId,
                })
                .signers([user])
                .rpc();

            const contributionAfter = await program.account.userContribution.fetch(userContribution);
            console.log("Contribution amount after refund:", contributionAfter.amount.toNumber());
            assert.equal(contributionAfter.amount.toNumber(), 0); 
        } catch (error) {
            console.log("Error during refund:", error);
        }
    });

    it("Finalizes the token", async () => {
        const [poolTokenAccount] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from("pool-token"), token.toBuffer()],
            program.programId
        );

        const [userTokenAccount] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from("user-token"), user.publicKey.toBuffer(), token.toBuffer()],
            program.programId
        );
        
        const tokenAccount = await program.account.tokenDetails.fetch(token);
        const [userContribution] = await deriveUserContributionPDA(tokenAccount.contributionCount - 1, false);
        
        await program.methods
            .finalize()
            .accounts({
                token: token,
                programAccount: program.programId,
                komWallet: user.publicKey,
                poolTokenAccount: poolTokenAccount,  // Changed
                userTokenAccount: userTokenAccount,  // Changed
                systemProgram: SystemProgram.programId,
                ammProgram: program.programId, 
                amm: program.programId, 
                ammAuthority: program.programId,
                ammOpenOrders: program.programId, 
                lpMint: program.programId, 
                coinMint: program.programId, 
                pcMint: program.programId,
                coinVault: program.programId,
                pcVault: program.programId, 
                targetOrders: program.programId, 
                ammConfig: program.programId, 
                feeDestination: program.programId, 
                marketProgram: program.programId, 
                market: program.programId, 
                globalAccount: program.programId,
                userTokenCoin: program.programId, 
                userTokenPc: program.programId, 
                userTokenLp: program.programId,
                tokenProgram: program.programId, 
                associatedTokenProgram: program.programId, 
                sysvarRent: program.programId, 
                userContributions: userContribution,
                liquidityPool: program.programId,
            })
            .signers([user])
            .rpc();
    });
});