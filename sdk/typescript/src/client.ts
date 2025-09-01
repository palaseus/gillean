/**
 * API Client for Gillean Blockchain TypeScript SDK
 */

import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import WebSocket from 'ws';
import { SDKConfig, GilleanError, WebSocketConfig, WebSocketMessage } from './types';

/**
 * HTTP API client for communicating with the Gillean blockchain
 */
export class APIClient {
  private config: SDKConfig;
  private httpClient: AxiosInstance;
  private wsConnection?: WebSocket;
  private wsConfig?: WebSocketConfig;

  constructor(config: SDKConfig) {
    this.config = config;
    this.httpClient = axios.create({
      baseURL: config.apiUrl,
      timeout: config.timeout,
      headers: {
        'Content-Type': 'application/json',
        ...(config.apiKey && { 'Authorization': `Bearer ${config.apiKey}` })
      }
    });

    // Add request interceptor for retries
    this.httpClient.interceptors.request.use(
      (config) => {
        config.metadata = { startTime: new Date() };
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Add response interceptor for error handling
    this.httpClient.interceptors.response.use(
      (response) => response,
      async (error) => {
        const config = error.config;
        
        if (config && !config._retry && this.config.retryAttempts && this.config.retryAttempts > 0) {
          config._retry = true;
          config.retryCount = (config.retryCount || 0) + 1;
          
          if (config.retryCount <= this.config.retryAttempts) {
            const delay = Math.pow(2, config.retryCount) * 1000; // Exponential backoff
            await new Promise(resolve => setTimeout(resolve, delay));
            return this.httpClient(config);
          }
        }
        
        return Promise.reject(this.handleError(error));
      }
    );
  }

  /**
   * Make a GET request
   */
  async get<T = any>(endpoint: string, config?: AxiosRequestConfig): Promise<T> {
    try {
      const response: AxiosResponse<T> = await this.httpClient.get(endpoint, config);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Make a POST request
   */
  async post<T = any>(endpoint: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    try {
      const response: AxiosResponse<T> = await this.httpClient.post(endpoint, data, config);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Make a PUT request
   */
  async put<T = any>(endpoint: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    try {
      const response: AxiosResponse<T> = await this.httpClient.put(endpoint, data, config);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Make a DELETE request
   */
  async delete<T = any>(endpoint: string, config?: AxiosRequestConfig): Promise<T> {
    try {
      const response: AxiosResponse<T> = await this.httpClient.delete(endpoint, config);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  /**
   * Connect to WebSocket
   */
  connectWebSocket(config: WebSocketConfig): void {
    if (this.wsConnection) {
      this.wsConnection.close();
    }

    this.wsConfig = config;
    this.wsConnection = new WebSocket(config.url);

    this.wsConnection.on('open', () => {
      console.log('WebSocket connected');
      config.onOpen?.();
    });

    this.wsConnection.on('message', (data: WebSocket.Data) => {
      try {
        const message: WebSocketMessage = JSON.parse(data.toString());
        config.onMessage?.(message);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    });

    this.wsConnection.on('error', (error: Error) => {
      console.error('WebSocket error:', error);
      config.onError?.(error);
    });

    this.wsConnection.on('close', () => {
      console.log('WebSocket disconnected');
      config.onClose?.();
      
      // Attempt to reconnect if configured
      if (config.reconnectInterval && config.maxReconnectAttempts) {
        this.attemptReconnect(config);
      }
    });
  }

  /**
   * Send WebSocket message
   */
  sendWebSocketMessage(message: WebSocketMessage): void {
    if (this.wsConnection && this.wsConnection.readyState === WebSocket.OPEN) {
      this.wsConnection.send(JSON.stringify(message));
    } else {
      throw new GilleanError('WebSocket not connected', 'WEBSOCKET_ERROR');
    }
  }

  /**
   * Close WebSocket connection
   */
  closeWebSocket(): void {
    if (this.wsConnection) {
      this.wsConnection.close();
      this.wsConnection = undefined;
    }
  }

  /**
   * Update client configuration
   */
  updateConfig(newConfig: SDKConfig): void {
    this.config = { ...this.config, ...newConfig };
    
    // Update axios configuration
    this.httpClient.defaults.baseURL = this.config.apiUrl;
    this.httpClient.defaults.timeout = this.config.timeout;
    
    if (this.config.apiKey) {
      this.httpClient.defaults.headers.common['Authorization'] = `Bearer ${this.config.apiKey}`;
    }
  }

  /**
   * Close all connections
   */
  async close(): Promise<void> {
    this.closeWebSocket();
    // Note: Axios doesn't need explicit cleanup
  }

  /**
   * Handle API errors
   */
  private handleError(error: any): GilleanError {
    if (error instanceof GilleanError) {
      return error;
    }

    if (axios.isAxiosError(error)) {
      const statusCode = error.response?.status || 500;
      const message = error.response?.data?.message || error.message || 'API request failed';
      const code = error.response?.data?.code || 'API_ERROR';
      
      return new GilleanError(message, code, statusCode);
    }

    return new GilleanError(
      error.message || 'Unknown error occurred',
      'UNKNOWN_ERROR',
      500
    );
  }

  /**
   * Attempt WebSocket reconnection
   */
  private attemptReconnect(config: WebSocketConfig): void {
    let attempts = 0;
    const maxAttempts = config.maxReconnectAttempts || 5;
    const interval = config.reconnectInterval || 5000;

    const reconnect = () => {
      if (attempts >= maxAttempts) {
        console.error('Max WebSocket reconnection attempts reached');
        return;
      }

      attempts++;
      console.log(`Attempting WebSocket reconnection ${attempts}/${maxAttempts}`);

      setTimeout(() => {
        this.connectWebSocket(config);
      }, interval);
    };

    reconnect();
  }
}
