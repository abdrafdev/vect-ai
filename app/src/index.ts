import { Connection, clusterApiUrl, PublicKey, Keypair } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, Wallet } from "@coral-xyz/anchor";

// Program IDs (from Anchor.toml)
const VECTAI_TOKEN_PROGRAM_ID = new PublicKey("DfpsT9PAeWbwwfE8EqTDqVUiCrsoHF1fogmPw42eqLPH");
const VECTAI_ORACLE_PROGRAM_ID = new PublicKey("8FWpTEk2NPut6MrKXiCGVzz9ZY247fcYGdL9TEoXFqzw");
const VECTAI_TRADER_PROGRAM_ID = new PublicKey("FEmf6TbtffcKVptbshZvCcg3CjQqsWodNwQhpXJff4NP");

// Pyth price feed IDs for testing (devnet)
const PYTH_FEEDS = {
  "BTC/USD": "GVXRSBjFk6e6J3NbVPXohDJetcTjaeeuykUpbQF8UoMU",
  "ETH/USD": "JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB",
  "SOL/USD": "H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG",
};

class VectaiApp {
  private connection: Connection;
  private provider: AnchorProvider;

  constructor() {
    // Connect to Solana devnet
    this.connection = new Connection(clusterApiUrl("devnet"), "confirmed");
    
    // Create a dummy wallet for demo purposes
    const wallet = new Wallet(Keypair.generate());
    
    // Set up Anchor provider
    this.provider = new AnchorProvider(
      this.connection,
      wallet,
      AnchorProvider.defaultOptions()
    );
    anchor.setProvider(this.provider);
  }

  async initialize(): Promise<void> {
    try {
      console.log("🪙 VECT.AI Solana MVP - Initializing...");
      
      // Test connection
      const version = await this.connection.getVersion();
      console.log("✅ Connected to Solana:", version);
      
      // Get cluster info
      const cluster = this.connection.rpcEndpoint;
      console.log("🌐 Cluster:", cluster);
      
      // Check program deployments (these will fail until deployed)
      await this.checkProgramDeployments();
      
      // Display Pyth feed information
      this.displayPythFeeds();
      
      console.log("🚀 VECT.AI app initialization complete!");
      console.log("📋 Ready for:");
      console.log("   • Token minting and transfers");
      console.log("   • Price data fetching from Pyth");
      console.log("   • Conditional swap execution");
      
    } catch (error) {
      console.error("❌ Initialization failed:", error);
    }
  }

  private async checkProgramDeployments(): Promise<void> {
    try {
      console.log("🔍 Checking program deployments...");
      
      const programs = [
        { name: "VECTAI Token", id: VECTAI_TOKEN_PROGRAM_ID },
        { name: "VECTAI Oracle", id: VECTAI_ORACLE_PROGRAM_ID },
        { name: "VECTAI Trader", id: VECTAI_TRADER_PROGRAM_ID },
      ];

      for (const program of programs) {
        try {
          const accountInfo = await this.connection.getAccountInfo(program.id);
          if (accountInfo) {
            console.log(`✅ ${program.name} deployed`);
          } else {
            console.log(`⚠️  ${program.name} not deployed yet`);
          }
        } catch (error) {
          console.log(`⚠️  ${program.name} check failed:`, error);
        }
      }
    } catch (error) {
      console.warn("⚠️  Program deployment check failed:", error);
    }
  }

  private displayPythFeeds(): void {
    console.log("📈 Available Pyth Price Feeds (Devnet):");
    Object.entries(PYTH_FEEDS).forEach(([pair, feedId]) => {
      console.log(`   ${pair}: ${feedId}`);
    });
  }

  async demonstrateWorkflow(): Promise<void> {
    console.log("\n🔄 Demonstrating VECT.AI Workflow:");
    console.log("   1. Fetch Price → 2. Check Condition → 3. Execute Swap");
    
    try {
      // This is a conceptual demonstration
      // Actual implementation would require deployed programs
      
      console.log("📊 Step 1: Fetching BTC/USD price from Pyth...");
      // const price = await this.oracleProgram.methods.getPrice().rpc();
      console.log("💰 Current BTC price: $45,000 (simulated)");
      
      console.log("🎯 Step 2: Checking if price > $40,000 threshold...");
      console.log("✅ Condition met: 45,000 > 40,000");
      
      console.log("🔄 Step 3: Executing conditional swap...");
      // await this.traderProgram.methods.executeConditionalSwap().rpc();
      console.log("✅ Swap executed successfully! (simulated)");
      
    } catch (error) {
      console.log("ℹ️  Simulation complete - deploy programs to test live");
    }
  }

  async getConnectionInfo(): Promise<void> {
    try {
      const slot = await this.connection.getSlot();
      const blockHeight = await this.connection.getBlockHeight();
      
      console.log("\n📡 Connection Info:");
      console.log(`   Current Slot: ${slot}`);
      console.log(`   Block Height: ${blockHeight}`);
      console.log(`   Endpoint: ${this.connection.rpcEndpoint}`);
    } catch (error) {
      console.error("Failed to get connection info:", error);
    }
  }
}

// Main execution
async function main() {
  console.log("Connection ready! 🚀");
  
  const app = new VectaiApp();
  
  // Initialize the application
  await app.initialize();
  
  // Get connection information
  await app.getConnectionInfo();
  
  // Demonstrate the workflow
  await app.demonstrateWorkflow();
  
  console.log("\n🎉 VECT.AI Solana MVP Demo Complete!");
  console.log("Next steps:");
  console.log("1. Deploy programs: anchor deploy");
  console.log("2. Initialize token mint");
  console.log("3. Set up oracle with Pyth feeds");
  console.log("4. Configure trader with conditions");
  console.log("5. Execute live conditional swaps");
}

// Handle async main
if (require.main === module) {
  main().catch((error) => {
    console.error("Application error:", error);
    process.exit(1);
  });
}