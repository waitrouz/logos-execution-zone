import { useState, useEffect } from 'react';
import { Shield, Key, Download, Upload, CheckCircle, AlertCircle, Loader } from 'lucide-react';
import toast, { Toaster } from 'react-hot-toast';

// Types
interface AirdropInfo {
  id: string;
  tokenName: string;
  merkleRoot: string;
  totalAmount: number;
  recipientCount: number;
  claimedCount: number;
}

interface ClaimStatus {
  hasClaimed: boolean;
  amount?: number;
  txHash?: string;
}

// Mock API calls (replace with actual LEZ SDK integration)
const mockAirdrops: AirdropInfo[] = [
  {
    id: 'airdrop_001',
    tokenName: 'PRIV Token',
    merkleRoot: '0x7a8f9c3d2e1b5a4c6d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c',
    totalAmount: 1000000,
    recipientCount: 100,
    claimedCount: 45,
  },
  {
    id: 'airdrop_002',
    tokenName: 'Shield NFT Access',
    merkleRoot: '0x1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c',
    totalAmount: 500,
    recipientCount: 50,
    claimedCount: 12,
  },
];

function App() {
  const [connected, setConnected] = useState(false);
  const [address, setAddress] = useState('');
  const [nullifierSecret, setNullifierSecret] = useState('');
  const [selectedAirdrop, setSelectedAirdrop] = useState<AirdropInfo | null>(null);
  const [claimStatus, setClaimStatus] = useState<ClaimStatus | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [activeTab, setActiveTab] = useState<'browse' | 'claim' | 'status'>('browse');

  // Connect to Logos wallet
  const connectWallet = async () => {
    try {
      // TODO: Integrate with actual Logos SDK
      // const logos = await window.logos.connect();
      // const addr = await logos.getAddress();
      
      // Mock connection
      await new Promise(resolve => setTimeout(resolve, 1000));
      const mockAddress = '0x' + Array(64).fill(0).map(() => Math.floor(Math.random() * 16).toString(16)).join('');
      setAddress(mockAddress);
      setConnected(true);
      toast.success('Wallet connected successfully!');
    } catch (error) {
      toast.error('Failed to connect wallet');
      console.error(error);
    }
  };

  // Generate nullifier secret if not provided
  const generateSecret = () => {
    const bytes = new Uint8Array(32);
    crypto.getRandomValues(bytes);
    const hex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
    setNullifierSecret(hex);
    toast.success('Nullifier secret generated! Keep this private!');
  };

  // Check claim status
  const checkClaimStatus = async () => {
    if (!selectedAirdrop || !nullifierSecret) {
      toast.error('Please select an airdrop and generate your nullifier secret');
      return;
    }

    try {
      // TODO: Query actual chain state
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      // Mock response
      setClaimStatus({
        hasClaimed: false,
        amount: 10000,
      });
      toast.success('Claim status checked');
    } catch (error) {
      toast.error('Failed to check claim status');
    }
  };

  // Generate claim proof
  const generateClaimProof = async () => {
    if (!selectedAirdrop || !nullifierSecret) {
      toast.error('Please select an airdrop and provide nullifier secret');
      return;
    }

    setIsGenerating(true);
    try {
      // TODO: Call actual Risc0 prover
      // const receipt = await generateProof({
      //   airdropId: selectedAirdrop.id,
      //   address,
      //   nullifierSecret,
      // });
      
      await new Promise(resolve => setTimeout(resolve, 3000));
      
      toast.success('Claim proof generated! Ready to submit.');
      return true;
    } catch (error) {
      toast.error('Failed to generate proof: ' + (error as Error).message);
      return false;
    } finally {
      setIsGenerating(false);
    }
  };

  // Submit claim
  const submitClaim = async () => {
    if (!selectedAirdrop) return;

    const proofGenerated = await generateClaimProof();
    if (!proofGenerated) return;

    setIsSubmitting(true);
    try {
      // TODO: Submit to LEZ network
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      setClaimStatus({
        hasClaimed: true,
        amount: 10000,
        txHash: '0x' + Array(64).fill(0).map(() => Math.floor(Math.random() * 16).toString(16)).join(''),
      });
      
      toast.success('Claim submitted successfully!');
      setActiveTab('status');
    } catch (error) {
      toast.error('Failed to submit claim: ' + (error as Error).message);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Download claim package
  const downloadClaimPackage = () => {
    const packageData = {
      airdropId: selectedAirdrop?.id,
      timestamp: Date.now(),
      note: 'This is a mock claim package for demonstration',
    };
    
    const blob = new Blob([JSON.stringify(packageData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `claim-package-${selectedAirdrop?.id}.json`;
    a.click();
    URL.revokeObjectURL(url);
    
    toast.success('Claim package downloaded!');
  };

  return (
    <div className="app">
      <Toaster position="top-right" />
      
      {/* Header */}
      <header style={{ padding: '1.5em', borderBottom: '1px solid #333', marginBottom: '2em' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', maxWidth: '1200px', margin: '0 auto' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5em' }}>
            <Shield size={32} color="#646cff" />
            <h1 style={{ margin: 0, fontSize: '1.5em' }}>Private Airdrop</h1>
          </div>
          
          {!connected ? (
            <button onClick={connectWallet} style={{ display: 'flex', alignItems: 'center', gap: '0.5em' }}>
              <Key size={18} />
              Connect Wallet
            </button>
          ) : (
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5em', fontSize: '0.9em' }}>
              <CheckCircle size={18} color="#51cf66" />
              <span>{address.slice(0, 10)}...{address.slice(-8)}</span>
            </div>
          )}
        </div>
      </header>

      {/* Main Content */}
      <main style={{ maxWidth: '1200px', margin: '0 auto', padding: '0 1.5em' }}>
        {/* Tabs */}
        <div style={{ display: 'flex', gap: '1em', marginBottom: '2em', borderBottom: '1px solid #333', paddingBottom: '1em' }}>
          <button
            onClick={() => setActiveTab('browse')}
            style={{ background: activeTab === 'browse' ? '#646cff' : 'transparent' }}
          >
            Browse Airdrops
          </button>
          <button
            onClick={() => setActiveTab('claim')}
            style={{ background: activeTab === 'claim' ? '#646cff' : 'transparent' }}
            disabled={!connected}
          >
            Claim
          </button>
          <button
            onClick={() => setActiveTab('status')}
            style={{ background: activeTab === 'status' ? '#646cff' : 'transparent' }}
            disabled={!connected}
          >
            My Claims
          </button>
        </div>

        {/* Browse Tab */}
        {activeTab === 'browse' && (
          <div>
            <h2>Available Airdrops</h2>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '1.5em' }}>
              {mockAirdrops.map((airdrop) => (
                <div key={airdrop.id} className="card">
                  <h3 style={{ marginTop: 0 }}>{airdrop.tokenName}</h3>
                  <p><strong>ID:</strong> {airdrop.id}</p>
                  <p><strong>Total Amount:</strong> {airdrop.totalAmount.toLocaleString()}</p>
                  <p><strong>Recipients:</strong> {airdrop.recipientCount}</p>
                  <p><strong>Claimed:</strong> {airdrop.claimedCount} ({Math.round(airdrop.claimedCount / airdrop.recipientCount * 100)}%)</p>
                  <p style={{ fontSize: '0.8em', opacity: 0.7 }}>
                    <strong>Merkle Root:</strong> {airdrop.merkleRoot.slice(0, 20)}...
                  </p>
                  <button 
                    onClick={() => {
                      setSelectedAirdrop(airdrop);
                      setActiveTab('claim');
                    }}
                    style={{ width: '100%', marginTop: '1em' }}
                  >
                    Select & Claim
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Claim Tab */}
        {activeTab === 'claim' && (
          <div style={{ maxWidth: '600px' }}>
            <h2>Claim Your Airdrop</h2>
            
            {!selectedAirdrop ? (
              <div className="card">
                <AlertCircle size={48} style={{ margin: '0 auto 1em', display: 'block' }} />
                <p style={{ textAlign: 'center' }}>Please select an airdrop from the Browse tab first.</p>
                <button onClick={() => setActiveTab('browse')} style={{ width: '100%' }}>
                  Browse Airdrops
                </button>
              </div>
            ) : (
              <>
                <div className="card">
                  <h3 style={{ marginTop: 0 }}>Selected Airdrop</h3>
                  <p><strong>{selectedAirdrop.tokenName}</strong></p>
                  <p>ID: {selectedAirdrop.id}</p>
                  <p>Your allocation will be revealed only after successful claim.</p>
                </div>

                <div className="card">
                  <h3 style={{ marginTop: 0 }}>Step 1: Nullifier Secret</h3>
                  <p style={{ fontSize: '0.9em', opacity: 0.8 }}>
                    This secret ensures you can only claim once while keeping your identity private.
                    <strong> Never share this with anyone!</strong>
                  </p>
                  
                  {!nullifierSecret ? (
                    <button onClick={generateSecret} style={{ width: '100%' }}>
                      Generate New Secret
                    </button>
                  ) : (
                    <>
                      <textarea
                        value={nullifierSecret}
                        onChange={(e) => setNullifierSecret(e.target.value)}
                        rows={3}
                        style={{ fontFamily: 'monospace', fontSize: '0.8em' }}
                      />
                      <button onClick={generateSecret} style={{ width: '100%', marginTop: '1em' }}>
                        Regenerate Secret
                      </button>
                    </>
                  )}
                </div>

                <div className="card">
                  <h3 style={{ marginTop: 0 }}>Step 2: Generate & Submit Claim</h3>
                  
                  {claimStatus?.hasClaimed ? (
                    <div className="success">
                      <CheckCircle size={24} style={{ display: 'inline', marginRight: '0.5em' }} />
                      <strong>Already Claimed!</strong>
                      <p>You have already claimed {claimStatus.amount} tokens.</p>
                      {claimStatus.txHash && (
                        <p style={{ fontSize: '0.8em' }}>TX: {claimStatus.txHash.slice(0, 20)}...</p>
                      )}
                    </div>
                  ) : (
                    <>
                      <button
                        onClick={submitClaim}
                        disabled={isGenerating || isSubmitting || !nullifierSecret}
                        style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5em' }}
                      >
                        {(isGenerating || isSubmitting) ? (
                          <>
                            <Loader className="spin" size={18} />
                            {isGenerating ? 'Generating Proof...' : 'Submitting Claim...'}
                          </>
                        ) : (
                          <>
                            <Shield size={18} />
                            Generate Proof & Claim
                          </>
                        )}
                      </button>
                      
                      <p style={{ fontSize: '0.8em', opacity: 0.7, marginTop: '1em' }}>
                        This will generate a zero-knowledge proof that proves your eligibility
                        without revealing which address on the allowlist you control.
                      </p>
                    </>
                  )}
                </div>

                {claimStatus?.hasClaimed && (
                  <div className="card">
                    <h3 style={{ marginTop: 0 }}>Download Receipt</h3>
                    <button onClick={downloadClaimPackage} style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5em' }}>
                      <Download size={18} />
                      Download Claim Package
                    </button>
                  </div>
                )}
              </>
            )}
          </div>
        )}

        {/* Status Tab */}
        {activeTab === 'status' && (
          <div style={{ maxWidth: '600px' }}>
            <h2>My Claim Status</h2>
            
            {claimStatus ? (
              claimStatus.hasClaimed ? (
                <div className="card success">
                  <CheckCircle size={48} style={{ margin: '0 auto 1em', display: 'block' }} />
                  <h3 style={{ textAlign: 'center', marginTop: 0 }}>Claimed Successfully!</h3>
                  <p><strong>Airdrop:</strong> {selectedAirdrop?.tokenName || 'N/A'}</p>
                  <p><strong>Amount:</strong> {claimStatus.amount} tokens</p>
                  {claimStatus.txHash && (
                    <p><strong>Transaction:</strong> {claimStatus.txHash.slice(0, 20)}...</p>
                  )}
                </div>
              ) : (
                <div className="card">
                  <AlertCircle size={48} style={{ margin: '0 auto 1em', display: 'block' }} />
                  <h3 style={{ textAlign: 'center', marginTop: 0 }}>Not Yet Claimed</h3>
                  <p>You are eligible but haven't claimed yet.</p>
                  <button onClick={() => setActiveTab('claim')} style={{ width: '100%' }}>
                    Claim Now
                  </button>
                </div>
              )
            ) : (
              <div className="card">
                <p>Select an airdrop and check your claim status.</p>
                <button onClick={() => setActiveTab('browse')} style={{ width: '100%' }}>
                  Browse Airdrops
                </button>
              </div>
            )}
          </div>
        )}
      </main>

      {/* Footer */}
      <footer style={{ padding: '2em', textAlign: 'center', marginTop: '3em', borderTop: '1px solid #333', opacity: 0.7 }}>
        <p>Private Airdrop Mini App for Logos Execution Zone</p>
        <p style={{ fontSize: '0.8em' }}>
          Your privacy is protected by zero-knowledge proofs. 
          Claims are unlinkable to your identity.
        </p>
      </footer>

      <style>{`
        .spin {
          animation: spin 1s linear infinite;
        }
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

export default App;
