interface Block {
    hash: string;
    previousHash: string;
    timestamp: number;
    transactions: Transaction[];
}

interface Transaction {
    from: string;
    to: string;
    amount: number;
    data?: string;
}

interface ApiResponse<T> {
    success: boolean;
    data: T;
    error?: string;
}

export class BlockchainClient {
    private baseUrl: string;

    constructor(baseUrl: string) {
        this.baseUrl = baseUrl;
    }

    async getBlock(blockHash: string): Promise<Block> {
        const response = await fetch(`${this.baseUrl}/blocks/${blockHash}`);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const result: ApiResponse<Block> = await response.json();
        return result.data;
    }

    async sendTransaction(transaction: Transaction): Promise<string> {
        const response = await fetch(`${this.baseUrl}/transactions`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(transaction),
        });
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const result: ApiResponse<{ txHash: string }> = await response.json();
        return result.data.txHash;
    }

    async getBalance(address: string): Promise<number> {
        const response = await fetch(`${this.baseUrl}/balance/${address}`);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const result: ApiResponse<{ balance: number }> = await response.json();
        return result.data.balance;
    }

    async getTransactionHistory(address: string): Promise<Transaction[]> {
        const response = await fetch(`${this.baseUrl}/transactions/${address}`);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const result: ApiResponse<Transaction[]> = await response.json();
        return result.data;
    }
}
