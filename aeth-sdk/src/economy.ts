import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AethProtocol } from "../target/types/aeth_protocol";
import { PublicKey, SystemProgram } from "@solana/web3.js";

export class AethEconomy {
  constructor(public program: Program<AethProtocol>) {}

  async initializePool(feeBps: number) {
    const [poolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool")],
      this.program.programId
    );

    return await this.program.methods
      .initializeEconomy(new anchor.BN(feeBps))
      .accounts({
        pool: poolPda,
        authority: this.program.provider.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async stakeTokens(amount: number, userTokenAccount: PublicKey, poolVault: PublicKey) {
    const [stakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), this.program.provider.publicKey!.toBuffer()],
      this.program.programId
    );

    const [poolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool")],
      this.program.programId
    );

    return await this.program.methods
      .stake(new anchor.BN(amount))
      .accounts({
        user: this.program.provider.publicKey,
        stakeAccount: stakePda,
        pool: poolPda,
        userToken: userTokenAccount,
        poolVault: poolVault,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async claimRewards(userTokenAccount: PublicKey, poolVault: PublicKey) {
    const [stakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), this.program.provider.publicKey!.toBuffer()],
      this.program.programId
    );

    const [poolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool")],
      this.program.programId
    );

    return await this.program.methods
      .claimRewards()
      .accounts({
        user: this.program.provider.publicKey,
        stakeAccount: stakePda,
        pool: poolPda,
        userToken: userTokenAccount,
        poolVault: poolVault,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .rpc();
  }

  async commitBatch(batchId: number, proofHash: number[]) {
    const [batchPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("batch"), new anchor.BN(batchId).toArrayLike(Buffer, "le", 8)],
      this.program.programId
    );

    const [stakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), this.program.provider.publicKey!.toBuffer()],
      this.program.programId
    );

    return await this.program.methods
      .commitBatch(new anchor.BN(batchId), Array.from(proofHash))
      .accounts({
        sequencer: this.program.provider.publicKey,
        sequencerStake: stakePda,
        batchRecord: batchPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }
}
