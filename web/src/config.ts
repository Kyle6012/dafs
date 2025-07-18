// DAFS Web Dashboard Configuration

export const config = {
  // Backend API URL - can be configured via environment variable
  apiUrl: import.meta.env.VITE_API_URL || 'http://localhost:6543',
  
  // Default timeout for API requests (30 seconds)
  apiTimeout: 30000,
  
  // Pagination defaults
  defaultPageSize: 20,
  
  // File upload settings
  maxFileSize: 100 * 1024 * 1024, // 100MB
  allowedFileTypes: ['*/*'], // Allow all file types
  
  // UI settings
  theme: {
    primaryColor: '#1976d2',
    secondaryColor: '#dc004e',
  },
  
  // Feature flags
  features: {
    enableRealTimeUpdates: true,
    enableFileSharing: true,
    enableAI: true,
    enablePeerManagement: true,
  },
};

// Helper function to get backend URL
export const getBackendUrl = (): string => {
  return config.apiUrl;
};

// Helper function to build API endpoint
export const buildApiUrl = (endpoint: string): string => {
  const baseUrl = getBackendUrl();
  const cleanEndpoint = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
  return `${baseUrl}${cleanEndpoint}`;
}; 