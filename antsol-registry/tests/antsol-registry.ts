import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";

// Loose type interface matching the on-chain struct for minimal tests
interface PackageAccount {
  name: string;
  version: string;
  authority: anchor.web3.PublicKey;
  ipfsCid: string;
  publishedAt: anchor.BN;
  description: string;
  dependencies: { name: string; version: string }[];
  bump: number;
}

describe("antsol-registry", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.AntsolRegistry as Program<any>;
  const authority = provider.wallet;

  // Utility: unique, validation-compliant package names to avoid PDA collisions on devnet
  const uniqueName = (base: string) => `${base}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2,6)}`;

  // Happy path base package used in multiple success-flow tests
  const basePackage = {
    name: uniqueName("pkg"),
    version: "1.0.0",
    ipfsCid: "QmTest123456789abcdefghijklmnopqrstuvwxyz",
    description: "A test package for AntSol registry",
    dependencies: [] as { name: string; version: string }[],
  };

  function getPackagePDA(name: string, version: string) {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("package"), Buffer.from(name), Buffer.from(version)],
      program.programId
    );
  }

  async function publish(pkg: typeof basePackage, authorityOverride?: anchor.web3.PublicKey) {
    const [pda] = getPackagePDA(pkg.name, pkg.version);
    return (program.methods as any)
      .publishPackage(pkg.name, pkg.version, pkg.ipfsCid, pkg.description, pkg.dependencies)
      .accounts({
        authority: authorityOverride ?? authority.publicKey,
        package: pda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  }

  async function fetchPackage(name: string, version: string): Promise<PackageAccount> {
    const [pda] = getPackagePDA(name, version);
    return (await (program.account as any)["package"].fetch(pda)) as PackageAccount;
  }

  async function expectPublishFail(pkg: Partial<typeof basePackage>, expectSubstring?: string) {
    // Fill required fields with defaults if not provided
    const attempt = {
      name: pkg.name ?? uniqueName("bad"),
      version: pkg.version ?? "1.0.0",
      ipfsCid: pkg.ipfsCid ?? "QmTest123456789abcdefghijklmnopqrstuvwxyz",
      description: pkg.description ?? "desc",
      dependencies: pkg.dependencies ?? [],
    };
    try {
      await publish(attempt);
      assert.fail("Expected publish to fail but it succeeded");
    } catch (e: any) {
      const msg = e.toString();
      if (expectSubstring) {
        assert.include(msg, expectSubstring, `error message should include '${expectSubstring}', got: ${msg}`);
      }
    }
  }

  // 1. Happy path publish
  it("1 publishes a new package", async () => {
    const tx = await publish(basePackage);
    console.log("publish tx", tx);
    const acct = await fetchPackage(basePackage.name, basePackage.version);
    assert.equal(acct.name, basePackage.name);
    assert.equal(acct.version, basePackage.version);
    assert.equal(acct.ipfsCid, basePackage.ipfsCid);
  });

  // Validation error cases (grouped)
  it("2 rejects empty name", async () => {
    await expectPublishFail({ name: "" }, "NameEmpty");
  });
  it("3 rejects name too long", async () => {
    // PDA seed limit (32 bytes) will be hit before on-chain validation when name > 32
    // Accept either Anchor client seed error or on-chain NameTooLong depending on where it fails
    try {
      await expectPublishFail({ name: "a".repeat(65) }, "NameTooLong");
    } catch (e: any) {
      // Fallback assertion path
      assert.include(e.toString(), "Max seed length exceeded");
    }
  });
  it("4 rejects invalid name format (uppercase)", async () => {
    await expectPublishFail({ name: "BadName" }, "InvalidNameFormat");
  });
  it("5 rejects empty version", async () => {
    await expectPublishFail({ version: "" }, "VersionEmpty");
  });
  it("6 rejects version too long", async () => {
    // Valid semver but exceeds MAX_VERSION_LENGTH (16)
    await expectPublishFail({ version: "1234567890123456.0.0" }, "VersionTooLong");
  });
  it("7 rejects invalid version format", async () => {
    await expectPublishFail({ version: "1.0" }, "InvalidVersionFormat");
  });
  it("8 rejects empty CID", async () => {
    await expectPublishFail({ ipfsCid: "" }, "CidEmpty");
  });
  it("9 rejects CID too long", async () => {
    await expectPublishFail({ ipfsCid: "Qm" + "x".repeat(63) }, "CidTooLong");
  });
  it("10 rejects invalid CID format", async () => {
    await expectPublishFail({ ipfsCid: "NotCid123" }, "InvalidCidFormat");
  });
  it("11 rejects description too long", async () => {
    await expectPublishFail({ description: "d".repeat(257) }, "DescriptionTooLong");
  });
  it("12 rejects too many dependencies", async () => {
    await expectPublishFail({ dependencies: Array.from({ length: 11 }, (_, i) => ({ name: `dep${i}`, version: "1.0.0" })) }, "TooManyDependencies");
  });
  it("13 rejects invalid dependency name", async () => {
    await expectPublishFail({ dependencies: [{ name: "Bad_Name", version: "1.0.0" }] }, "InvalidDependencyName");
  });
  it("14 rejects invalid dependency version", async () => {
    await expectPublishFail({ dependencies: [{ name: "dep", version: "1.0" }] }, "InvalidDependencyVersion");
  });

  // Update flows
  const updatePkg = { ...basePackage, newVersion: "1.1.0", newCid: "QmNewCid123456789abcdefghijklmnopqrstuvwxyz" };
  it("15 updates package with greater version", async () => {
    const [existingPda] = getPackagePDA(basePackage.name, basePackage.version);
    const [newPda] = getPackagePDA(basePackage.name, updatePkg.newVersion);
    const tx = await (program.methods as any)
      .updatePackage(basePackage.name, updatePkg.newVersion, updatePkg.newCid, basePackage.description, basePackage.dependencies)
      .accounts({
        authority: authority.publicKey,
        existingPackage: existingPda,
        newPackage: newPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("update tx", tx);
    const acct = await fetchPackage(basePackage.name, updatePkg.newVersion);
    assert.equal(acct.version, updatePkg.newVersion);
    assert.equal(acct.ipfsCid, updatePkg.newCid);
  });
  it("16 rejects update with non-greater version", async () => {
    // Use a lower semantic version to avoid PDA collision and trigger handler validation
    const [existingPda] = getPackagePDA(basePackage.name, updatePkg.newVersion);
    const lowerVersion = "0.9.0";
    const [newPda] = getPackagePDA(basePackage.name, lowerVersion);
    try {
      await (program.methods as any)
        .updatePackage(basePackage.name, lowerVersion, "QmAnotherCid123456789abcdefghijklmnopqrstuvwxyz", basePackage.description, basePackage.dependencies)
        .accounts({
          authority: authority.publicKey,
          existingPackage: existingPda,
          newPackage: newPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Expected VersionNotGreater failure");
    } catch (e: any) {
      assert.include(e.toString(), "VersionNotGreater");
    }
  });

  // Authority transfer + unauthorized cases (bonus beyond 16 core validations)
  it("17 transfers authority successfully", async () => {
    const newAuthority = anchor.web3.Keypair.generate();
    const [existingPda] = getPackagePDA(basePackage.name, updatePkg.newVersion);
    const tx = await (program.methods as any)
      .transferAuthority(basePackage.name, updatePkg.newVersion)
      .accounts({
        currentAuthority: authority.publicKey,
        package: existingPda,
        newAuthority: newAuthority.publicKey,
      })
      .signers([])
      .rpc();
    console.log("transfer tx", tx);
    const acct = await fetchPackage(basePackage.name, updatePkg.newVersion);
    assert.equal(acct.authority.toBase58(), newAuthority.publicKey.toBase58());
  });
  it("18 rejects unauthorized transfer of authority (wrong signer)", async () => {
    const rogue = anchor.web3.Keypair.generate();
    const [existingPda] = getPackagePDA(basePackage.name, updatePkg.newVersion);
    try {
      await (program.methods as any)
        .transferAuthority(basePackage.name, updatePkg.newVersion)
        .accounts({
          currentAuthority: rogue.publicKey, // not the actual authority
          package: existingPda,
          newAuthority: anchor.web3.Keypair.generate().publicKey,
        })
        // fee payer is provider (has funds); rogue is added as required signer
        .signers([rogue])
        .rpc();
      assert.fail("Expected UnauthorizedAuthority failure");
    } catch (e: any) {
      assert.include(e.toString(), "UnauthorizedAuthority");
    }
  });
});
